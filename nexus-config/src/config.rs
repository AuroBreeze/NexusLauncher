use nexus_core::AnyError;
use serde::{Serialize, de::DeserializeOwned};
use std::path::PathBuf;
use tokio::fs;

pub trait Config: Serialize + DeserializeOwned + Default {
    fn get_config_path() -> PathBuf;

    #[allow(async_fn_in_trait)]
    async fn load() -> Self {
        let path = Self::get_config_path();
        if path.exists()
            && let Ok(content) = fs::read_to_string(&path).await
        {
            match toml::from_str::<Self>(&content) {
                Ok(config) => {
                    tracing::debug!("Successfully loaded configuration from TOML.");
                    return config;
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to parse TOML config at {}, falling back to default: {}",
                        path.display(),
                        e
                    );
                }
            }
        }
        tracing::debug!("No valid config found, using default settings.");

        Self::default()
    }

    /// Saves the current configuration to disk as a TOML file.
    #[allow(async_fn_in_trait)]
    async fn save(&self) -> Result<(), AnyError> {
        let path = Self::get_config_path();
        // Create a temporary file
        let temp_path = path.with_extension("toml.tmp");

        // Serialize the struct into a nicely formatted TOML string
        let content = toml::to_string_pretty(self)?;
        fs::write(&temp_path, content).await?;

        // Move the temporary file into place
        fs::rename(&temp_path, &path).await?;
        tracing::debug!("Launcher configuration saved to {}", path.display());
        Ok(())
    }
}
