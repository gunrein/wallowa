use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use minijinja::{context, Environment, Source};
use minijinja_autoreload::AutoReloader;
use std::{net::SocketAddr, sync::Arc};
use tokio::signal;
use tower_http::trace::TraceLayer;
use tower_http::{compression::CompressionLayer, services::ServeDir, CompressionLevel};
use tracing::{debug, info};

use crate::{
    config_value,
    db::Pool,
    github::{
        fetch::latest_fetch,
        web::{data_routes, page_routes},
    },
    AppError, AppResult,
};

pub async fn sources(State(state): State<Arc<AppState>>) -> AppResult<Html<String>> {
    let github_last_fetched = latest_fetch(&state.pool)?
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();

    Ok(Html(render(
        state,
        "sources/index.html",
        context! {
            current_nav => "/sources",
            github_last_fetched,
        },
    )?))
}

pub async fn dashboard(State(state): State<Arc<AppState>>) -> AppResult<Html<String>> {
    Ok(Html(render(
        state,
        "dashboard.html",
        context! { current_nav => "/dashboard" },
    )?))
}

pub async fn bookmark(State(state): State<Arc<AppState>>) -> AppResult<Html<String>> {
    Ok(Html(render(
        state,
        "bookmark.html",
        context! { current_nav => "/bookmark" },
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

    let compression_level_cfg: String = config_value("server.response.compression.level")
        .await
        .expect("Config error for `server.response.compression.level`");
    let compression_level = match compression_level_cfg.to_ascii_lowercase().as_str() {
        "default" => CompressionLevel::Default,
        "best" => CompressionLevel::Best,
        "fastest" => CompressionLevel::Fastest,
        _ => CompressionLevel::Fastest,
    };
    let compression_layer = CompressionLayer::new()
        .br(config_value("server.response.compression.br")
            .await
            .expect("Config error for `server.response.compression.br`"))
        .gzip(
            config_value("server.response.compression.gzip")
                .await
                .expect("Config error for `server.response.compression.gzip`"),
        )
        .zstd(
            config_value("server.response.compression.zstd")
                .await
                .expect("Config error for `server.response.compression.zstd`"),
        )
        .deflate(
            config_value("server.response.compression.deflate")
                .await
                .expect("Config error for `server.response.compression.deflate`"),
        )
        .quality(compression_level);

    let app = Router::new()
        .nest("/github", page_routes())
        .nest("/data", Router::new().nest("/github", data_routes()))
        .route("/sources", get(sources))
        .route("/dashboard", get(dashboard))
        .route("/bookmark", get(bookmark))
        // The compression layer comes before the `/static` since `/static` is pre-compressed
        // by the build process (for release builds)
        .layer(compression_layer)
        .nest_service("/static", static_dir)
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
    pub pool: Pool,
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
