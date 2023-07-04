use std::sync::OnceLock;

use anyhow::Result;
use config::Config;
use tokio::sync::RwLock;

pub mod db;
pub mod queries;
pub mod sources;
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
