use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use minijinja::{context, Environment, Source};
use minijinja_autoreload::AutoReloader;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{signal, time::sleep};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::{debug, info};

use crate::{
    db::{open_db_pool, Pool},
    sources::github::{fetch_pulls, latest_fetch, request_pulls},
};

pub async fn sources(State(state): State<Arc<AppState>>) -> Result<Html<String>, AppError> {
    let github_last_fetched = latest_fetch(&state.pool)?;

    Ok(Html(render(
        state,
        "sources/index.html",
        context! {
            current_nav => "sources",
            github_last_fetched,
        },
    )?))
}

pub async fn dashboard(State(state): State<Arc<AppState>>) -> Result<Html<String>, AppError> {
    Ok(Html(render(
        state,
        "dashboard.html",
        context! { current_nav => "dashboard" },
    )?))
}

pub async fn bookmark(State(state): State<Arc<AppState>>) -> Result<Html<String>, AppError> {
    Ok(Html(render(
        state,
        "bookmark.html",
        context! { current_nav => "bookmark" },
    )?))
}

pub async fn fetch_source(
    State(state): State<Arc<AppState>>,
    Path(source_id): Path<crate::sources::Source>,
) -> Result<Html<String>, AppError> {
    let _ = sleep(Duration::from_millis(1000)).await;

    let timestamp = match source_id {
        crate::sources::Source::Github => {
            // TODO get this from config
            //let repos = vec!["open-telemetry/opentelemetry-rust".to_string(), "gunrein/wallowa".to_string()];
            let repos = vec!["gunrein/wallowa".to_string()];

            let responses = request_pulls(state.pool.clone(), &repos).await?;

            fetch_pulls(&state.pool, &responses)?
        }
    };

    Ok(Html(render(
        state,
        "sources/fetch_source.html",
        context! {
            source_id => source_id,
            timestamp => timestamp.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        },
    )?))
}

pub async fn serve(host: &str, port: &str) -> Result<()> {
    let reloader = AutoReloader::new(|notifier| {
        let mut env = Environment::new();
        // TODO embed templates
        let template_path = "templates";
        notifier.watch_path(template_path, true);
        env.set_source(Source::from_path(template_path));
        Ok(env)
    });

    // TODO load parameters from config
    let pool = open_db_pool(":memory:", 2)?;

    let state = Arc::new(AppState {
        template_loader: reloader,
        pool,
    });

    // TODO embed static files
    let static_dir = ServeDir::new("static")
        .precompressed_br()
        .precompressed_gzip();

    let app = Router::new()
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

// Adapted from https://github.com/tokio-rs/axum/blob/c97967252de9741b602f400dc2b25c8a33216039/examples/anyhow-error-response/src/main.rs under MIT license
// Make our own error that wraps `anyhow::Error`.
pub struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

type AppResult<T> = anyhow::Result<T, AppError>;
