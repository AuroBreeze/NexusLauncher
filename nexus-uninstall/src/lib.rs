use nexus_cli::cli::{UninstallInstanceArgs, UninstallModArgs};
use nexus_core::{AnyError, get_clients_dir, validate_instance_name};
use nexus_mods::models::ModManifest;

pub async fn handle_uninstall_instance(args: &UninstallInstanceArgs) -> Result<(), AnyError> {
    validate_instance_name(&args.instance)?;

    let inst_path = get_clients_dir().join(&args.instance);

    if !inst_path.exists() || !inst_path.is_dir() {
        tracing::warn!("Instance '{}' does not exist.", args.instance);
        return Ok(());
    }

    tracing::info!("Removing instance '{}'...", args.instance);
    tokio::fs::remove_dir_all(&inst_path).await?;
    tracing::info!("Instance '{}' removed.", args.instance);

    Ok(())
}

pub async fn handle_uninstall_mod(args: &UninstallModArgs) -> Result<(), AnyError> {
    validate_instance_name(&args.instance)?;

    let inst_path = get_clients_dir().join(&args.instance);
    let mods_dir = inst_path.join("mods");

    if !mods_dir.exists() || !mods_dir.is_dir() {
        tracing::warn!("No mods directory in instance '{}'.", args.instance);
        return Ok(());
    }

    tracing::debug!("Scanning mods directory: {}", mods_dir.display());
    let mut removed = 0u32;
    let mut removed_names: Vec<String> = Vec::new();
    let mut dirs = tokio::fs::read_dir(&mods_dir).await?;

    while let Some(entry) = dirs.next_entry().await? {
        if !entry.file_type().await?.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "nexus_mods.toml" {
            continue;
        }
        if name.to_lowercase().contains(&args.query.to_lowercase()) {
            tracing::info!("Removing mod: {}", name);
            tokio::fs::remove_file(entry.path()).await?;
            removed_names.push(name);
            removed += 1;
        }
    }

    if removed == 0 {
        tracing::info!(
            "No mods matching '{}' found in instance '{}'.",
            args.query,
            args.instance
        );
    } else {
        tracing::info!(
            "Removed {} mod(s) from instance '{}'.",
            removed,
            args.instance
        );

        // Clean up stale entries in the manifest
        if let Ok(mut manifest) = ModManifest::load(&args.instance) {
            let before = manifest.mods.len();
            manifest
                .mods
                .retain(|m| !removed_names.contains(&m.filename));
            if manifest.mods.len() < before {
                if let Err(e) = manifest.save(&args.instance) {
                    tracing::warn!("Failed to save manifest after uninstall: {}", e);
                } else {
                    tracing::info!(
                        "Cleaned {} manifest entr{}.",
                        before - manifest.mods.len(),
                        if before - manifest.mods.len() == 1 {
                            "y"
                        } else {
                            "ies"
                        }
                    );
                }
            }
        }
    }

    Ok(())
}
