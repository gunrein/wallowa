use anyhow::Result;
use dotenvy::dotenv;
use opsql::db::open_db_pool;
use opsql::web::serve;
use opsql::{get_config, CONFIG};
use tokio::sync::RwLock;

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

    let database_string: String = get_config("database").await?;
    let pool = open_db_pool(database_string.as_str(), 1)?;

    serve("127.0.0.1", "3825", pool).await?;

    Ok(())
}
