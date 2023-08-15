use crate::{config_value, db::Pool};
use anyhow::{bail, Result};
use chrono::{DateTime, LocalResult, NaiveDateTime, TimeZone, Utc};
use duckdb::{params, OptionalExt};
use futures::future::join_all;
use reqwest::{
    header::{
        HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LINK,
    },
    StatusCode, Url,
};
use tracing::{debug, error, info};

/// Fetch pull requests
///
/// TODO test the fetching by adding PRs to the wallowa-old repo
/// TODO fix this comment
///
/// For each repo
///     Make HTTP request to see if there are any updates since last fetch (if that is available)
///         https://docs.github.com/en/rest/overview/resources-in-the-rest-api?apiVersion=2022-11-28#conditional-requests
///     Make HTTP request to list Pulls
///         - ordered by updatedDate with most recent update first
///     if ok response
///         save http body to rawdata
///         inspect the 'updated' time for the first and last PR in the returned list
///             if the watermark is within those dates (inclusive) then no more requests are needed
///                 store the latest etag as metadata
///             else make the same request again (for the next page)
///                 don't store the latest etag as metadata
pub async fn fetch_pulls_2(pool: &Pool) -> Result<()> {
    let github_api_token: String = config_value("github.auth.token").await?;
    let per_page: String = config_value("github.per_page").await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        ACCEPT,
        HeaderValue::from_str("application/vnd.github+json")?,
    );
    headers.insert("X-GitHub-Api-Version", HeaderValue::from_str("2022-11-28")?);
    let mut authz_value = HeaderValue::from_str(format!("Bearer {}", github_api_token).as_str())?;
    authz_value.set_sensitive(true);
    headers.insert(AUTHORIZATION, authz_value);

    let client = reqwest::ClientBuilder::new()
        .user_agent("wallowa/0.1.0")
        .default_headers(headers)
        .build()?;

    let owner = "gunrein";
    let repo = "wallowa-old";

    // TODO add page? and per_page safely
    let request_url = format!(
        "https://api.github.com/repos/{owner}/{repo}/pulls?state=all&sort=updated&direction=desc",
        owner = owner,
        repo = repo
    );

    let mut conn = pool.get()?;

    // Select the most recent updated_at date and etag from raw_data
    let mut watermark_query = conn.prepare(
        r#"
WITH raw AS (
    SELECT
        metadata->>'$.etag' AS etag,
        metadata->>'$.owner' AS "owner",
        metadata->>'$.repo' AS repo,
        unnest(json_transform_strict("data",
            '[{
                "updated_at": "TIMESTAMP",
            }]')) AS row,
    FROM wallowa_raw_data
    WHERE "data_source" = 'github_rest_api'
    AND data_type = 'pulls'
    AND "owner" = ?
    AND repo = ?
    ORDER BY created_at DESC
)
SELECT etag, row.updated_at AS updated_at
FROM raw
ORDER BY updated_at DESC
LIMIT 1
"#,
    )?;

    let mut req_builder = client.get(request_url);

    let watermark = watermark_query
        .query_row(params![owner, repo], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, DateTime<Utc>>(1)?))
        })
        .optional()?;
    if let Some((etag, modified_since)) = watermark {
        if !etag.is_empty() {
            req_builder = req_builder.header(IF_NONE_MATCH, etag);
        }
        req_builder = req_builder.header(IF_MODIFIED_SINCE, modified_since.to_string());
    }
    debug!("Request for Github Pulls: {:?}", req_builder);

    let resp = req_builder.send().await?;

    let resp_status = resp.status().clone();
    let resp_headers = resp.headers().clone();
    let latest_etag = if let Some(response_etag) = resp_headers.get(ETAG) {
        response_etag.to_str()?
    } else {
        ""
    };

    let text = resp.text().await?;

    debug!("Response status code and etag from Github Pulls: {:?}, {:?}", resp_status, latest_etag);
    if resp_status != StatusCode::OK {
        // A 304 or error, no need to further process the response
        return Ok(());
    }

    // 200 response code; process the response
    // Write to the database in a transaction
    // Query the new row to see if there is any new data (compare the newest row with the watermark, or maybe do away with watermark by comparing with existing PR rows/dates)
    // if there new data, ommit the transaction
    // if no new data, rollback the transaction since there is no need to store the redundant data

    // Insert the raw JSON into the database
    let tx = conn.transaction()?;

    let mut insert_stmt = tx.prepare(
        r#"
INSERT INTO wallowa_raw_data (
    "data_source",
    data_type,
    metadata,
    "data"
) VALUES (
    'github_rest_api',
    'pulls',
    to_json({owner: ?, repo: ?, etag: ?}),
    ?
)
RETURNING id
"#,
    )?;
    let row_id = insert_stmt.query_row(params![owner, repo, latest_etag, text], |row| {
        row.get::<_, i64>(0)
    })?;

    let mut query_stmt = tx.prepare(
        r#"
-- Figure out if the newly inserted JSON object has any new updates in it.
-- This approach doesn't require storing state external to the `wallowa_raw_data` table
-- for tracking the latest data but causes increased query cost when fetching/inserting new data.
--
-- Extract the necessary fields from the raw JSON structures into `raw`
WITH raw AS (
    SELECT
        id,
        metadata->>'$.owner' AS "owner",
        metadata->>'$.repo' AS repo,
        unnest(json_transform_strict("data",
            '[{
                "updated_at": "TIMESTAMP",
                "base": {
                    "repo": {
                        "name": "VARCHAR",
                        "owner": {
                            "login": "VARCHAR"
                        }
                    }
                },
            }]')) AS row,
    FROM wallowa_raw_data
    WHERE "data_source" = 'github_rest_api'
    AND data_type = 'pulls'
    AND "owner" = ?
    AND repo = ?
    ORDER BY created_at DESC
),
-- Calculate the maximum (most recent) updated date for the repo
max_date AS (
    SELECT MAX(row.updated_at) AS latest
    FROM raw
),
-- Select the set of raw data JSON rows with the maximum (latest) updated date for the repo
rows_with_max_date AS (
    SELECT raw.id
    FROM raw, max_date
    WHERE raw.row.updated_at = max_date.latest
)
-- Return true if there is only 1 row with the maximum date and that row has the ID of the newly
-- inserted JSON object. True indicates that this new JSON object should be kept.
SELECT (? IN (SELECT id FROM rows_with_max_date)) AND (SELECT count(id) = 1 FROM rows_with_max_date)
"#,
    )?;

    let save_new_data =
        query_stmt.query_row(params![owner, repo, row_id], |row| row.get::<_, bool>(0))?;

    if save_new_data {
        tx.commit()?;
    } else {
        tx.rollback()?;
    }

    Ok(())
}

/// Fetch the latest data from Github
pub async fn fetch_all(pool: &Pool) -> Result<NaiveDateTime> {
    let repos: Vec<String> = config_value("github.repos").await?;
    info!("Fetching from GitHub");
    let responses = request_pulls(pool, &repos).await?;
    let result = fetch_pulls(pool, &responses);
    info!("Fetching from GitHub complete");
    result
}

// TODO update Pulls fetch logic to:
// - sort by updated, in order from newest to oldest
// - for each payload, check whether the watermark has been reached (with the update date for first and last Pull in the payload)
// - stop requesting more pages when a payload is reached that was updated before the watermark timestamp
// https://docs.github.com/en/rest/pulls/pulls?apiVersion=2022-11-28#list-pull-requests

/// Return the timestamp of the most recent API request
pub fn latest_fetch(pool: &Pool) -> Result<NaiveDateTime> {
    let conn = pool.get()?;

    let sql = r#"
SELECT watermark
FROM wallowa_watermark
WHERE prefix(request_url,'https://api.github.com/')
ORDER BY watermark DESC LIMIT 1
"#;
    let watermark: Result<NaiveDateTime, duckdb::Error> =
        match conn.query_row(sql, [], |row| row.get(0)) {
            Ok(w) => Ok(w),
            Err(duckdb::Error::QueryReturnedNoRows) => {
                Ok(NaiveDateTime::from_timestamp_opt(0, 0).unwrap())
            } // Return a default if there are no existing rows
            Err(e) => bail!(e), // Unexpected error
        };

    Ok(watermark?)
}

/// Insert raw pulls data from a single Github API response
fn insert_pulls_single_repo(pool: &Pool, response: &ResponseInfo) -> Result<()> {
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

/// Request the latest pulls from the given GitHub API responses and insert them into the database
pub fn fetch_pulls(pool: &Pool, responses: &Vec<ResponseInfo>) -> Result<NaiveDateTime> {
    for response in responses {
        if response.status != 200 {
            error!(
                "Skipping response for '{}' with status {}",
                response.request_url, response.status
            );
            continue;
        }

        insert_pulls_single_repo(pool, response)?;
    }

    latest_fetch(pool)
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

/// Request commits from the GitHub API
pub async fn request_commits(pool: &Pool, repo_strings: &[String]) -> Result<Vec<ResponseInfo>> {
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

async fn make_requests(pool: &Pool, requests: &[GithubRequest]) -> Result<Vec<ResponseInfo>> {
    let github_api_token: String = config_value("github.auth.token").await?;

    let mut headers = HeaderMap::new();
    let mut header_value = HeaderValue::from_str(format!("Bearer {}", github_api_token).as_str())?;
    header_value.set_sensitive(true);
    headers.insert(AUTHORIZATION, header_value);

    let client = reqwest::ClientBuilder::new()
        .user_agent("reqwest/0.11.17")
        .default_headers(headers)
        .build()?;

    let request_futs = requests.iter()
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
            let per_page: String = config_value("github.per_page").await?;
            parsed_url = parsed_url
                .query_pairs_mut()
                .append_pair("since", &old_watermark.to_rfc3339())
                .append_pair("per_page", per_page.as_str())
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
    });
    let computed: Vec<Result<Vec<ResponseInfo>, anyhow::Error>> = join_all(request_futs).await;

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
pub async fn request_pulls(pool: &Pool, repo_strings: &[String]) -> Result<Vec<ResponseInfo>> {
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
