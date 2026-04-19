use crate::{config::Config, models::LaunchConfig};
use nexus_core::get_minecraft_dir;
use nexus_java::java::check_java_executable;
use std::collections::HashMap;
use std::path::PathBuf;

impl Config for LaunchConfig {
    fn get_config_path() -> PathBuf {
        get_minecraft_dir().join("launch_config.toml")
    }
}

impl Default for LaunchConfig {
    fn default() -> Self {
        Self {
            java_paths: HashMap::new(),
            offline: true,
        }
    }
}

impl LaunchConfig {
    /// Checks if a cached Java path is still valid (exists and is a file).
    pub async fn get_valid_java(&self, major_version: u32) -> Option<PathBuf> {
        if let Some(path) = self.java_paths.get(&major_version) {
            // Instead of checking is_file(), we execute it to verify.
            // This perfectly handles system PATH commands like "java" and absolute paths.
            if let Some(info) = check_java_executable(path).await {
                // Double check if the major version still matches
                // (in case the user updated their system "java" environment variable)
                if info.major_version == major_version {
                    return Some(path.clone());
                } else {
                    tracing::warn!(
                        "Cached Java path exists, but version changed from {} to {}",
                        major_version,
                        info.major_version
                    );
                }
            } else {
                tracing::warn!(
                    "Cached Java path for version {} is invalid or missing: {}",
                    major_version,
                    path.display()
                );
            }
        }
        None
    }
}
