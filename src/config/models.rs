use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// The structure representing the launcher's persistent settings.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LauncherConfig {
    /// The username of the current user
    pub user_profile: UserProfiles,

    /// A mapping from username to UUID
    pub username: HashMap<String, String>,

    /// A mapping from Java major version to its executable path
    /// e.g., 17 = "/usr/lib/jvm/java-17-openjdk/bin/java"
    pub java_paths: HashMap<u32, PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UserProfiles {
    pub offline: UserProfile,
    pub online: UserProfile,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UserProfile {
    pub username: String,
    pub uuid: String,
}
