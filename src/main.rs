use std::path::Path;

use clap::Parser;
use dotenvy::dotenv;
use opsql::cli::{Cli, Commands};
use opsql::db::open_db_pool;
use opsql::web::serve;
use opsql::{
    config_value, init_config, init_logging, AppResult, NEW_CONFIG, NEW_DOT_ENV, NEW_GITIGNORE,
};
use tokio::fs::{try_exists, DirBuilder, OpenOptions};
use tokio::io::AsyncWriteExt;
use tracing::{error, info};

#[tokio::main]
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
                init_config("opsql.config")?;
            }

            // Each source is expected to run *only* if it is configured
            let database_string: String = config_value("database").await?;
            let pool = open_db_pool(database_string.as_str(), 1)?;

            info!("Fetching from:");
            info!("    GitHub...");
            opsql::github::fetch::fetch_all(&pool).await?;
        }
        Some(Commands::New { path }) => {
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
                .open(project_path.join("opsql.config.toml"))
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
            info!("  1. Add your GitHub repos to `opsql.config.toml`");
            info!("  2. Add your GitHub access key to `.env`");
            info!(
                "  3. Fetch initial data: `opsql fetch` (this can take a while for active repos)"
            );
            info!("  4. Start the server: `opsql serve`");
            info!("  5. Open your browser to https://localhost:3825/");
            info!("");
            info!("Check out the documentation at https://localhost:3825/docs/ or https://www.opsql.io/docs/");
            info!("");
            info!("Enjoy!");
        }
        Some(Commands::Serve {}) | None => {
            if let Some(cmd_line_cfg_file) = cli.config {
                init_config(cmd_line_cfg_file.as_str())?;
            } else {
                init_config("opsql.config")?;
            }

            let database_string: String = config_value("database").await?;
            let pool = open_db_pool(database_string.as_str(), 1)?;

            let host: String = config_value("server.host").await?;
            let port: String = config_value("server.port").await?;

            serve(&host, &port, pool).await?;
        }
    }

    Ok(())
}
