use clap::Parser;
use dotenvy::dotenv;
use opsql::cli::{Cli, Commands};
use opsql::db::open_db_pool;
use opsql::web::serve;
use opsql::{config_value, init_config, AppResult};

#[tokio::main]
async fn main() -> AppResult<()> {
    dotenv().ok();

    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    if let Some(cmd_line_cfg_file) = cli.config {
        init_config(cmd_line_cfg_file.as_str())?;
    } else {
        init_config("opsql.config")?;
    }

    match cli.command {
        Some(Commands::Fetch {}) => {
            println!("TODO - implement `fetch`");
        }
        Some(Commands::Init {}) => {
            println!("TODO - implement `init`");
        }
        Some(Commands::Serve {}) => {
            let database_string: String = config_value("database")
                .await
                .expect("Unable to get config for `database`");
            let pool = open_db_pool(database_string.as_str(), 1)?;

            let host: String = config_value("server.host")
                .await
                .expect("Unable to get config value for `server.host`");
            let port: String = config_value("server.port")
                .await
                .expect("Unable to get config value for `server.port`");

            serve(&host, &port, pool).await?;
        }
        None => {
            println!(
                "No command provided. Please run `opsql help` for a list of available commands."
            );
        }
    }

    Ok(())
}
