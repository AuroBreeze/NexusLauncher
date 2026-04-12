use std::{fs, path::Path};

use crate::version::{
    download::download_and_verify,
    models::VersionDetail,
    source::{download_assets, download_libraries},
};

pub mod download;
pub mod models;
pub mod source;
pub mod utils;

pub type AnyError = Box<dyn std::error::Error + Send + Sync>;

pub async fn verify_game_integrity(game_path: &Path) -> Result<(), AnyError> {
    let game_version_json_path = game_path.join("version.json");
    if !game_version_json_path.exists() {
        return Err("Game version JSON not found".into());
    }
    let data = fs::read_to_string(game_version_json_path)?;
    let detail: VersionDetail = serde_json::from_str(&data)?;

    let target_version = &detail.id;

    let client_jar_path = utils::get_clients_dir()
        .join(target_version)
        .join(format!("{}.jar", target_version));

    if !client_jar_path.exists() {
        tracing::info!("Verifying core JAR file...");
        download_and_verify(
            &detail.downloads.client.url,
            &client_jar_path,
            detail.downloads.client.sha1.as_str(),
        )
        .await?;
    }

    download_libraries(&detail).await?;
    download_assets(&detail).await?;
    tracing::info!("Fetching version data for {}...", target_version);

    Ok(())
}
