use crate::db::Pool;
use anyhow::Result;
use chrono::{DateTime, LocalResult, TimeZone, Utc};
use duckdb::params;
use futures::{stream, StreamExt};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION, LINK},
    Url,
};
use tracing::{debug, error, info};

/// Load all outstanding Github pulls into the `github_pull` table
pub fn load_pulls(pool: Pool) -> Result<()> {
    let mut conn = pool.get()?;

    let tx = conn.transaction()?;
    tx.execute_batch(
        r#"
WITH pulls_rows AS (
SELECT
    id,
    "data_source",
    metadata->>'$."owner"' AS owner,
    metadata->>'$."repo"' AS repo,
    unnest(json_transform_strict("data",
        '[{
            "user": {
                "login": "VARCHAR"
            },
            "assignee": {
                "login": "VARCHAR"
            },
            "number": "VARCHAR",
            "state": "VARCHAR",
            "title": "VARCHAR",
            "url": "VARCHAR",
            "created_at": "TIMESTAMP",
            "updated_at": "TIMESTAMP",
            "closed_at": "TIMESTAMP",
            "merged_at": "TIMESTAMP",
            "author_association": "VARCHAR",
            "draft": "BOOLEAN"
        }]')) AS row,
FROM wallowa_raw_data
WHERE loaded_at IS NULL
AND "data_source" = 'github_rest_api'
AND data_type = 'pulls'
)
INSERT INTO github_pull (
    raw_data_id,
    "data_source",
    "owner",
    repo,
    "number",
    "state",
    user,
    assignee,
    assignees,
    requested_reviewers,
    requested_teams,
    title,
    "url",
    created_at,
    updated_at,
    closed_at,
    merged_at,
    author_association,
    draft
)
SELECT
    id,
    "data_source",
    "owner",
    repo,
    row.number,
    row.state,
    row.user.login,
    row.assignee.login,
    NULL, -- TODO fix me
    NULL, -- TODO fix me
    NULL, -- TODO fix me
    row.title,
    row.url,
    row.created_at,
    row.updated_at,
    row.closed_at,
    row.merged_at,
    row.author_association,
    row.draft
FROM pulls_rows
ON CONFLICT ("number", "owner", repo) DO UPDATE SET
    state = excluded.state,
    user = excluded.user,
    assignee = excluded.assignee,
    assignees = excluded.assignees,
    requested_reviewers = excluded.requested_reviewers,
    requested_teams = excluded.requested_teams,
    title = excluded.title,
    "url" = excluded."url",
    created_at = excluded.created_at,
    updated_at = excluded.updated_at,
    closed_at = excluded.closed_at,
    merged_at = excluded.merged_at,
    author_association = excluded.author_association,
    draft = excluded.draft;

-- Update the loaded_at timestamp for all of the rows that were just loaded
UPDATE wallowa_raw_data
SET loaded_at = now()
WHERE loaded_at IS NULL
AND "data_source" = 'github_rest_api'
AND data_type = 'pulls';
"#,
    )?;

    tx.commit()?;

    Ok(())
}

/// Insert raw pulls data from a single Github API response
fn fetch_pulls_single_repo(pool: Pool, response: &ResponseInfo) -> Result<()> {
    let mut conn = pool.get()?;

    // Insert the raw JSON into the database
    let tx = conn.transaction()?;

    tx.execute(
        r#"
INSERT INTO wallowa_raw_data (
    "data_source",
    data_type,
    metadata,
    "data"
) VALUES (
    'github_rest_api',
    'pulls',
    to_json({owner: ?, repo: ?}),
    ?);
    "#,
        params![&response.owner, &response.repo, response.response],
    )?;

    // Update the high watermark for this request
    tx.execute(
        r#"
INSERT OR REPLACE INTO wallowa_watermark (
    request_url,
    watermark
) VALUES (
    ?,
    ?
);
"#,
        params![response.request_url, response.watermark],
    )?;

    tx.commit()?;

    Ok(())
}

/// Insert raw data for all of the pulls from the given GitHub API responses
pub fn fetch_pulls(pool: Pool, responses: &Vec<ResponseInfo>) -> Result<()> {
    // Fetch the commits from each response
    for response in responses {
        if response.status != 200 {
            error!(
                "Skipping response for '{}' with status {}",
                response.request_url, response.status
            );
            continue;
        }

        fetch_pulls_single_repo(pool.clone(), response)?;
    }

    Ok(())
}

/// Load all outstanding Github commits into the `github_commit` table
pub fn load_commits(pool: Pool) -> Result<()> {
    let mut conn = pool.get()?;

    let tx = conn.transaction()?;
    tx.execute_batch(
        r#"
WITH commit_rows AS (
SELECT
    id,
    "data_source",
    metadata->>'$."owner"' AS owner,
    metadata->>'$."repo"' AS repo,
    unnest(json_transform_strict(data,
        '[{
            "author": {
                "avatar_url": "VARCHAR"
            },
            "commit": {
                "message": "VARCHAR",
                "author": {
                    "email": "VARCHAR"
                },
                "committer": {
                    "email": "VARCHAR",
                    "date": "TIMESTAMP"
                }
            },
            "sha": "VARCHAR",
            "url": "VARCHAR"
        }]')) AS row, 
FROM wallowa_raw_data
WHERE loaded_at IS NULL
AND "data_source" = 'github_rest_api'
AND data_type = 'github_commit'
)
INSERT INTO github_commit (
    raw_data_id,
    "data_source",
    "owner",
    repo,
    sha,
    author,
    committer,
    "message",
    "url",
    "timestamp"
)
SELECT
    id,
    "data_source",
    "owner",
    repo,
    row.sha,
    row.commit.author.email,
    row.commit.committer.email,
    row.commit.message,
    row.url,
    row.commit.committer.date
FROM commit_rows
ON CONFLICT DO NOTHING;

-- Update the loaded_at timestamp for all of the rows that were just loaded
UPDATE wallowa_raw_data
SET loaded_at = now()
WHERE loaded_at IS NULL
AND "data_source" = 'github_rest_api'
AND data_type = 'github_commit';
"#,
    )?;

    tx.commit()?;

    Ok(())
}

/// Insert raw commit data from a single Github API response
fn fetch_commits_single_repo(pool: Pool, response: &ResponseInfo) -> Result<()> {
    let mut conn = pool.get()?;

    // Insert the raw JSON into the database
    let tx = conn.transaction()?;

    tx.execute(
        r#"
INSERT INTO wallowa_raw_data (
    "data_source",
    data_type,
    metadata,
    "data"
) VALUES (
    'github_rest_api',
    'github_commit',
    to_json({owner: ?, repo: ?}),
    ?);
    "#,
        params![&response.owner, &response.repo, response.response],
    )?;

    // Update the high watermark for this request
    tx.execute(
        r#"
INSERT OR REPLACE INTO wallowa_watermark (
    request_url,
    watermark
) VALUES (
    ?,
    ?
);
"#,
        params![response.request_url, response.watermark],
    )?;

    tx.commit()?;

    Ok(())
}

/// Insert raw data for all of the commits from the given GitHub API responses
pub fn fetch_commits(pool: Pool, responses: &Vec<ResponseInfo>) -> Result<()> {
    // Fetch the commits from each response
    for response in responses {
        if response.status != 200 {
            error!(
                "Skipping response for '{}' with status {}",
                response.request_url, response.status
            );
            continue;
        }

        fetch_commits_single_repo(pool.clone(), response)?;
    }

    Ok(())
}

/// Keeps track of the owner and repo for a given API response
pub struct ResponseInfo {
    pub request_url: String,
    pub owner: String,
    pub repo: String,
    pub status: u16,
    pub response: String,
    pub watermark: DateTime<Utc>,
}

/// Parse a repo string of the form `{owner}/{repo}` into a tuple of (owner, repo)
/// Returns an error if the string is not in the correct format
fn parse_repo_str(repo: &str) -> anyhow::Result<(&str, &str)> {
    let parts: Vec<&str> = repo.split('/').collect();
    if parts.len() != 2 {
        anyhow::bail!("Repo string must be of the form {{owner}}/{{repo}}");
    }
    Ok((parts[0], parts[1]))
}

/// Query the high watermark for the given request URL
/// Returns the watermark as a DateTime<Utc>
fn query_watermark(pool: Pool, request_url: &str) -> Result<DateTime<Utc>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        r#"
SELECT watermark
FROM wallowa_watermark
WHERE request_url = ?
LIMIT 1; -- There shouldn't be duplicate rows for the same `request_url`, but just in case.
"#,
    )?;
    let watermark = stmt.query_row(params![request_url], |row| {
        let watermark: DateTime<Utc> = row.get(0)?;
        Ok(watermark)
    })?;

    Ok(watermark)
}

// TODO make this configurable
const MAX_CONCURRENT_REQUESTS: usize = 3;

/// Request commits from the GitHub API
pub async fn request_commits(pool: Pool, repo_strings: &[String]) -> Result<Vec<ResponseInfo>> {
    let repos: Vec<(&str, &str)> = repo_strings
        .iter()
        .map(|s| parse_repo_str(s))
        .collect::<Result<Vec<_>, _>>()?;

    let requests = repos
        .iter()
        .map(|(owner, repo)| {
            let request_url = format!(
                "https://api.github.com/repos/{owner}/{repo}/commits",
                owner = owner,
                repo = repo
            );
            GithubRequest {
                url: request_url,
                owner: owner.to_string(),
                repo: repo.to_string(),
            }
        })
        .collect::<Vec<_>>();

    make_requests(pool, &requests).await
}

struct GithubRequest {
    owner: String,
    repo: String,
    url: String,
}

async fn make_requests(pool: Pool, requests: &[GithubRequest]) -> Result<Vec<ResponseInfo>> {
    let github_api_token: String = std::env::var("WALLOWA_GITHUB_AUTH_TOKEN")
        .expect("Missing WALLOWA_GITHUB_AUTH_TOKEN env var");

    let mut headers = HeaderMap::new();
    let mut header_value = HeaderValue::from_str(format!("Bearer {}", github_api_token).as_str())?;
    header_value.set_sensitive(true);
    headers.insert(AUTHORIZATION, header_value);

    let client = reqwest::ClientBuilder::new()
        .user_agent("reqwest/0.11.17")
        .default_headers(headers)
        .build()?;

    let responses = stream::iter(requests)
    .map(|request| {
        let url = &request.url;
        let client = &client;
        let old_watermark = match query_watermark(pool.clone(), url) {
            Ok(db_watermark) => {
                // Go back 5 minutes from the previous watermark to avoid missing commits
                db_watermark - chrono::Duration::minutes(5)
            },
            Err(e) => {
                info!("No previous watermark found for '{}'. Using Unix epoch time (1970-01-01 00:00:00 UTC). Message: {}", url, e);
                // This should never fail. If it does then default to about 10 years ago.
                match Utc.timestamp_opt(0, 0) {
                    LocalResult::Single(default_watermark) => default_watermark,
                    LocalResult::Ambiguous(default_watermark, _) => default_watermark,
                    LocalResult::None => {
                        debug!("Unexpected 'None' result from Utc.timestamp_opt(0, 0). Using 10 years ago as default watermark.");
                        Utc::now() - chrono::Duration::days(3652)
                    },
                }
            }
        };
        debug!("Using watermark: {} for url {}", old_watermark.to_rfc3339(), url);

        async move {
            let owner = &request.owner;
            let repo = &request.repo;
            let new_watermark = Utc::now();
            let mut inner_responses: Vec<ResponseInfo> = vec![];
            let base_url = url;
            let mut parsed_url = &mut Url::parse(base_url)?;
            parsed_url = parsed_url
                .query_pairs_mut()
                .append_pair("since", &old_watermark.to_rfc3339())
                // TODO make the per_page value configurable
                .append_pair("per_page", "100")
                .finish();
            let mut url_opt = Some(String::from(parsed_url.as_str()));

            // Loop to handle paginated responses
            while let Some(url) = url_opt {
                debug!("GET {}", url);
                let resp = client.get(url).send().await?;

                // Use the `Link` header from the Github API response in case of more pages of results
                url_opt = match resp.headers().get(LINK) {
                    Some(link_header) => {
                        let link_header_str = link_header.to_str()?;
                        let res = parse_link_header::parse_with_rel(link_header_str);
                        match res {
                            Ok(links) => {
                                links.get("next").map(|next_link| next_link.raw_uri.clone())
                            }
                            Err(e) => {
                                debug!("Error parsing link header: {}", e);
                                None
                            }
                        }
                    }
                    None => None,
                };

                let status = resp.status().as_u16();
                let text = resp.text().await?;
                inner_responses.push(ResponseInfo {
                    request_url: base_url.clone(), // use the base URL with no query parameters to match correctly in the wallowa_watermark table
                    owner: owner.to_string(),
                    repo: repo.to_string(),
                    status,
                    response: text,
                    watermark: new_watermark,
                });
            }
            Ok(inner_responses)
        }
    })
    .buffer_unordered(MAX_CONCURRENT_REQUESTS);

    // Wait for the streams to finish
    let computed = responses
        .collect::<Vec<Result<Vec<ResponseInfo>, anyhow::Error>>>()
        .await;

    // Not using `iter::collect()` here because we don't want to stop collecting the results if one of them fails
    let mut all_responses: Vec<ResponseInfo> = vec![];
    for inner_responses_res in computed.into_iter() {
        match inner_responses_res {
            Ok(inner_responses) => {
                all_responses.extend(inner_responses);
            }
            Err(e) => {
                error!("Error requesting commits: {}", e);
            }
        }
    }

    Ok(all_responses)
}

/// Request Pull Requests (PRs) from the GitHub API
pub async fn request_pulls(pool: Pool, repo_strings: &[String]) -> Result<Vec<ResponseInfo>> {
    let repos: Vec<(&str, &str)> = repo_strings
        .iter()
        .map(|s| parse_repo_str(s))
        .collect::<Result<Vec<_>, _>>()?;

    let requests = repos
        .iter()
        .map(|(owner, repo)| {
            let request_url = format!(
                "https://api.github.com/repos/{owner}/{repo}/pulls?state=all",
                owner = owner,
                repo = repo
            );
            GithubRequest {
                url: request_url,
                owner: owner.to_string(),
                repo: repo.to_string(),
            }
        })
        .collect::<Vec<_>>();

    make_requests(pool, &requests).await
}
