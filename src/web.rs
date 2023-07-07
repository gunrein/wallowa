use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use chrono::{TimeZone, Utc};
use duckdb::arrow::{datatypes::Schema, ipc::writer::FileWriter};
use minijinja::{context, Environment, Source};
use minijinja_autoreload::AutoReloader;
use std::{io::BufWriter, net::SocketAddr, sync::Arc};
use tokio::signal;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::{debug, info};

use crate::{
    db::Pool,
    queries::github::merged_pr_duration_30_day_rolling_avg_hours,
    sources::{fetch_given_source, github::latest_fetch},
    AppError, AppResult,
};

pub async fn handler_merged_pr_duration_30_day_rolling_avg_hours(
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

pub async fn github_pr_duration(State(state): State<Arc<AppState>>) -> AppResult<Html<String>> {
    let html = render(
        state,
        "queries/github/pr_duration.html",
        context! {
            current_nav => "github/pr_duration",
        },
    )?;
    Ok(Html(html))
}

pub async fn sources(State(state): State<Arc<AppState>>) -> AppResult<Html<String>> {
    let github_last_fetched = latest_fetch(&state.pool)?
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();

    Ok(Html(render(
        state,
        "sources/index.html",
        context! {
            current_nav => "sources",
            github_last_fetched,
        },
    )?))
}

pub async fn dashboard(State(state): State<Arc<AppState>>) -> AppResult<Html<String>> {
    Ok(Html(render(
        state,
        "dashboard.html",
        context! { current_nav => "dashboard" },
    )?))
}

pub async fn bookmark(State(state): State<Arc<AppState>>) -> AppResult<Html<String>> {
    Ok(Html(render(
        state,
        "bookmark.html",
        context! { current_nav => "bookmark" },
    )?))
}

pub async fn fetch_source(
    State(state): State<Arc<AppState>>,
    Path(source_id): Path<crate::sources::Source>,
) -> AppResult<Html<String>> {
    let timestamp = fetch_given_source(&state.pool, &source_id).await?;

    Ok(Html(render(
        state,
        "sources/fetch_source.html",
        context! {
            source_id => source_id,
            timestamp => timestamp.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        },
    )?))
}

pub async fn serve(host: &str, port: &str, pool: Pool) -> AppResult<()> {
    let reloader = AutoReloader::new(|notifier| {
        let mut env = Environment::new();
        // TODO embed templates
        let template_path = "templates";
        notifier.watch_path(template_path, true);
        env.set_source(Source::from_path(template_path));
        Ok(env)
    });

    let state = Arc::new(AppState {
        template_loader: reloader,
        pool,
    });

    // TODO embed static files
    let static_dir = ServeDir::new("dist")
        .precompressed_br()
        .precompressed_gzip();

    let app = Router::new()
        .route(
            "/query/merged_pr_duration_30_day_rolling_avg_hours.arrow",
            get(handler_merged_pr_duration_30_day_rolling_avg_hours),
        )
        .route("/query/github/pr_duration", get(github_pr_duration))
        .route("/sources/:source_id/fetch", post(fetch_source))
        .route("/sources", get(sources))
        .route("/dashboard", get(dashboard))
        .route("/bookmark", get(bookmark))
        .nest_service("/static", static_dir)
        .route("/", get(|| async { "Hello, World!" }))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let addr_str = format!("{}:{}", host, port);
    debug!("Parsing address for `serve` binding: {}", addr_str);
    let address: SocketAddr = format!("{}:{}", host, port).parse()?;

    info!("Server listening at {}...", address);
    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

pub fn render(
    state: Arc<AppState>,
    template: &str,
    ctx: minijinja::value::Value,
) -> AppResult<String> {
    let env = state.template_loader.acquire_env()?;
    let tmpl = env.get_template(template)?;
    Ok(tmpl.render(ctx)?)
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    debug!("Signal received, starting graceful shutdown");

    //opentelemetry::global::shutdown_tracer_provider();
}

pub struct AppState {
    template_loader: AutoReloader,
    pool: Pool,
}

/// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}
