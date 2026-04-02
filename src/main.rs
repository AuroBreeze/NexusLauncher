mod config;
mod java;
mod launch;
mod version;

use std::path::PathBuf;
use version::AnyError;

#[tokio::main]
async fn main() -> Result<(), AnyError> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Nexus Launcher Starting...");
    version::utils::init_workspace()?;

    let manifest = version::source::obtain_manifest().await?;

    let target_version = "1.20.1";
    let required_java_version = 17;

    // Load the launcher config
    let mut launcher_config = config::LauncherConfig::load().await;
    let mut final_java_executable: Option<PathBuf> = None;

    // Check if we already have a valid cached path for this version
    if let Some(cached_path) = launcher_config.get_valid_java(required_java_version).await {
        tracing::info!("Using cached Java {}: {}", required_java_version, cached_path.display());
        final_java_executable = Some(cached_path);
    } else {
        tracing::info!(
            "No valid cached Java {} found. Starting scan...",
            required_java_version
        );

        // Scan local environments
        let custom_runtime_dir = version::utils::get_minecraft_dir().join("runtimes");
        let local_javas = java::scan_local_java_environments(Some(&custom_runtime_dir)).await;

        let mut found_path = None;
        for j in local_javas {
            tracing::info!(
                "📦 Found Java {} (full version: {}) -> Path: {}",
                j.major_version,
                j.full_version,
                j.path.display()
            );

            if j.major_version == required_java_version {
                tracing::info!(
                    "Found matching Java {}: {}",
                    required_java_version,
                    j.path.display()
                );
                found_path = Some(j.path);
                break;
            }
        }

        // Handle the case where scanning still fails (e.g., prompt user or trigger download)
        if let Some(verified_path) = found_path {
            // Update the cache and save to the TOML file
            launcher_config
                .java_paths
                .insert(required_java_version, verified_path.clone());
            launcher_config.save().await?;

            final_java_executable = Some(verified_path);
        } else {
            return Err(format!("Could not find Java {}", required_java_version).into());
        }
    }

    if let Some(v_info) = manifest.versions.iter().find(|v| v.id == target_version) {
        tracing::info!("Parsing data of {}...", target_version);
        let detail = version::source::fetch_version_detail(&v_info.url).await?;

        let client_jar_path = version::utils::get_minecraft_dir()
            .join("versions")
            .join(target_version)
            .join(format!("{}.jar", target_version));

        if !client_jar_path.exists() {
            tracing::info!("Downloading core files...");
            version::download::download_and_verify(
                &detail.downloads.client.url,
                &client_jar_path,
                detail.downloads.client.sha1.as_str(),
            )
            .await?;
        }

        let classpath_libs = version::source::download_libraries(&detail).await?;

        version::source::download_assets(&detail).await?;
        tracing::info!("\nAll core components of {} are ready!", target_version);
        tracing::info!("Core Path: {:?}", client_jar_path);

        launch::start_game(&detail, &client_jar_path, classpath_libs, "AuroBreeze", final_java_executable.as_ref().unwrap())?;
    }

    Ok(())
}
