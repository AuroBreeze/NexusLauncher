use clap::Parser;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli{
    /// The version to launch
    #[arg(short, long, default_value = "1.20.1")]
    pub game_version: String,

    /// The username to use for the game
    #[arg(short, long, default_value = "AuroBreeze")]
    pub player_name: String,
    
    /// Allocate maximum memory
    #[arg(short, long, default_value = "2048")]
    pub max_memory: u32,

    /// Force a re-scan for Java
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    pub force_scan: bool
}



