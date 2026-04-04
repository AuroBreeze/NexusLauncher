mod auth;
mod cli;
mod config;
mod java;
mod launch;
mod loader;
mod mode;
mod version;

use clap::Parser;
use std::path::PathBuf;
use version::AnyError;

use crate::{
    cli::{AuthArgs, JavaArgs, LaunchArgs},
    java::download_java,
};

#[tokio::main]
async fn main() -> Result<(), AnyError> {
    let cli = cli::Cli::parse();

    version::utils::init_workspace()?;
    // Initialize the logger
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    match cli.command {
        cli::Commands::Launch(args) => handle_launch(&args).await?,
        cli::Commands::Java(args) => handle_java(&args).await?,
        cli::Commands::Auth(args) => handle_auth(&args).await?,
        cli::Commands::Mode(args) => handle_mode(&args).await?,
        cli::Commands::Loader(args) => handle_loader(&args).await?,
    }

    Ok(())
}

async fn handle_launch(args: &LaunchArgs) -> Result<(), AnyError> {
    tracing::info!("Nexus Launcher Starting...");
    // Print out the configuration we are using
    tracing::info!("Target Version: {}", args.game_version);
    tracing::info!("Player Name: {}", args.player_name);
    tracing::info!("Allocated Memory: {} MB", args.max_memory);

    let manifest = version::source::obtain_manifest().await?;

    let target_version = &args.game_version;
    let required_java_version = 17;

    // Load the launcher config
    let mut launcher_config = LauncherConfig::load().await;

    #[allow(unused_assignments)]
    let mut final_java_executable: Option<PathBuf> = None;

    // Check if we already have a valid cached path for this version
    if let Some(cached_path) = launcher_config.get_valid_java(required_java_version).await
        && !args.force_scan
    {
        tracing::info!(
            "Using cached Java {}: {}",
            required_java_version,
            cached_path.display()
        );
        final_java_executable = Some(cached_path);
    } else {
        tracing::info!(
            "No valid cached Java {} found. Starting scan...",
            required_java_version
        );

        // Scan local environments
        let local_javas = java::scan_local_java_environments(None).await;

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

        if found_path.is_none() {
            tracing::warn!(
                "Java {} not found locally. Initiating automatic download...",
                required_java_version
            );

            // 1. Download and extract Java into the runtimes folder
            let custom_runtime_dir = version::utils::get_minecraft_dir().join("runtimes");
            let new_java_dir =
                java::download_java(required_java_version, &custom_runtime_dir).await?;

            // 2. Rescan the newly downloaded directory to dynamically find the exact bin/java path
            let new_javas = java::scan_local_java_environments(Some(&new_java_dir)).await;

            if let Some(j) = new_javas
                .into_iter()
                .find(|j| j.major_version == required_java_version)
            {
                found_path = Some(j.path);
            } else {
                return Err(format!(
                    "Failed to locate Java executable after downloading version {}",
                    required_java_version
                )
                .into());
            }
        }
        // ============================

        // Update the cache and save to the TOML file
        if let Some(verified_path) = found_path {
            launcher_config
                .java_paths
                .insert(required_java_version, verified_path.clone());
            launcher_config.save().await?;
            final_java_executable = Some(verified_path);
        }
    }

    if let Some(v_info) = manifest.versions.iter().find(|v| v.id == *target_version) {
        tracing::info!("Parsing data of {}...", target_version);
        let detail = version::source::fetch_version_detail(&v_info.url).await?;

        let client_jar_path = version::utils::get_clients_dir()
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
        tracing::info!("Core Path: {:?}", client_jar_path);
        tracing::info!("\nAll core components of {} are ready!", target_version);

        // launch::start_game(
        //     &detail,
        //     &client_jar_path,
        //     classpath_libs,
        //     final_java_executable.as_ref().unwrap(),
        //     &cli,
        // )?;
    }

    Ok(())
}

async fn handle_java(args: &JavaArgs) -> Result<(), AnyError> {
    if args.download {
        let java_version = args.version;
        let custom_runtime_dir = version::utils::get_minecraft_dir().join("runtimes");
        download_java(java_version, custom_runtime_dir.as_path()).await?;
    }
    if args.scan {
        tracing::info!("📦 Scanning local Java environments...");
        let local_javas = java::scan_local_java_environments(None).await;
        tracing::info!("📦 Found {} Java environments:", local_javas.len());
        for j in local_javas {
            tracing::info!(
                "📦 Found Java {} (full version: {}) -> Path: {}",
                j.major_version,
                j.full_version,
                j.path.display()
            );
        }
    }

    Ok(())
}

async fn handle_auth(args: &AuthArgs) -> Result<(), AnyError> {
    if args.login {
        //  Retrieve the device code and display it
        let device_resp = auth::utils::get_device_code().await?;
        tracing::info!(
            "Please open in your browser: {}",
            device_resp.verification_uri
        );
        tracing::info!("Enter the code: {}", device_resp.user_code);

        // Poll Microsoft Token
        let ms_token =
            auth::utils::poll_for_ms_token(&device_resp.device_code, device_resp.interval).await?;
        tracing::info!("✅ Microsoft authentication successful");

        // 3. 换取 Xbox Token
        let (xbox_token, uhs) = auth::utils::get_xbox_token(&ms_token.access_token).await?;

        // 4. 换取 XSTS Token
        let xsts_token = auth::utils::get_xsts_token(&xbox_token).await?;

        // 5. obtain Minecraft token
        let mc_token = auth::utils::get_minecraft_token(&xsts_token, &uhs).await?;
        tracing::info!("✅ Minecraft token successfully obtained!");
    }
    Ok(())
}

// TODO: will be implemented
async fn handle_mode(args: &ModeArgs) -> Result<(), AnyError> {
    if args.download {
        search_mods(&args.query).await?;
    }
    Ok(())
}

async fn handle_loader(args: &LoaderArgs) -> Result<(), AnyError> {
    let loader_verison = get_latest_loader(&args.game_version).await;
    match loader_verison {
        Ok(v) => {
            tracing::info!("Latest Fabric Loader: {}", v);
            let profile = get_fabric_profile(&args.game_version, &v).await?;
            let mut extra_classpath: Vec<PathBuf> = install_fabric_libraries(&profile).await?;

            let main_class = profile.main_class;
            tracing::info!("Main Class: {}", main_class);
            tracing::info!("Libraries: {:#?}", extra_classpath);
        }
        Err(e) => {
            tracing::error!("Failed to fetch Fabric Loader: {}", e);
        }
    }
    Ok(())
}
