use std::path::Path;

use clap::Parser;
use dotenvy::dotenv;
use opsql::cli::{Cli, Commands};
use opsql::db::open_db_pool;
use opsql::web::serve;
use opsql::{config_value, init_config, AppResult, NEW_CONFIG, NEW_DOT_ENV, NEW_GITIGNORE};
use tokio::fs::{try_exists, DirBuilder, OpenOptions};
use tokio::io::AsyncWriteExt;
use tracing::metadata::LevelFilter;
use tracing::{error, info};
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> AppResult<()> {
    dotenv().ok();

    let cli = Cli::parse();

    let plain_format = fmt::format()
        .with_level(false)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .without_time()
        .compact();

    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    if let Some(log_format) = cli.log_format {
        match log_format.as_str() {
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

            info!("Created new project at `{path}`");
        }
        Some(Commands::Serve {}) => {
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
        None => {
            error!(
                "No command provided. Please run `opsql help` for a list of available commands."
            );
        }
    }

    Ok(())
}
