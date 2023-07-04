use anyhow::Result;
use dotenvy::dotenv;
use opsql::db::open_db_pool;
use opsql::web::serve;
use opsql::{config_value, init_config};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    tracing_subscriber::fmt::init();

    init_config("opsql.config")?;

    let database_string: String = config_value("database")
        .await
        .expect("Unable to get config for `database`");
    let pool = open_db_pool(database_string.as_str(), 1)?;

    serve("127.0.0.1", "3825", pool).await?;

    Ok(())
}
