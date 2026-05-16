use clap::{Args, Parser, Subcommand};
use nexus_core::Loaders;

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
    /// Search for mods, loaders, and more
    Search(SearchArgs),
}

#[derive(Args)]
pub struct SearchArgs {
    #[command(subcommand)]
    pub command: SearchCommands,
}

#[derive(Subcommand)]
pub enum SearchCommands {
    /// Search for mods on Modrinth
    Mod(SearchModArgs),
    /// Search for installed Java runtimes
    Java(SearchJavaArgs),
    /// List cached user profiles from a game instance's usercache.json
    User(SearchUserArgs),
    // TODO: Add search subcommands for Loader, Version, etc.
}

#[derive(Args, Debug)]
pub struct SearchUserArgs {
    /// The name of the game instance (e.g. "1.20")
    pub instance: String,
}

#[derive(Args, Debug)]
pub struct SearchJavaArgs {
    /// Filter by major Java version (e.g. 17, 21)
    #[arg(short, long)]
    pub version: Option<u32>,

    /// Force a fresh scan for installed Java before listing
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    pub scan: bool,
}

#[derive(Args, Debug)]
pub struct SearchModArgs {
    /// Query string to search for
    pub query: String,

    /// Maximum number of results (default 5, max 100)
    #[arg(short, long, default_value = "5")]
    pub limit: i32,

    /// Sort order: relevance, downloads, follows, newest, updated
    #[arg(short, long)]
    pub index: Option<String>,

    /// Filter by game version (e.g. "1.21.4")
    #[arg(short = 'g', long)]
    pub game_version: Option<String>,

    /// Number of results to skip for pagination
    #[arg(short = 'o', long)]
    pub offset: Option<i32>,
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

    /// Search and download mods
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

    /// Custom directory name for the game instance (defaults to the game version)
    #[arg(short, long)]
    pub name: Option<String>,

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

    /// Flag to trigger the download and install process
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub download: bool,

    /// Game instance name to install the mod into (required with --download)
    #[arg(short, long)]
    pub instance_name: Option<String>,

    /// Mod loader to filter versions for (required with --download)
    #[arg(short = 'L', long)]
    pub loader: Option<Loaders>,

    /// Release channel: release, beta, or alpha (omit for latest regardless of channel)
    #[arg(short = 't', long)]
    pub version_type: Option<String>,
}

// ==========================================
// Enums & Other Arguments
// ==========================================

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
