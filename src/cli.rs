use clap::{Args, Parser, Subcommand, ValueEnum};
use std::fmt::Display;
use std::str::FromStr;

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

    /// download and install mode
    Mode(ModeArgs),

    /// download and install loader
    Loader(LoaderArgs),
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Loaders {
    Fabric,
    Quilt,
}

impl FromStr for Loaders {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "fabric" => Ok(Loaders::Fabric),
            "quilt" => Ok(Loaders::Quilt),
            _ => Err(format!(
                "Invalid loader: {}. Expected 'fabric' or 'quilt'",
                s
            )),
        }
    }
}

impl Display for Loaders {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Loaders::Fabric => write!(f, "fabric"),
            Loaders::Quilt => write!(f, "quilt"),
        }
    }
}

#[derive(Args, Debug)]
pub struct LoaderArgs {
    /// The game version to install the loader for
    pub game_version: String,

    #[arg(short, long)]
    pub loader: Loaders,
}

#[derive(Args)]
pub struct ModeArgs {
    // Query
    #[arg(short, long)]
    pub query: String,

    #[arg(short, long)]
    pub game_version: String,

    /// Download
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub download: bool,
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
    #[arg(long, conflicts_with = "logout")]
    pub login: bool,

    /// clear auth
    #[arg(long)]
    pub logout: Option<String>,
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
