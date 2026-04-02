// src/config.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use crate::java;
use crate::version::utils;
use crate::version::AnyError;

/// The structure representing the launcher's persistent settings.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LauncherConfig {
    /// A mapping from Java major version to its executable path
    /// e.g., 17 = "/usr/lib/jvm/java-17-openjdk/bin/java"
    pub java_paths: HashMap<u32, PathBuf>,
}

impl LauncherConfig {
    /// Gets the path to the configuration file (e.g., ~/.minecraft/nexus_config.toml)
    fn get_config_path() -> PathBuf {
        utils::get_minecraft_dir().join("nexus_config.toml")
    }

    /// Loads the configuration from disk. 
    /// If it doesn't exist or is invalid, returns a default (empty) config.
    pub async fn load() -> Self {
        let path = Self::get_config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path).await {
                match toml::from_str(&content) {
                    Ok(config) => {
                        tracing::debug!("Successfully loaded launcher configuration from TOML.");
                        return config;
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse TOML config, falling back to default: {}", e);
                    }
                }
            }
        }
        tracing::debug!("No valid config found, using default settings.");
        LauncherConfig::default()
    }

    /// Saves the current configuration to disk as a TOML file.
    pub async fn save(&self) -> Result<(), AnyError> {
        let path = Self::get_config_path();
        // Serialize the struct into a nicely formatted TOML string
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content).await?;
        tracing::debug!("Launcher configuration saved to {}", path.display());
        Ok(())
    }

    /// Checks if a cached Java path is still valid (exists and is a file).
    pub async fn get_valid_java(&self, major_version: u32) -> Option<PathBuf> {
        if let Some(path) = self.java_paths.get(&major_version) {
            
            // Instead of checking is_file(), we execute it to verify.
            // This perfectly handles system PATH commands like "java" and absolute paths.
            if let Some(info) = java::check_java_executable(path).await {
                
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
