use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use minijinja::{context, path_loader, Environment};
use minijinja_autoreload::AutoReloader;
use reqwest::header;
use rust_embed::RustEmbed;
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
        context! { current_nav => "/" },
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
    let (env, reloader) = if cfg!(debug_assertions) {
        (
            None,
            Some(AutoReloader::new(|notifier| {
                let mut env = Environment::new();
                let template_path = "templates";
                env.set_loader(path_loader(&template_path));
                notifier.set_fast_reload(true);
                notifier.watch_path(template_path, true);
                Ok(env)
            })),
        )
    } else {
        let mut env: Environment<'static> = Environment::new();
        for template_name in TemplateSrc::iter() {
            if let Some(template) = TemplateSrc::get(&template_name) {
                env.add_template_owned(
                    template_name,
                    String::from_utf8(template.data.into_owned())?,
                )?;
            }
        }
        (Some(env), None)
    };

    let state = Arc::new(AppState {
        template_loader: reloader,
        template_env: env,
        pool,
    });

    // TODO embed static files
    let static_dir = ServeDir::new("dist")
        .precompressed_br()
        .precompressed_gzip();

    let compression_level_cfg: String = config_value("server.response.compression.level").await?;
    let compression_level = match compression_level_cfg.to_ascii_lowercase().as_str() {
        "algo_default" => CompressionLevel::Default,
        "best" => CompressionLevel::Best,
        "fastest" => CompressionLevel::Fastest,
        _ => CompressionLevel::Fastest,
    };
    let compression_layer = CompressionLayer::new()
        .br(config_value("server.response.compression.br").await?)
        .gzip(config_value("server.response.compression.gzip").await?)
        .zstd(config_value("server.response.compression.zstd").await?)
        .deflate(config_value("server.response.compression.deflate").await?)
        .quality(compression_level);

    let app = Router::new()
        .nest("/github", page_routes())
        .nest("/data", Router::new().nest("/github", data_routes()))
        .route("/sources", get(sources))
        .route("/bookmark", get(bookmark))
        .route("/", get(dashboard))
        // The compression layer comes before `/static` since `/static` is pre-compressed
        // by the build process for release builds
        .layer(compression_layer)
        .nest_service("/static", static_dir)
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let addr_str = format!("{}:{}", host, port);
    debug!("Parsing address for `serve` binding: {}", addr_str);
    let address: SocketAddr = format!("{}:{}", host, port).parse()?;

    info!("Listening at {address}...");
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
    // Use the template loader if it exists, otherwise use the environment
    let rendered = if state.template_loader.is_some() {
        state
            .template_loader
            .as_ref()
            .unwrap()
            .acquire_env()?
            .get_template(template)?
            .render(ctx)
    } else {
        // If template_loader is None then template_env should be Some. If it isn't, treat the
        // situation as a fatal error.
        state
            .template_env
            .as_ref()
            .unwrap()
            .get_template(template)?
            .render(ctx)
    };
    Ok(rendered?)
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
    template_loader: Option<AutoReloader>,
    template_env: Option<Environment<'static>>,
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

#[derive(RustEmbed)]
#[folder = "templates/"]
struct TemplateSrc;

#[derive(RustEmbed)]
#[folder = "dist/"]
struct Asset;

pub struct StaticFile<T>(pub T);

impl<T> IntoResponse for StaticFile<T>
where
    T: Into<String>,
{
    fn into_response(self) -> Response {
        let path = self.0.into();

        match Asset::get(path.as_str()) {
            Some(content) => {
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
            }
            None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
        }
    }
}
