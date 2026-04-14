// src/launch/models.rs
use std::path::PathBuf;

use crate::loader::models::FabricProfile;

#[derive(Debug, Clone, Default)]
pub struct LaunchContext {
    // TODO: The game no longer relies on version numbers to launch; instead, it uses the folder name.
    pub game_path: PathBuf,
    pub version_id: String,         // such as "1.20.1-fabric"
    pub java_path: Option<PathBuf>, // Path to the verified Java executable file
    pub core_jar: PathBuf,          // Path to the original version's core jar
    pub user: UserContext,
    pub max_memory: Option<u32>,

    // The Classpath and the Main Class and other parameters
    pub main_class: String,
    pub libraries: Vec<PathBuf>,
    pub asset_index_id: String,
    pub fabric_loader: Option<FabricProfile>,
}

#[derive(Debug, Clone, Default)]
pub struct UserContext {
    pub username: String,
    pub uuid: String,
    pub access_token: Option<String>, // The access token for authentication
}
