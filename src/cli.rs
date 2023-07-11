use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
pub struct Cli {
    /// Optionally, set a custom config file
    #[arg(
        short,
        long,
        global = true,
        env = "WALLOWA_CONFIG",
        value_name = "CONFIG"
    )]
    pub config: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Fetch the latest data from configured sources
    Fetch {},

    /// Initialize a new project
    Init {},

    /// Serve the web app
    Serve {},
}
