use crate::db::Pool;
use anyhow::Result;
use chrono::{DateTime, Utc};
use duckdb::params;
use serde::Serialize;
use tracing::debug;

pub struct CountByRepo {
    pub owner: String,
    pub repo: String,
    pub count: usize,
}

/// Count the number of commits by repo
/// TODO fix to query directly against raw data since the `github_commit` table no longer exists
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
/// TODO fix to query directly against raw data since the `github_commit` table no longer exists
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

#[derive(Serialize)]
pub struct DurationByDay {
    pub date: DateTime<Utc>,
    pub duration: Option<f64>,
}

pub fn merged_pr_duration_30_day_rolling_avg_hours(
    pool: Pool,
    owner: &str,
    repo: &str,
    end_date: DateTime<Utc>,
) -> Result<Vec<DurationByDay>> {
    debug!("Running `avg_merged_pr_duration`");

    let conn = pool.get()?;

    let mut stmt = conn.prepare(
        r#"
WITH calendar_day AS (
    -- Generate a series of days so that each day has a rolling average represented
    SELECT CAST(unnest(generate_series(CAST(? AS DATE) - interval 30 day, CAST(? AS DATE), interval '1' day)) AS DATE) as "day"
),
rolling AS (
    SELECT "owner", repo, CAST(created_at AS DATE) AS created_date, CAST(merged_at AS DATE) AS merged_date,
    AVG(EPOCH(AGE(merged_at, created_at)) / 86400) OVER thirty AS "PR duration in days, 30-day moving average"
    FROM github_pull
    WHERE "owner" = ?
    AND repo = ?
    AND merged_at NOT NULL
    WINDOW thirty AS (
        PARTITION BY owner, repo
        ORDER BY created_at ASC
        RANGE BETWEEN INTERVAL 30 DAYS PRECEDING
                AND INTERVAL 0 DAYS FOLLOWING)
)
SELECT calendar_day."day", AVG(rolling."PR duration in days, 30-day moving average")
FROM calendar_day
ASOF LEFT JOIN rolling ON calendar_day."day" >= rolling.merged_date
GROUP BY 1
ORDER BY 1;
"#)?;

    Ok(stmt
        .query_map(params![end_date, end_date, owner, repo], |row| {
            Ok(DurationByDay {
                date: row.get(0)?,
                duration: row.get(1)?,
            })
        })?
        .collect::<Result<Vec<DurationByDay>, duckdb::Error>>()?)
}
