use std::sync::OnceLock;

use anyhow::Result;
use tokio::sync::RwLock;
use dotenvy::dotenv;
use config::Config;
use opsql::db::open_db_pool;
use opsql::web::serve;

pub static CONFIG: OnceLock<RwLock<Config>> = OnceLock::new();

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    tracing_subscriber::fmt::init();

    let config_path = "opsql.config";

    let config = config::Config::builder()
        .set_default("database", "opsql.db")?
        .add_source(config::File::with_name(config_path))
        .add_source(config::Environment::with_prefix("WALLOWA"))
        .build()?;

    let _ = CONFIG.set(RwLock::new(config));

    let database_string = if let Some(lock) = CONFIG.get() {
        lock.read().await.get_string("database")?
    } else {
        "opsql.db".to_string()
    };
    let _pool = open_db_pool(database_string.as_str(), 1)?;

    serve("127.0.0.1", "3825").await?;

    Ok(())
}