use nexus_cli::cli::ListInfoArgs;
use nexus_config::config::Config;
use nexus_config::models::{LaunchConfig, UserConfig};
use nexus_core::{AnyError, UserCacheEntry, get_clients_dir, validate_instance_name};
use nexus_loader::fabric::find_fabric_json;
use nexus_loader::models::FabricProfile;
use nexus_version::models::VersionDetail;

pub async fn handle_list_instances() -> Result<(), AnyError> {
    let clients_dir = get_clients_dir();

    if !clients_dir.exists() {
        tracing::info!("No game instances found. Download a game version first.");
        return Ok(());
    }

    let mut dirs = tokio::fs::read_dir(&clients_dir).await?;
    let mut instances: Vec<String> = Vec::new();

    while let Some(entry) = dirs.next_entry().await? {
        if entry.file_type().await?.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            instances.push(name);
        }
    }

    if instances.is_empty() {
        tracing::info!("No game instances found in {}", clients_dir.display());
        return Ok(());
    }

    instances.sort();
    tracing::info!(
        "Found {} instance(s) in {}:",
        instances.len(),
        clients_dir.display()
    );
    for inst in &instances {
        let version_path = clients_dir.join(inst).join("version.json");
        if version_path.exists()
            && let Ok(data) = tokio::fs::read_to_string(&version_path).await
            && let Ok(detail) = serde_json::from_str::<VersionDetail>(&data)
        {
            tracing::info!(
                "  {} (version: {}, java: {})",
                inst,
                detail.id,
                detail.java_version.major_version,
            );
        } else if version_path.exists() {
            tracing::warn!("  {} (version.json parse failed)", inst);
        } else {
            tracing::info!("  {}", inst);
        }
    }

    Ok(())
}

pub async fn handle_list_users() -> Result<(), AnyError> {
    let config = UserConfig::load().await;

    tracing::info!("Online profile:");
    if config.user_profile.online.username.is_empty() {
        tracing::info!("  (not configured)");
    } else {
        tracing::info!("  username: {}", config.user_profile.online.username);
        tracing::info!("  uuid:     {}", config.user_profile.online.uuid);
    }

    tracing::info!("Offline profile:");
    if config.user_profile.offline.username.is_empty() {
        tracing::info!("  (not configured)");
    } else {
        tracing::info!("  username: {}", config.user_profile.offline.username);
        tracing::info!("  uuid:     {}", config.user_profile.offline.uuid);
    }

    let launch_config = LaunchConfig::load().await;
    tracing::info!(
        "Offline mode: {}",
        if launch_config.offline {
            "enabled"
        } else {
            "disabled"
        }
    );

    if !config.username.is_empty() {
        tracing::info!("Cached username→uuid mappings:");
        for (name, uuid) in &config.username {
            tracing::info!("  {} -> {}", name, uuid);
        }
    }

    Ok(())
}

pub async fn handle_list_info(args: &ListInfoArgs) -> Result<(), AnyError> {
    validate_instance_name(&args.instance)?;

    let inst_path = get_clients_dir().join(&args.instance);

    if !inst_path.exists() || !inst_path.is_dir() {
        tracing::warn!("Instance '{}' does not exist.", args.instance);
        return Ok(());
    }

    tracing::info!("Instance: {}", args.instance);

    // Version info
    let version_path = inst_path.join("version.json");
    if version_path.exists() {
        let data = tokio::fs::read_to_string(&version_path).await?;
        let detail: VersionDetail = serde_json::from_str(&data)?;
        tracing::info!("  version:        {}", detail.id);
        tracing::info!("  java version:   {}", detail.java_version.major_version);
        tracing::info!("  main class:     {}", detail.main_class);
        tracing::info!("  type:           {}", detail.type_);
        tracing::info!("  libraries:      {}", detail.libraries.len());
        tracing::info!("  asset index:    {}", detail.asset_index.id);
    } else {
        tracing::info!("  version.json:   (not found)");
    }

    // Fabric loader
    let fabric_path = find_fabric_json(&inst_path).ok().flatten();
    let loader_type = fabric_path.as_ref().and_then(|p| {
        let stem = p.file_stem()?.to_str()?;
        if stem.starts_with("fabric") {
            Some("fabric".to_string())
        } else if stem.starts_with("quilt") {
            Some("quilt".to_string())
        } else {
            None
        }
    });

    let fabric_profile = fabric_path
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|data| serde_json::from_str::<FabricProfile>(&data).ok());

    if let Some(ltype) = &loader_type {
        if args.loader {
            tracing::info!("  loader:         {}", ltype);
        } else {
            tracing::info!(
                "  loader:         {} ({} libraries)",
                ltype,
                fabric_profile.as_ref().map_or(0, |p| p.libraries.len())
            );
        }
    }

    // Mods
    let mods_dir = inst_path.join("mods");
    if mods_dir.exists() && mods_dir.is_dir() {
        let mut files: Vec<String> = Vec::new();
        if let Ok(mut dirs) = tokio::fs::read_dir(&mods_dir).await {
            while let Some(entry) = dirs.next_entry().await? {
                let name = entry.file_name().to_string_lossy().to_string();
                if entry.file_type().await?.is_file() && name != "nexus_mods.toml" {
                    files.push(name);
                }
            }
        }
        if !files.is_empty() {
            files.sort();
            if args.mods {
                tracing::info!("  mods:           {} file(s)", files.len());
                for f in &files {
                    tracing::info!("    - {}", f);
                }
            } else {
                tracing::info!("  mods:           {} file(s)", files.len());
            }
        }
    }

    // User cache
    let cache_path = inst_path.join("usercache.json");
    if cache_path.exists() {
        let content = tokio::fs::read_to_string(&cache_path).await?;
        if let Ok(entries) = serde_json::from_str::<Vec<UserCacheEntry>>(&content) {
            tracing::info!("  cached users:   {}", entries.len());
            for entry in &entries {
                tracing::info!("    - {} ({})", entry.name, entry.uuid);
            }
        }
    }

    Ok(())
}
