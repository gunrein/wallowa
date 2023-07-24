use crate::db::Pool;
use anyhow::Result;
use arrow::record_batch::RecordBatch;
use chrono::{DateTime, FixedOffset};
use duckdb::params;
use tracing::debug;

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

pub fn merged_pr_duration_rolling_daily_average(
    pool: &Pool,
    start_date: DateTime<FixedOffset>,
    end_date: DateTime<FixedOffset>,
) -> Result<Vec<RecordBatch>> {
    debug!("Running `merged_pr_duration_rolling_daily_average`");

    let conn = pool.get()?;

    let mut stmt = conn.prepare(
        r#"
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
            '[{
                "url": "VARCHAR",
                "base": {
                    "repo": {
                        "name": "VARCHAR",
                        "owner": {
                            "login": "VARCHAR"
                        }
                    }
                },
                "state": "VARCHAR",
                "created_at": "TIMESTAMP",
                "closed_at": "TIMESTAMP",
                "merged_at": "TIMESTAMP",
                "draft": "BOOLEAN"
            }]')) AS row,
    FROM wallowa_raw_data
    WHERE "data_source" = 'github_rest_api'
    AND data_type = 'pulls'
),
rolling AS (
    SELECT DISTINCT ON (row.url)
        row.base.repo.owner.login AS "owner",
        row.base.repo.name AS repo,
        CAST(row.created_at AS DATE) AS created_date,
        CAST(row.merged_at AS DATE) AS merged_date,
        AVG(EPOCH(AGE(row.merged_at, row.created_at)) / 86400) OVER thirty AS duration
    FROM pulls
    WHERE row.merged_at NOT NULL
    WINDOW thirty AS (
        PARTITION BY "owner", repo
        ORDER BY row.created_at ASC
        RANGE BETWEEN INTERVAL 30 DAYS PRECEDING
                AND INTERVAL 0 DAYS FOLLOWING)
)
SELECT calendar_day."day" AS "day", ("owner" || '/' || repo) AS repo, AVG(rolling.duration) AS "duration"
FROM calendar_day
ASOF LEFT JOIN rolling ON calendar_day."day" >= rolling.merged_date
GROUP BY 1,2
ORDER BY 1,2
"#)?;

    Ok(stmt
        .query_arrow(params![start_date.naive_utc(), end_date.naive_utc()])?
        .collect::<Vec<RecordBatch>>())
}
