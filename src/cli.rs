use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
pub struct Cli {
    /// Set a custom config file
    #[arg(
        long,
        global = true,
        env = "WALLOWA_CONFIG",
        value_name = "CONFIG",
        default_value = "opsql.config.toml"
    )]
    pub config: Option<String>,

    /// Set the log format. Accepted values are:
    ///
    /// - `terminal` - terminal-friendly human-readable basic log messages (the default)
    /// - `full` - richer human-readable log messages
    /// - `compact` - similar to `full`, but with less information
    /// - `pretty` - multi-line version of `full`
    /// - `json` - newline-delimited JSON logs
    ///
    /// See https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/#formatters
    /// for more details.
    #[arg(
        long,
        global = true,
        env = "WALLOWA_LOG_FORMAT",
        value_name = "LOG_FORMAT",
        default_value = "terminal"
    )]
    pub log_format: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Fetch the latest data from configured sources
    Fetch {},

    /// Create a new project in an new directory
    New {
        /// The path of the new project directory
        path: String,
    },

    /// Serve the web app
    Serve {},
}
