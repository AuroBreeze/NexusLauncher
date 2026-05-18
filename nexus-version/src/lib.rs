pub mod download;
pub mod models;
pub mod source;

pub use crate::source::verify_game_integrity;
pub use nexus_core::AnyError;
use nexus_core::get_clients_dir;

use crate::download::download_and_verify;
use crate::source::{download_assets, download_libraries, fetch_version_detail, obtain_manifest};

/// Download and install a game core version into the clients directory.
///
/// If the directory already exists, verifies integrity instead of re-downloading.
pub async fn install_game_core(game_version: &str, dir_name: &str) -> Result<(), AnyError> {
    let version_dir = get_clients_dir().join(dir_name);
    let local_json_path = version_dir.join("version.json");

    if local_json_path.exists() {
        let content = tokio::fs::read_to_string(&local_json_path).await?;
        let detail: serde_json::Value = serde_json::from_str(&content)?;
        let existing_id = detail["id"].as_str().unwrap_or("");
        if existing_id != game_version {
            return Err(format!(
                "Instance '{}' already contains version {} (requested {}). \
                 Use `uninstall instance {}` first, or specify a different --name.",
                dir_name, existing_id, game_version, dir_name
            )
            .into());
        }
        tracing::info!(
            "Game directory '{}' already exists, verifying integrity...",
            dir_name
        );
        verify_game_integrity(&version_dir).await?;
    } else {
        let manifest = obtain_manifest().await?;

        let v_info = manifest
            .versions
            .iter()
            .find(|v| v.id == game_version)
            .ok_or_else(|| format!("Version {} not found in manifest", game_version))?;

        tracing::info!("Fetching version data for {}...", game_version);
        let detail = fetch_version_detail(&v_info.url).await?;

        tokio::fs::create_dir_all(&version_dir).await?;

        download_and_verify(
            &detail.downloads.client.url,
            &version_dir.join(format!("{}.jar", game_version)),
            detail.downloads.client.sha1.as_str(),
        )
        .await?;

        download_libraries(&detail).await?;
        download_assets(&detail).await?;

        let json_content = serde_json::to_string_pretty(&detail)?;
        tokio::fs::write(&local_json_path, json_content).await?;
    }

    tracing::info!("All core components for {} are ready!", game_version);

    Ok(())
}
