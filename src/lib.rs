use std::sync::OnceLock;

use anyhow::Result;
use config::Config;
use tokio::sync::RwLock;

pub mod cli;
pub mod db;
pub mod github;
pub mod web;

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
        .set_default("database", "opsql.db")?
        .set_default("github.per_page", "100")?
        .set_default::<&str, Vec<String>>("github.repos", vec![])?
        .set_default("server.host", "127.0.0.1")?
        .set_default("server.port", "3825")?
        .set_default("server.response.compression.br", false)?
        .set_default("server.response.compression.gzip", true)?
        .set_default("server.response.compression.zstd", true)?
        .set_default("server.response.compression.deflate", true)?
        .set_default("server.response.compression.level", "fastest")?
        .add_source(config::File::with_name(config_path))
        .add_source(env_source)
        .build()?;

    let _ = CONFIG.set(RwLock::new(config));

    Ok(())
}

// Adapted from https://github.com/tokio-rs/axum/blob/c97967252de9741b602f400dc2b25c8a33216039/examples/anyhow-error-response/src/main.rs under MIT license
// Make our own error that wraps `anyhow::Error`.
#[derive(Debug)]
pub struct AppError(anyhow::Error);

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

pub const NEW_GITIGNORE: &str = r#"
# Avoid committing sensitive environment variables to source control
.env

opsql.db.wal

# Optionally ignore the database itself
#opsql.db
"#;

pub const NEW_DOT_ENV: &str = r#"# Put your authentication keys in this file to avoid committing
# them to source control.
WALLOWA_GITHUB_AUTH_TOKEN='YOUR_TOKEN'
"#;

pub const NEW_CONFIG: &str = r#"# Config files are looked for at
# `opsql.config.[toml | json | yaml | ini | ron | json5]` by default.
# This file is in [TOML](https://github.com/toml-lang/toml) format.
# You can specify a config file to use with the `opsql --config CONFIG`
# argument or using the `OPSQL_CONFIG` environment variable
# (`OPSQL_CONFIG=opsql.config.toml`, for example).

# Add any GitHub repos that you'd like to track inside the `repos = []`
# brackets. For example, "gunrein/opsql" is currently configured.
# Default: [] (empty list)
[github]
repos = ["gunrein/opsql"]
# The number of items to fetch per page. Default: 100
#per_page = "100"

# The database file to use. Default: opsql.db
#database = "opsql.db"

[server]
# The network address to bind to. Default: 127.0.0.1
#host = "127.0.0.1"
# The network port to bind to. Default: 3825
#port = "3825"

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
