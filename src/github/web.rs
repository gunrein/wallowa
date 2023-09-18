use std::{io::BufWriter, sync::Arc};

use axum::{
    body::Body,
    extract::State,
    response::Html,
    routing::{get, post},
    Router,
};
use axum_extra::extract::Query;
use chrono::{DateTime, Datelike, Days, FixedOffset, TimeZone, Utc};
use duckdb::arrow::{datatypes::Schema, ipc::writer::FileWriter};
use minijinja::context;
use serde::Deserialize;
use tracing::error;

use crate::{
    web::{render, AppState},
    AppResult,
};

use super::{
    fetch::fetch_all,
    queries::{merged_pr_duration_rolling_daily_average, select_distinct_repos, closed_prs},
};

/// All page-related routes for GitHub
pub fn page_routes() -> Router<Arc<AppState>, Body> {
    Router::new()
        .route("/pr_duration", get(github_pr_duration))
        .route("/closed_pr_count", get(github_closed_pr_count))
        .route("/", get(github_dashboard))
        .route("/fetch", post(fetch_source))
}

/// All data-related routes for GitHub
pub fn data_routes() -> Router<Arc<AppState>, Body> {
    Router::new()
        .route(
            "/merged_pr_duration_rolling_daily_average.arrow",
            get(merged_pr_duration_rolling_daily_average_arrow),
        )
        .route("/closed_prs.arrow", get(closed_prs_arrow))
}

async fn fetch_source(State(state): State<Arc<AppState>>) -> AppResult<Html<String>> {
    let result = fetch_all(&state.pool).await;

    let message = match result {
        Ok(timestamp) => timestamp.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        Err(err) => {
            let msg = format!("{err}");
            error!(msg);
            msg
        },
    };
    Ok(Html(render(
        state,
        "sources/fetch_source.html",
        context! {
            message,
        },
    )?))
}

#[derive(Deserialize, Debug)]
struct MergedPRParams {
    start_date: Option<DateTime<FixedOffset>>,
    end_date: Option<DateTime<FixedOffset>>,
    #[serde(default)]
    repo: Vec<String>,
}

async fn merged_pr_duration_rolling_daily_average_arrow(
    State(state): State<Arc<AppState>>,
    Query(params): Query<MergedPRParams>,
) -> AppResult<Vec<u8>> {
    // TODO better error handling for invalid or missing parameters
    let end_date = if let Some(end) = params.end_date {
        end
    } else {
        let now = chrono::offset::Utc::now();
        let beginning_of_today = Utc
            .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
            .unwrap();
        beginning_of_today.fixed_offset()
    };
    let start_date = if let Some(start) = params.start_date {
        start
    } else {
        end_date.checked_sub_days(Days::new(30)).unwrap()
    };

    let results =
        merged_pr_duration_rolling_daily_average(&state.pool, start_date, end_date, &params.repo)?;

    let mut ipc_data: Vec<u8> = Vec::new();
    if !results.is_empty() {
        // Use the schema from the first RecordBatch as the IPC schema
        let schema = results[0].schema();
        let metadata = schema.metadata.clone();
        let fields: Vec<Arc<duckdb::arrow::datatypes::Field>> = schema
            .all_fields()
            .iter()
            .map(|field| Arc::new((*field).clone()))
            .collect();
        let ipc_schema = Schema::new_with_metadata(fields, metadata);

        let buf = BufWriter::new(&mut ipc_data);
        let mut writer = FileWriter::try_new(buf, &ipc_schema)?;
        for batch in results {
            writer.write(&batch)?;
        }
        writer.finish()?;
    }

    Ok(ipc_data)
}

async fn github_pr_duration(State(state): State<Arc<AppState>>) -> AppResult<Html<String>> {
    let distinct_repos = select_distinct_repos(&state.pool)?;
    let html = render(
        state,
        "github/pr_duration.html",
        context! {
            current_nav => "/github/pr_duration",
            repos => distinct_repos,
        },
    )?;
    Ok(Html(html))
}

async fn closed_prs_arrow(
    State(state): State<Arc<AppState>>,
    Query(params): Query<MergedPRParams>,
) -> AppResult<Vec<u8>> {
    // TODO better error handling for invalid or missing parameters
    let end_date = if let Some(end) = params.end_date {
        end
    } else {
        let now = chrono::offset::Utc::now();
        let beginning_of_today = Utc
            .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
            .unwrap();
        beginning_of_today.fixed_offset()
    };
    let start_date = if let Some(start) = params.start_date {
        start
    } else {
        end_date.checked_sub_days(Days::new(30)).unwrap()
    };

    let results = closed_prs(&state.pool, start_date, end_date, &params.repo)?;

    let mut ipc_data: Vec<u8> = Vec::new();
    if !results.is_empty() {
        // Use the schema from the first RecordBatch as the IPC schema
        let schema = results[0].schema();
        let metadata = schema.metadata.clone();
        let fields: Vec<Arc<duckdb::arrow::datatypes::Field>> = schema
            .all_fields()
            .iter()
            .map(|field| Arc::new((*field).clone()))
            .collect();
        let ipc_schema = Schema::new_with_metadata(fields, metadata);

        let buf = BufWriter::new(&mut ipc_data);
        let mut writer = FileWriter::try_new(buf, &ipc_schema)?;
        for batch in results {
            writer.write(&batch)?;
        }
        writer.finish()?;
    }

    Ok(ipc_data)
}

async fn github_closed_pr_count(State(state): State<Arc<AppState>>) -> AppResult<Html<String>> {
    let distinct_repos = select_distinct_repos(&state.pool)?;
    let html = render(
        state,
        "github/pr_count.html",
        context! {
            current_nav => "/github/closed_pr_count",
            repos => distinct_repos,
        },
    )?;
    Ok(Html(html))
}

async fn github_dashboard(State(state): State<Arc<AppState>>) -> AppResult<Html<String>> {
    let html = render(
        state,
        "github/index.html",
        context! {
            current_nav => "/github",
        },
    )?;
    Ok(Html(html))
}
