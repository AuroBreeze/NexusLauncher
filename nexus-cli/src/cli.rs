use clap::{Args, Parser, Subcommand, ValueEnum};
use std::fmt::Display;
use std::str::FromStr;

#[derive(Parser)]
#[command(name = "Nexus Launcher")]
#[command(about = "A high-performance Minecraft launcher written in Rust", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>, // Option

    #[arg(short, long, global = true)]
    pub debug: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    // TODO: Add support for launching on older versions
    /// Launch a Minecraft game
    Launch(LaunchArgs),

    /// Authenticate
    Auth(AuthArgs),

    /// Check, download and install java
    Java(JavaArgs),

    /// Install various components like loaders or mods
    Install(InstallArgs),

    /// Set and get options
    Set(SetArgs),
    // TODO: Add a search subcommand and extract and migrate the search functionality from other commands
    // such as Search for Java, Search for mod, Search for loader, Search for version and so on.
    Search(SearchArgs),
}

#[derive(Args)]
pub struct SearchArgs {
    // category: String,
    /// Query string to search for the mod
    pub category: String,

    #[arg(short, long)]
    pub version: Option<String>,
}

// ==========================================
// Install Subcommands group
// ==========================================

#[derive(Args)]
pub struct InstallArgs {
    #[command(subcommand)]
    pub command: InstallCommands,
}

#[derive(Subcommand)]
pub enum InstallCommands {
    // TODO: Increase downloads of other loaders
    // TODO: Add a search function to the loader
    /// Download and install a loader (e.g., Fabric, Quilt)
    Loader(LoaderArgs),

    // TODO: Add a mod search and download function
    /// Download and install a mod
    Mod(ModArgs),

    // TODO: Downloads compatible with older versions
    //
    /// Download and install the game core
    Core(CoreArgs),
}

// ==========================================
// Component Arguments
// ==========================================
// TODO: Prioritize downloading the main file
#[derive(Args, Debug)]
pub struct CoreArgs {
    #[arg(short, long)]
    pub game_version: Option<String>,

    #[arg(short, long)]
    pub list: Option<String>,
}

#[derive(Args, Debug)]
pub struct LoaderArgs {
    // Game Version Name
    pub game_name: String,

    #[arg(short, long)]
    pub loader: Loaders,
}

#[derive(Args)]
pub struct ModArgs {
    /// Query string to search for the mod
    #[arg(short, long)]
    pub query: Option<String>,

    #[arg(
        short,
        long,
        default_value = "5", 
        value_parser = clap::value_parser!(i32).range(1..=10),
        help = "The maximum number is 10"
    )]
    pub limit: Option<i32>,

    /// Target game version for the mod
    #[arg(short, long)]
    pub game_version: Option<String>,

    /// Flag to trigger the download process
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub download: bool,
}

// ==========================================
// Enums & Other Arguments
// ==========================================

// PERF: Migrate code
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

#[derive(Args)]
pub struct SetArgs {
    /// Set a game name that becomes invalid when logging in with a genuine copy
    #[arg(short, long)]
    pub name: Option<String>,

    /// Set a game UUID that becomes invalid when logging in with a genuine copy
    #[arg(short, long)]
    pub uuid: Option<String>,

    /// Display settings
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    pub show: bool,

    /// Enable offline login
    #[arg(short, long, default_value = None)]
    pub offline: Option<bool>,
}

#[derive(Args)]
pub struct LaunchArgs {
    // TODO: The game version download has been moved to the `install` command; replace `game_version` here with the folder where the game is located.
    // Rename and update the names in other places as well
    /// The instance to launch
    pub instance_name: String,

    /// The username to use for the game
    #[arg(short, long, default_value = "Default")]
    pub player_name: String,

    /// Maximum memory allocation (in MB)
    #[arg(short, long, default_value = "2048")]
    pub max_memory: u32,

    /// Launch the game in offline mode
    #[arg(long, short)]
    pub offline: Option<bool>,

    /// Force a re-scan for Java
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    pub force_scan: bool,
}

#[derive(Args)]
pub struct AuthArgs {
    /// Sign in with Microsoft Device ID
    #[arg(long, conflicts_with = "logout")]
    pub login: bool,

    /// Clear auth data
    #[arg(long)]
    pub logout: Option<String>,
}

#[derive(Args)]
pub struct JavaArgs {
    /// The version of Java to download
    #[arg(short, long, default_value = "17")]
    pub version: u32,

    /// Scan for installed Java versions
    #[arg(long)]
    pub scan: bool,

    /// Force the download of a specific version of the runtime
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub download: bool,
}
