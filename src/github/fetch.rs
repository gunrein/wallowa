use crate::{config_value, db::Pool};
use anyhow::{bail, Result};
use chrono::{DateTime, LocalResult, NaiveDateTime, TimeZone, Utc};
use duckdb::{params, OptionalExt};
use reqwest::{
    header::{
        HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LINK,
    },
    StatusCode,
};
use tracing::{debug, info};

/// Fetch pull requests for a specific owner+repo
pub async fn fetch_pulls(pool: &Pool, owner: &str, repo: &str) -> Result<()> {
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

    let mut url_opt = Some(format!(
        "https://api.github.com/repos/{owner}/{repo}/pulls?state=all&sort=updated&direction=desc&per_page={per_page}",
        owner = owner,
        repo = repo,
        per_page = per_page,
    ));

    let mut conn = pool.get()?;

    // Select the most recent updated_at date and etag from raw_data
    let watermark = conn.query_row(
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
params![owner, repo], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, DateTime<Utc>>(1)?))
        })
        .optional()?;
    let (etag, modified_since) = if let Some((inner_etag, inner_modified_since)) = watermark.clone()
    {
        (inner_etag, inner_modified_since)
    } else {
        (
            "".to_string(),
            // This should never fail. If it does then default to about 10 years ago.
            match Utc.timestamp_opt(0, 0) {
                LocalResult::Single(default_watermark) => default_watermark,
                LocalResult::Ambiguous(default_watermark, _) => default_watermark,
                LocalResult::None => {
                    debug!("Unexpected 'None' result from Utc.timestamp_opt(0, 0). Using 10 years ago as default watermark.");
                    Utc::now() - chrono::Duration::days(3652)
                }
            },
        )
    };

    while let Some(request_url) = url_opt {
        let mut req_builder = client.get(request_url);
        if watermark.is_some() {
            if !etag.clone().is_empty() {
                req_builder = req_builder.header(IF_NONE_MATCH, etag.clone());
            }
            req_builder = req_builder.header(IF_MODIFIED_SINCE, modified_since.to_string());
        }

        debug!("Request for Github Pulls: {:?}", req_builder);

        let resp = req_builder.send().await?;

        let resp_status = resp.status();
        let resp_headers = resp.headers().clone();
        let latest_etag = if let Some(response_etag) = resp_headers.get(ETAG) {
            response_etag.to_str()?
        } else {
            ""
        };

        let text = resp.text().await?;

        debug!(
            "Response status code and etag from Github Pulls: {:?}, {:?}",
            resp_status, latest_etag
        );
        if resp_status != StatusCode::OK {
            // A 304 or error, no need to further process the response
            return Ok(());
        }

        // 200 response code; process the response
        // The data is inserted into the database to check whether any new data is in the response.
        // If new data is found, it is committed to the database.
        // If no new data is found, the insert is rolled back and the function completes.

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
)
SELECT COUNT(id) > 0
FROM raw
WHERE raw.row.updated_at >= ?
AND id = ?
"#,
        )?;

        let save_new_data = query_stmt
            .query_row(params![owner, repo, modified_since, row_id], |row| {
                row.get::<_, bool>(0)
            })?;

        if save_new_data {
            debug!("New data found for Github Pulls; committing");
            tx.commit()?;

            // Check for a `next` header in case of another page of results, but only when the
            // current page of results has new data
            url_opt = match resp_headers.get(LINK) {
                Some(link_header) => {
                    let link_header_str = link_header.to_str()?;
                    let res = parse_link_header::parse_with_rel(link_header_str);
                    match res {
                        Ok(links) => links.get("next").map(|next_link| next_link.raw_uri.clone()),
                        Err(e) => {
                            debug!("Error parsing link header: {}", e);
                            None
                        }
                    }
                }
                None => None,
            };
        } else {
            debug!("No new data found for Github Pulls; rolling back");
            tx.rollback()?;

            // No need to fetch more pages since the latest data isn't new
            url_opt = None;
        }
    }

    Ok(())
}

/// Fetch the latest data from Github
pub async fn fetch_all(pool: &Pool) -> Result<()> {
    let repos: Vec<String> = config_value("github.repos").await?;
    info!("Fetching from GitHub");
    for repo_string in repos {
        let (owner, repo_name) = parse_repo_str(&repo_string)?;
        fetch_pulls(pool, owner, repo_name).await?;
    }

    // TODO decide whether to work through the compiler error in order to add concurrency to these requests
    // TODO make this configurable
    /*
    let github_request_max_concurrency = 2;
    let repo_requests = repos.iter().map(|repo| {
        let (owner, repo_name) = parse_repo_str(repo).expect(format!("Malformed repository string {}", repo).as_str());
        fetch_pulls_2(pool, owner, repo_name)
    });
    let mut executed_requests =
        futures::stream::iter(repo_requests)
            .buffer_unordered(github_request_max_concurrency);
    while executed_requests.next().await.is_some() {}
    */
    info!("Fetching from GitHub complete");
    Ok(())
}

/// Return the timestamp of the most recent API request
/// TODO fix this for Github Pulls
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

/// Parse a repo string of the form `{owner}/{repo}` into a tuple of (owner, repo)
/// Returns an error if the string is not in the correct format
fn parse_repo_str(repo: &str) -> anyhow::Result<(&str, &str)> {
    let parts: Vec<&str> = repo.split('/').collect();
    if parts.len() != 2 {
        anyhow::bail!("Repo string must be of the form {{owner}}/{{repo}}");
    }
    Ok((parts[0], parts[1]))
}
