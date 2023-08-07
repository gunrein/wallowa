use std::path::Path;

use clap::Parser;
use dotenvy::dotenv;
use tokio::fs::{try_exists, DirBuilder, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::join;
use tracing::{error, info};
use wallowa::cli::{Cli, Commands};
use wallowa::db::open_db_pool;
use wallowa::web::serve;
use wallowa::{
    config_value, fetch_all, fetch_all_periodically, init_config, init_logging, AppResult,
    NEW_CONFIG, NEW_DOT_ENV, NEW_GITIGNORE,
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

            fetch_all(&pool).await?;
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
            info!(
                "  3. Fetch initial data: `wallowa fetch` (this can take a while for active repos)"
            );
            info!("  4. Start the server: `wallowa serve`");
            info!("  5. Open your browser to https://localhost:9843/");
            info!("");
            info!("Check out the documentation at https://localhost:9843/docs/ or https://www.wallowa.io/docs/");
            info!("");
            info!("Enjoy!");
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
