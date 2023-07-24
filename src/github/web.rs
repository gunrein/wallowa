use std::{collections::HashMap, io::BufWriter, sync::Arc};

use axum::{
    body::Body,
    extract::{Query, State},
    response::Html,
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Datelike, FixedOffset, TimeZone, Utc};
use duckdb::arrow::{datatypes::Schema, ipc::writer::FileWriter};
use minijinja::context;

use crate::{
    web::{render, AppState},
    AppResult,
};

use super::{fetch::fetch_all, queries::merged_pr_duration_30_day_rolling_avg_hours};

/// All page-related routes for GitHub
pub fn page_routes() -> Router<Arc<AppState>, Body> {
    Router::new()
        .route("/pr_duration", get(github_pr_duration))
        .route("/", get(github_dashboard))
        .route("/fetch", post(fetch_source))
}

/// All data-related routes for GitHub
pub fn data_routes() -> Router<Arc<AppState>, Body> {
    Router::new().route(
        "/merged_pr_duration_30_day_rolling_avg_hours.arrow",
        get(merged_pr_duration_30_day_rolling_avg_hours_arrow),
    )
}

async fn fetch_source(State(state): State<Arc<AppState>>) -> AppResult<Html<String>> {
    let timestamp = fetch_all(&state.pool).await?;

    Ok(Html(render(
        state,
        "sources/fetch_source.html",
        context! {
            timestamp => timestamp.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        },
    )?))
}

fn parse_date_param(date_param: Option<&String>) -> AppResult<DateTime<FixedOffset>> {
    let date = if let Some(date_str) = date_param {
        DateTime::parse_from_rfc3339(date_str)?
    } else {
        let now = chrono::offset::Utc::now();
        let beginning_of_today = Utc
            .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
            .unwrap();
        beginning_of_today.fixed_offset()
    };

    Ok(date)
}

async fn merged_pr_duration_30_day_rolling_avg_hours_arrow(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> AppResult<Vec<u8>> {
    let start_date = parse_date_param(params.get("start_date"))?;
    let end_date = parse_date_param(params.get("end_date"))?;

    // TODO better error handling for invalid or missing parameters

    let results = merged_pr_duration_30_day_rolling_avg_hours(&state.pool, start_date, end_date)?;

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
    let html = render(
        state,
        "github/pr_duration.html",
        context! {
            current_nav => "/github/pr_duration",
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
