use super::models::{FabricLoaderResponse, FabricProfile};
use nexus_version::{AnyError, utils::get_clients_dir};
use serde_json::Value;
use std::path::PathBuf;
use tokio::fs;

/// Find the version JSON file within the game directory
pub fn find_game_json(game_name: &str) -> Result<PathBuf, AnyError> {
    // 1. Get the base directory and join with game_name
    let clients_dir = get_clients_dir();
    let target_dir = clients_dir.join(game_name);

    // 2. Check if the directory exists and is indeed a directory
    if !target_dir.is_dir() {
        // Return error if directory not found
        return Err(format!(
            "Game directory '{}' not found at {:?}",
            game_name, target_dir
        )
        .into());
    }

    // 3. Read the directory and look for the first .json file
    let json_file = std::fs::read_dir(&target_dir)?
        .filter_map(|entry| entry.ok()) // Ignore IO errors on individual entries
        .map(|entry| entry.path())
        .find(|path| {
            // Check if it's a file and has the "json" extension
            path.is_file() && path.file_name().unwrap().to_str().unwrap() == "version.json"
        })
        .ok_or_else(|| format!("No JSON version file found in {}", target_dir.display()))?;

    Ok(json_file)
}

/// Find a JSON file starting with "fabric" in the specified directory
pub fn find_fabric_json(dir_path: &PathBuf) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    // 1. Check if the directory exists
    if !dir_path.is_dir() {
        return Ok(None);
    }

    // 2. Read the directory and filter entries
    let target_file = std::fs::read_dir(dir_path)?
        .filter_map(|entry| entry.ok()) // Skip entries with IO errors
        .map(|entry| entry.path()) // Convert DirEntry to PathBuf
        .find(|path| {
            // Check if it's a file
            if !path.is_file() {
                return false;
            }

            // Get the file name as a string for prefix/suffix matching
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                // Logic: starts with "fabric" AND ends with ".json"
                name.starts_with("fabric") && name.ends_with(".json")
            } else {
                false
            }
        });

    Ok(target_file)
}

/// fetch latest stable Fabric Loader
pub async fn get_latest_loader(game_name: &str) -> Result<String, AnyError> {
    tracing::info!("Fetching latest Fabric Loader for {}", game_name);

    let json_file_path = find_game_json(game_name)?;
    let data = std::fs::read_to_string(&json_file_path)?;
    let v: Value = serde_json::from_str(&data)?;
    let id = v["id"].as_str().ok_or("Cannot find game id")?;
    tracing::info!("Game version: {}", id);

    let url = format!("https://meta.fabricmc.net/v2/versions/loader/{}", id);
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
    game_name: &str,
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
    let save_path = get_clients_dir().join(game_name).join(name);
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

use nexus_version::{download::pool_download_and_link, utils::maven_to_path};

pub async fn install_fabric_libraries(
    profile: &FabricProfile,
    game_name: &str,
) -> Result<Vec<PathBuf>, AnyError> {
    tracing::info!(
        "Installing Fabric dependencies ({} in total)...",
        profile.libraries.len()
    );
    let mut classpath = Vec::new();

    for lib in &profile.libraries {
        let rel_path = maven_to_path(&lib.name);

        let download_url = format!("{}{}", lib.url, rel_path);

        match pool_download_and_link(&download_url, &rel_path, game_name).await {
            Ok(path) => classpath.push(path),
            Err(e) => tracing::error!("Failed to download the Fabric library {}: {}", lib.name, e),
        }
    }

    Ok(classpath)
}
