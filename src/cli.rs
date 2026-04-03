use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "Nexus Launcher")]
#[command(about = "A high-performance Minecraft launcher written in Rust", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, global = true)]
    pub debug: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Launch a Minecraft game
    Launch(LaunchArgs),

    /// Authenticate
    Auth(AuthArgs),

    /// check, download and install java
    Java(JavaArgs),
}

#[derive(Args)]
pub struct LaunchArgs {
    /// The version to launch
    pub game_version: String,

    /// The username to use for the game
    #[arg(short, long, default_value = "Default")]
    pub player_name: String,

    #[arg(short, long, default_value = "2048")]
    pub max_memory: u32,

    /// Launch the game in offline mode
    #[arg(long)]
    pub offline: bool,

    /// Force a re-scan for Java
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    pub force_scan: bool,
}

#[derive(Args)]
pub struct AuthArgs {
    /// Sign in with Microsoft Device ID
    #[arg(long)]
    pub login: bool,

    /// clear auth
    #[arg(long)]
    pub logout: bool,
}

#[derive(Args)]
pub struct JavaArgs {
    /// The version of Java to download
    #[arg(short, long, default_value = "17")]
    pub version: u32,

    /// Scan for Java
    #[arg(long)]
    pub scan: bool,

    /// Force the download of a specific version of the runtime (e.g., 17)
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub download: bool,
}
