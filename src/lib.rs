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
pub async fn get_config<T: for<'de> serde::de::Deserialize<'de>>(key: &str) -> Result<T> {
    let val = if let Some(lock) = CONFIG.get() {
        lock.read().await.get::<T>(key)
    } else {
        panic!("Unable to get lock on config");
    }?;

    Ok(val)
}
