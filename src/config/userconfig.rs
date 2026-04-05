use super::models::UserConfig;
use crate::config::config::Config;
use crate::version::utils;
use std::path::PathBuf;

impl Config for UserConfig {
    /// Gets the path to the configuration file (e.g., ~/.minecraft/nexus_config.toml)
    fn get_config_path() -> PathBuf {
        utils::get_minecraft_dir().join("nexus_config.toml")
    }
}
