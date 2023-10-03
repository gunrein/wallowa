use anyhow::Result;
use arrow::record_batch::RecordBatch;
use chrono::{DateTime, FixedOffset};
use tracing::{debug, error};
use wallowa_duckdb::duckdb::{params_from_iter, ToSql};
use wallowa_duckdb::Pool;

/// Get the list of distinct GitHub repository names in the database.
/// Repository names consist of `owner/repo`.
pub fn select_distinct_repos(pool: &Pool) -> Result<Vec<String>> {
    let conn = pool.get()?;

    let mut stmt = conn.prepare(
        r#"
WITH pulls AS (
    SELECT
        unnest(json_transform_strict("data",
            '[{
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
)
SELECT DISTINCT (row.base.repo.owner.login || '/' || row.base.repo.name) AS repo
FROM pulls
"#,
    )?;
    let rows = stmt.query_map([], |row| row.get(0))?;
    let mut repo_names = vec![];
    for row in rows {
        match row {
            Ok(repo_name) => repo_names.push(repo_name),
            Err(e) => error!("Error querying distinct repos: {:?}", e),
        }
    }
    Ok(repo_names)
}

/// Query the rolling daily average time to merge GitHub Pull Requests
pub fn merged_pr_duration_rolling_daily_average(
    pool: &Pool,
    start_date: DateTime<FixedOffset>,
    end_date: DateTime<FixedOffset>,
    repos: &Vec<String>,
) -> Result<Vec<RecordBatch>> {
    debug!("Running `merged_pr_duration_rolling_daily_average`");

    let conn = pool.get()?;

    let repo_placeholders = if repos.is_empty() {
        "SELECT DISTINCT (row.base.repo.owner.login || '/' || row.base.repo.name) AS repo FROM pulls".to_string()
    } else {
        let mut placeholders = "?,".repeat(repos.len());
        placeholders.pop(); // Remove the trailing comma (`,`)
        format!("SELECT unnest([{}]) AS repo", placeholders)
    };
    debug!("repo_placeholders: {:?} for {:?}", repo_placeholders, repos);

    let mut stmt = conn.prepare(&format!(r#"
-- merged_pr_duration_rolling_daily_average
-- Duration of merged GitHub Pull Requests, rolling daily average
WITH calendar_day AS (
    -- Generate a series of days so that each day has a rolling average represented
    SELECT CAST(unnest(generate_series(CAST(? AS TIMESTAMP), CAST(? AS TIMESTAMP), interval '1' day)) AS DATE) as "day"
),
pulls AS (
    SELECT
        id,
        "data_source",
        unnest(json_transform_strict("data",
            '[{{
                "url": "VARCHAR",
                "base": {{
                    "repo": {{
                        "name": "VARCHAR",
                        "owner": {{
                            "login": "VARCHAR"
                        }}
                    }}
                }},
                "state": "VARCHAR",
                "created_at": "TIMESTAMP",
                "closed_at": "TIMESTAMP",
                "merged_at": "TIMESTAMP",
                "updated_at": "TIMESTAMP",
                "draft": "BOOLEAN"
            }}]')) AS row,
    FROM wallowa_raw_data
    WHERE "data_source" = 'github_rest_api'
    AND data_type = 'pulls'
),
repos AS (
    {repo_placeholders}
),
calendar_day_repos AS (
    -- Generate a series of days for each repo so that each day+repo has a rolling average represented
    SELECT calendar_day."day", repos.repo FROM calendar_day CROSS JOIN repos
),
latest_deduped_pulls AS (
    SELECT
        row.url AS "url",
        (row.base.repo.owner.login || '/' || row.base.repo.name) AS repo,
        row.created_at AS created_at,
        row.merged_at AS merged_at,
        row.updated_at AS updated_at,
        row_number() OVER (PARTITION BY "url" ORDER BY updated_at DESC) AS row_number
    FROM pulls
    WHERE repo IN (SELECT repo FROM repos)
),
rolling AS (
    SELECT
        repo,
        CAST(created_at AS DATE) AS created_date,
        CAST(merged_at AS DATE) AS merged_date,
        AVG(EPOCH(AGE(merged_at, created_at)) / 86400) OVER thirty AS duration
    FROM latest_deduped_pulls
    WHERE row_number = 1
    AND merged_at NOT NULL
    WINDOW thirty AS (
        PARTITION BY repo
        ORDER BY created_at ASC
        RANGE BETWEEN INTERVAL 30 DAYS PRECEDING
                AND INTERVAL 0 DAYS FOLLOWING)
)
SELECT calendar_day_repos."day" AS "day", rolling.repo, AVG(rolling.duration) AS "duration"
FROM calendar_day_repos ASOF LEFT JOIN rolling ON (calendar_day_repos.repo = rolling.repo AND calendar_day_repos."day" >= rolling.merged_date)
GROUP BY 1,2
ORDER BY 1,2
"#, repo_placeholders = repo_placeholders))?;

    let mut params = Vec::new();
    let start_date_naive = start_date.naive_utc();
    let end_date_naive = end_date.naive_utc();
    params.push(start_date_naive.to_sql()?);
    params.push(end_date_naive.to_sql()?);
    for repo in repos {
        params.push(repo.to_sql()?);
    }

    let rows = stmt.query_arrow(params_from_iter(params))?;
    let mut batches = Vec::new();
    for row in rows {
        batches.push(row);
    }
    Ok(batches)
}

/// Query the closed GitHub Pull Requests
pub fn closed_prs(
    pool: &Pool,
    start_date: DateTime<FixedOffset>,
    end_date: DateTime<FixedOffset>,
    repos: &Vec<String>,
) -> Result<Vec<RecordBatch>> {
    debug!("Running `closed_prs`");

    let conn = pool.get()?;

    let repo_placeholders = if repos.is_empty() {
        "SELECT DISTINCT (row.base.repo.owner.login || '/' || row.base.repo.name) AS repo FROM pulls".to_string()
    } else {
        let mut placeholders = "?,".repeat(repos.len());
        placeholders.pop(); // Remove the trailing comma (`,`)
        format!("SELECT unnest([{}]) AS repo", placeholders)
    };
    debug!("repo_placeholders: {:?} for {:?}", repo_placeholders, repos);

    let mut stmt = conn.prepare(&format!(
        r#"
WITH pulls AS (
    SELECT
        id,
        "data_source",
        unnest(json_transform_strict("data",
            '[{{
                "url": "VARCHAR",
                "base": {{
                    "repo": {{
                        "name": "VARCHAR",
                        "owner": {{
                            "login": "VARCHAR"
                        }}
                    }}
                }},
                "state": "VARCHAR",
                "created_at": "TIMESTAMP",
                "closed_at": "TIMESTAMP",
                "merged_at": "TIMESTAMP",
                "updated_at": "TIMESTAMP",
                "draft": "BOOLEAN"
            }}]')) AS row,
    FROM wallowa_raw_data
    WHERE "data_source" = 'github_rest_api'
    AND data_type = 'pulls'
),
repos AS (
    {repo_placeholders}
),
latest_deduped_pulls_window AS (
    SELECT
        row.url AS "url",
        (row.base.repo.owner.login || '/' || row.base.repo.name) AS repo,
        row.created_at AS created_at,
        row.merged_at AS merged_at,
        row.updated_at AS updated_at,
        row.closed_at AS closed_at,
        row_number() OVER (PARTITION BY "url" ORDER BY updated_at DESC) AS row_number
    FROM pulls
    WHERE repo IN (SELECT repo FROM repos)
)
SELECT
    "url",
    repo,
    created_at,
    merged_at,
    updated_at,
    CAST(latest_deduped_pulls_window.closed_at AS DATE) AS closed_at
FROM latest_deduped_pulls_window
WHERE row_number = 1
AND closed_at >= ?
AND closed_at <= ?
"#,
        repo_placeholders = repo_placeholders
    ))?;

    let mut params = Vec::new();
    let start_date_naive = start_date.naive_utc();
    let end_date_naive = end_date.naive_utc();
    for repo in repos {
        params.push(repo.to_sql()?);
    }
    params.push(start_date_naive.to_sql()?);
    params.push(end_date_naive.to_sql()?);

    let rows = stmt.query_arrow(params_from_iter(params))?;
    let mut batches = Vec::new();
    for row in rows {
        batches.push(row);
    }
    Ok(batches)
}
