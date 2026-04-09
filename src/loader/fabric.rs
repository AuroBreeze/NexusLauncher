use super::models::{FabricLoaderResponse, FabricProfile};
use crate::version::{AnyError, utils::get_minecraft_dir};
use std::path::PathBuf;
use tokio::fs;

/// fetch latest stable Fabric Loader
pub async fn get_latest_loader(game_version: &str) -> Result<String, AnyError> {
    tracing::info!("Fetching latest Fabric Loader for {}", game_version);
    let url = format!(
        "https://meta.fabricmc.net/v2/versions/loader/{}",
        game_version
    );
    let resp: Vec<FabricLoaderResponse> = reqwest::get(url).await?.json().await?;

    // find the first stable version
    let latest = resp
        .iter()
        .find(|v| v.loader.stable)
        .ok_or("Cannot find latest stable Fabric Loader")?;

    tracing::info!("Latest Fabric Loader: {}", latest.loader.version);
    Ok(latest.loader.version.clone())
}

/// Obtain Fabric profile (MainClass and Libraries)
pub async fn get_fabric_profile(
    game_version: &str,
    loader_version: &str,
) -> Result<FabricProfile, AnyError> {
    tracing::info!(
        "Fetching Fabric profile for {} ({})",
        game_version,
        loader_version
    );

    let url = format!(
        "https://meta.fabricmc.net/v2/versions/loader/{}/{}/profile/json",
        game_version, loader_version
    );

    let name = format!("fabric_profile_{}_{}.json", game_version, loader_version);
    let save_path = get_minecraft_dir().join("loader").join(name);
    // 1. Get the raw response text instead of directly deserializing
    let response_text = reqwest::get(url).await?.text().await?;

    // 2. Ensure the parent directory exists before writing
    if let Some(parent) = save_path.parent() {
        fs::create_dir_all(parent).await?;
    }

    // 3. Write the raw JSON text to the local file
    fs::write(&save_path, &response_text).await?;
    tracing::debug!("Fabric profile saved locally to {}", &save_path.display());

    // 4. Parse the JSON text into the FabricProfile struct
    let profile: FabricProfile = serde_json::from_str(&response_text)?;

    tracing::info!("Fabric profile obtained and saved");
    Ok(profile)
}

// src/loader/fabric.rs

use crate::version::{download::pool_download_and_link, utils::maven_to_path};

pub async fn install_fabric_libraries(profile: &FabricProfile) -> Result<Vec<PathBuf>, AnyError> {
    tracing::info!(
        "Installing Fabric dependencies ({} in total)...",
        profile.libraries.len()
    );
    let mut classpath = Vec::new();

    for lib in &profile.libraries {
        let rel_path = maven_to_path(&lib.name);

        let download_url = format!("{}{}", lib.url, rel_path);

        match pool_download_and_link(&download_url, &rel_path).await {
            Ok(path) => classpath.push(path),
            Err(e) => tracing::error!("Failed to download the Fabric library {}: {}", lib.name, e),
        }
    }

    Ok(classpath)
}
