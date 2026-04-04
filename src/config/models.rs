use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// The structure representing the launcher's persistent settings.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LauncherConfig {
    /// A mapping from Java major version to its executable path
    /// e.g., 17 = "/usr/lib/jvm/java-17-openjdk/bin/java"
    pub java_paths: HashMap<u32, PathBuf>,
}
