use clap::Parser;
use dotenvy::dotenv;
use tokio::join;
use tracing::error;
use wallowa::cli::{Cli, Commands};
use wallowa::db::open_db_pool;
use wallowa::web::serve;
use wallowa::{
    config_value, create_project, fetch_all, fetch_all_periodically, init_config, init_logging,
    AppResult,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> AppResult<()> {
    dotenv().ok();

    let cli = Cli::parse();

    init_logging(&cli.log_format)?;

    match cli.command {
        Some(Commands::Fetch {}) => {
            // Fetches from all sources
            if let Some(cmd_line_cfg_file) = cli.config {
                init_config(cmd_line_cfg_file.as_str())?;
            } else {
                init_config("wallowa.config")?;
            }

            // Each source is expected to run *only* if it is configured
            let database_string: String = config_value("database").await?;
            let pool = open_db_pool(database_string.as_str(), 1)?;

            if let Err(error) = fetch_all(&pool).await {
                let e: anyhow::Error = error.0;
                error!("{e:#}")
            };
        }
        Some(Commands::New { path }) => {
            create_project(&path).await?;
        }
        Some(Commands::Serve {}) | None => {
            if let Some(cmd_line_cfg_file) = cli.config {
                init_config(cmd_line_cfg_file.as_str())?;
            } else {
                init_config("wallowa.config")?;
            }

            let database_string: String = config_value("database").await?;
            let pool = open_db_pool(database_string.as_str(), 1)?;

            let fetcher = fetch_all_periodically(&pool);

            let host: String = config_value("server.host").await?;
            let port: String = config_value("server.port").await?;
            let server = serve(&host, &port, pool.clone());

            let (fetcher_result, server_result) = join!(fetcher, server);
            fetcher_result?;
            server_result?;
        }
    }

    Ok(())
}
