use crate::db::Pool;
use anyhow::Result;
use arrow::record_batch::RecordBatch;
use chrono::{DateTime, FixedOffset};
use duckdb::{params_from_iter, ToSql};
use tracing::{debug, error};

pub struct CountByRepo {
    pub owner: String,
    pub repo: String,
    pub count: usize,
}

/// Count the number of commits by repo
/// TODO fix to query directly against raw data since the `github_commit` table no longer exists (BE CAREFUL TO FILTER DUPLICATES!)
pub fn count_commits(pool: Pool) -> Result<Vec<CountByRepo>> {
    debug!("Running `count_commits`");

    let conn = pool.get()?;

    let mut stmt = conn.prepare(
        r#"
SELECT "owner", repo, count(*)
FROM github_commit
GROUP BY "owner", repo
ORDER BY "owner", repo;
"#,
    )?;

    Ok(stmt
        .query_map([], |row| {
            Ok(CountByRepo {
                owner: row.get(0)?,
                repo: row.get(1)?,
                count: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<CountByRepo>, duckdb::Error>>()?)
}

/// Count the number of pulls by repo
/// TODO fix to query directly against raw data since the `github_pull` table no longer exists (BE CAREFUL TO FILTER DUPLICATES!)
pub fn count_pulls(pool: Pool) -> Result<Vec<CountByRepo>> {
    debug!("Running `count_pulls`");

    let conn = pool.get()?;

    let mut stmt = conn.prepare(
        r#"
SELECT "owner", repo, count(*)
FROM github_pull
GROUP BY "owner", repo
ORDER BY "owner", repo;
"#,
    )?;

    Ok(stmt
        .query_map([], |row| {
            Ok(CountByRepo {
                owner: row.get(0)?,
                repo: row.get(1)?,
                count: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<CountByRepo>, duckdb::Error>>()?)
}

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

pub fn merged_pr_duration_rolling_daily_average(
    pool: &Pool,
    start_date: DateTime<FixedOffset>,
    end_date: DateTime<FixedOffset>,
    repos: &Vec<String>,
) -> Result<Vec<RecordBatch>> {
    debug!("Running `merged_pr_duration_rolling_daily_average`");

    let conn = pool.get()?;

    let repo_placeholders = if repos.is_empty() {
        "".to_string()
    } else {
        let mut placeholders = "?,".repeat(repos.len());
        placeholders.pop(); // Remove the trailing comma (`,`)
        format!("AND repo IN ({})", placeholders)
    };

    let mut stmt = conn.prepare(
        &format!(r#"
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
                "draft": "BOOLEAN"
            }}]')) AS row,
    FROM wallowa_raw_data
    WHERE "data_source" = 'github_rest_api'
    AND data_type = 'pulls'
),
rolling AS (
    SELECT DISTINCT ON (row.url)
        (row.base.repo.owner.login || '/' || row.base.repo.name) AS repo,
        CAST(row.created_at AS DATE) AS created_date,
        CAST(row.merged_at AS DATE) AS merged_date,
        AVG(EPOCH(AGE(row.merged_at, row.created_at)) / 86400) OVER thirty AS duration
    FROM pulls
    WHERE row.merged_at NOT NULL
    {repo_placeholders}
    WINDOW thirty AS (
        PARTITION BY repo
        ORDER BY row.created_at ASC
        RANGE BETWEEN INTERVAL 30 DAYS PRECEDING
                AND INTERVAL 0 DAYS FOLLOWING)
)
SELECT calendar_day."day" AS "day", rolling.repo, AVG(rolling.duration) AS "duration"
FROM calendar_day
ASOF LEFT JOIN rolling ON calendar_day."day" >= rolling.merged_date
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
