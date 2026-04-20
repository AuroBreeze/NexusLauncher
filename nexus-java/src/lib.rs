pub mod java;

use crate::java::{download_java, scan_local_java_environments};
use nexus_cli::cli::JavaArgs;
use nexus_config::config::Config;
use nexus_config::models::LaunchConfig;
use nexus_core::AnyError;
use nexus_core::get_minecraft_dir;

pub async fn handle_java(args: &JavaArgs) -> Result<(), AnyError> {
    if args.download {
        let java_version = args.version;
        let custom_runtime_dir = get_minecraft_dir().join("runtimes");
        download_java(java_version, custom_runtime_dir.as_path()).await?;
    }

    if args.scan {
        tracing::info!("📦 Scanning local Java environments...");

        let mut config = LaunchConfig::load().await;
        let local_javas = scan_local_java_environments(None).await;

        tracing::info!("📦 Found {} Java environments:", local_javas.len());
        for j in local_javas {
            tracing::info!(
                "📦 Found Java {} (full version: {}) -> Path: {}",
                j.major_version,
                j.full_version,
                j.path.display()
            );

            config.java_paths.insert(j.major_version, j.path.clone());
        }

        config.save().await?;
    }

    Ok(())
}
