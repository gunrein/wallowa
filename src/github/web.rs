use std::{io::BufWriter, sync::Arc};

use axum::{
    body::Body,
    extract::State,
    response::Html,
    routing::{get, post},
    Router,
};
use chrono::{TimeZone, Utc};
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

async fn merged_pr_duration_30_day_rolling_avg_hours_arrow(
    State(state): State<Arc<AppState>>,
) -> AppResult<Vec<u8>> {
    let end_date = Utc.with_ymd_and_hms(2023, 6, 16, 0, 0, 0).unwrap();
    let results = merged_pr_duration_30_day_rolling_avg_hours(&state.pool, end_date)?;

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
