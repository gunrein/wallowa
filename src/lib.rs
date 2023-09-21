use std::path::Path;
use std::{sync::OnceLock, time::Duration};

use anyhow::Result;
use config::Config;
use db::Pool;
use tokio::fs::{try_exists, DirBuilder, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::{
    sync::RwLock,
    task::{self, JoinHandle},
    time,
};
use tracing::error;
use tracing::{debug, info, metadata::LevelFilter};
use tracing_subscriber::{
    fmt, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

pub mod cli;
pub mod db;
pub mod github;
pub mod web;

/// Fetch all of the configured data sources in the background one time
pub async fn fetch_all(pool: &Pool) -> AppResult<()> {
    info!("Fetching in background");
    github::fetch::fetch_all(pool).await?;
    info!("Fetching in background complete");
    Ok(())
}

/// Fetch all of the configured data sources in the background on the interval
/// configured with `fetch.interval` (default: 1 hour) if `fetch.enabled` is
/// true (default: true).
pub async fn fetch_all_periodically(pool: &Pool) -> AppResult<JoinHandle<()>> {
    let fetch_enabled: bool = config_value("fetch.enabled").await?;
    if fetch_enabled {
        let fetch_interval: u64 = config_value("fetch.interval").await?;
        debug!(
            "Background fetch task started with interval {} seconds",
            fetch_interval
        );
        let mut interval = time::interval(Duration::from_secs(fetch_interval));
        let pool = pool.clone();

        let forever = task::spawn(async move {
            loop {
                interval.tick().await;
                match fetch_all(&pool).await {
                    Ok(_) => (),
                    Err(e) => debug!("Error with periodic fetch all: {:?}", e),
                }
            }
        });
        Ok(forever)
    } else {
        debug!("Background fetch task disabled");
        Ok(task::spawn(async {})) // Intentional no-op
    }
}

/// Global static reference to a RwLock'd configuration initialized in `main`
pub static CONFIG: OnceLock<RwLock<Config>> = OnceLock::new();

/// Utility for getting the config value for a given `key`
//pub async fn config_value<T: for<'de> serde::de::Deserialize<'de>>(key: &str) -> Result<T> {
pub async fn config_value<'de, T: serde::de::Deserialize<'de>>(key: &str) -> Result<T> {
    let val = if let Some(lock) = CONFIG.get() {
        lock.read().await.get::<T>(key)
    } else {
        panic!("Unable to get lock on config");
    }?;

    Ok(val)
}

/// Initialize the configuration system
pub fn init_config(config_path: &str) -> Result<()> {
    let env_source = config::Environment::with_prefix("WALLOWA")
        .try_parsing(true)
        .separator("_")
        .list_separator(",")
        .with_list_parse_key("github.repos");

    let config = config::Config::builder()
        .set_default("database", "wallowa.db")?
        .set_default("github.per_page", "100")?
        .set_default::<&str, Vec<String>>("github.repos", vec![])?
        .set_default("server.host", "0.0.0.0")?
        .set_default("server.port", "9843")?
        .set_default("server.response.compression.br", false)?
        .set_default("server.response.compression.gzip", true)?
        .set_default("server.response.compression.zstd", true)?
        .set_default("server.response.compression.deflate", true)?
        .set_default("server.response.compression.level", "fastest")?
        .set_default("fetch.enabled", "true")?
        .set_default("fetch.interval", "3600")?
        .add_source(config::File::with_name(config_path))
        .add_source(env_source)
        .build()?;

    let _ = CONFIG.set(RwLock::new(config));

    Ok(())
}

/// Initialize the logging system
pub fn init_logging(log_format: &Option<String>) -> Result<()> {
    let plain_format = fmt::format()
        .with_level(false)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .without_time()
        .compact();

    let env_filter = EnvFilter::builder()
        .with_env_var("WALLOWA_LOG")
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    if let Some(log_format_string) = log_format {
        match log_format_string.as_str() {
            "full" => {
                tracing_subscriber::registry()
                    .with(fmt::layer())
                    .with(env_filter)
                    .init();
            }
            "compact" => {
                tracing_subscriber::registry()
                    .with(fmt::layer().compact())
                    .with(env_filter)
                    .init();
            }
            "pretty" => {
                tracing_subscriber::registry()
                    .with(fmt::layer().pretty())
                    .with(env_filter)
                    .init();
            }
            "json" => {
                tracing_subscriber::registry()
                    .with(fmt::layer().json())
                    .with(env_filter)
                    .init();
            }
            _ => {
                tracing_subscriber::registry()
                    .with(fmt::layer().event_format(plain_format))
                    .with(env_filter)
                    .init();
            }
        }
    } else {
        tracing_subscriber::registry()
            .with(fmt::layer().event_format(plain_format))
            .with(env_filter)
            .init();
    }

    Ok(())
}

// Adapted from https://github.com/tokio-rs/axum/blob/c97967252de9741b602f400dc2b25c8a33216039/examples/anyhow-error-response/src/main.rs under MIT license
// Make our own error that wraps `anyhow::Error`.
#[derive(Debug)]
pub struct AppError(pub anyhow::Error);

/// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
/// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

pub type AppResult<T> = anyhow::Result<T, AppError>;

/// Create a new project at the given `path`
pub async fn create_project(path: &str) -> Result<()> {
    if try_exists(&path).await? {
        error!("Directory `{path}` already exists. Cancelled.");
        return Ok(());
    }

    DirBuilder::new().recursive(true).create(&path).await?;
    let project_path = Path::new(&path);

    let mut outfile = OpenOptions::new()
        .read(false)
        .write(true)
        .create(true)
        .truncate(true)
        .open(project_path.join("wallowa.config.toml"))
        .await?;
    outfile.write_all(NEW_CONFIG.as_bytes()).await?;
    outfile.flush().await?;

    let mut outfile = OpenOptions::new()
        .read(false)
        .write(true)
        .create(true)
        .truncate(true)
        .open(project_path.join(".env"))
        .await?;
    outfile.write_all(NEW_DOT_ENV.as_bytes()).await?;
    outfile.flush().await?;

    let mut outfile = OpenOptions::new()
        .read(false)
        .write(true)
        .create(true)
        .truncate(true)
        .open(project_path.join(".gitignore"))
        .await?;
    outfile.write_all(NEW_GITIGNORE.as_bytes()).await?;
    outfile.flush().await?;

    info!("A new project has been created at `{path}`");
    info!("");
    info!("To get started:");
    info!("");
    info!("  1. Add your GitHub repos to `wallowa.config.toml`");
    info!("  2. Add your GitHub access key to `.env`");
    info!("  3. Fetch initial data: `wallowa fetch` (this can take a while for active repos)");
    info!("  4. Start the server: `wallowa serve`");
    info!("  5. Open your browser to http://localhost:9843/");
    info!("");
    info!("Check out the documentation at https://localhost:9843/docs/ or https://www.wallowa.io/docs/");
    info!("");
    info!("Enjoy!");
    Ok(())
}

const NEW_GITIGNORE: &str = r#"
# Avoid committing sensitive environment variables to source control
.env

wallowa.db.wal

# Optionally ignore the database itself
#wallowa.db
"#;

const NEW_DOT_ENV: &str = r#"# See http://wallowa.io/docs/configuration for documentation

# Put your authentication keys in this file to avoid committing
# them to source control.
WALLOWA_GITHUB_AUTH_TOKEN='YOUR_TOKEN'
"#;

const NEW_CONFIG: &str = r#"# See http://wallowa.io/docs/configuration for documentation

# Config files are loaded from
# `wallowa.config.[toml | json | yaml | ini | ron | json5]` by default (this file).
# This file is in [TOML](https://github.com/toml-lang/toml) format.
# You can specify a config file to use with the `wallowa --config CONFIG`
# argument or using the `WALLOWA_CONFIG` environment variable
# (`WALLOWA_CONFIG=wallowa.config.toml`, for example).

# Add any GitHub repos that you'd like to track inside the `repos = []`
# brackets. For example, "open-telemetry/opentelemetry-rust" is currently configured.
# Default: [] (empty list)
[github]
repos = ["open-telemetry/opentelemetry-rust"]
# The number of items to fetch per page (maximum of 100). Default: 100
#per_page = "100"

# The database file to use. Default: wallowa.db
#database = "wallowa.db"

[fetch]
# The time interval to wait between fetching for additional data, in seconds.
# Default: 3600 seconds (1 hour)
#interval = 3600
# Whether to fetch new data in the background. If this is disabled, then use the
# `wallowa fetch` CLI command to fetch whenever you'd like.
# Default: true (enabled)
#enabled = true

[server]
# The network address to bind to. Default: 0.0.0.0
#host = "0.0.0.0"
# The network port to bind to. Default: 9843
#port = "9843"

[server.response.compression]
# Compression level to use for HTTP server responses. Options are:
# algo_default, best, fastest. Default: fastest
# `algo_default` uses the default compression level for the given type.
# See https://docs.rs/tower-http/0.4.1/tower_http/enum.CompressionLevel.html#variant.Default
#level = "fastest"

# Use brotli compression for HTTP server responses when requested by
# the client. Default: false
#br = false

# Use gzip compression for HTTP server responses when requested by
# the client. Default: true
#gzip = true

# Use zstd compression for HTTP server responses when requested by
# the client. Default: true
#zstd = true

# Use deflate compression for HTTP server responses when requested by
# the client. Default: true
#deflate = true
"#;
