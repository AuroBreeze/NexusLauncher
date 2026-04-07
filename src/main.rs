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

use crate::config::config::Config;
use crate::config::models::LaunchConfig;
use crate::launch::models::{LaunchContext, UserContext};
use crate::{
    auth::utils::silent_login,
    cli::{AuthArgs, JavaArgs, LaunchArgs, LoaderArgs, ModeArgs, SetArgs},
    config::models::UserConfig,
    java::download_java,
    launch::launcher::start_game,
    loader::fabric::{get_fabric_profile, get_latest_loader, install_fabric_libraries},
    mode::models::search_mods,
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
        Some(cli::Commands::Launch(args)) => handle_launch(&args).await?,
        Some(cli::Commands::Java(args)) => handle_java(&args).await?,
        Some(cli::Commands::Auth(args)) => handle_auth(&args).await?,
        Some(cli::Commands::Mode(args)) => handle_mode(&args).await?,
        Some(cli::Commands::Loader(args)) => handle_loader(&args).await?,
        Some(cli::Commands::Set(args)) => handle_set(&args).await?,

        None => {
            println!("Please specify a command. Use --help");
        }
    }

    Ok(())
}

async fn handle_launch(args: &LaunchArgs) -> Result<(), AnyError> {
    tracing::info!("Nexus Launcher Starting...");
    // Print out the configuration we are using

    let manifest = version::source::obtain_manifest().await?;
    let target_version = &args.game_version;
    let required_java_version = 17;

    // Load the launcher and user config
    let mut user_config = UserConfig::load().await;
    let mut launcher_config = LaunchConfig::load().await;

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
            let java_bin = if cfg!(target_os = "windows") {
                "java.exe"
            } else {
                "java"
            };

            use walkdir::WalkDir;

            let mut found = None;

            for entry in WalkDir::new(&new_java_dir) {
                let entry = entry?;
                if entry.file_type().is_file() {
                    let name = entry.file_name().to_string_lossy().to_lowercase();

                    if name == java_bin {
                        found = Some(entry.path().to_path_buf());
                        break;
                    }
                }
            }

            if let Some(java_path) = found {
                tracing::info!(
                    "✅ Found downloaded Java {} at {}",
                    required_java_version,
                    java_path.display()
                );
                found_path = Some(java_path);
            } else {
                return Err(format!(
                    "Java downloaded but executable not found in {:?}",
                    new_java_dir
                )
                .into());
            }
        }

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

        let access_token;
        let mut username;
        let mut uuid;
        if launcher_config.offline {
            access_token = "offline_token".to_string();
            username = user_config.user_profile.offline.username.clone();
            uuid = user_config.user_profile.offline.uuid.clone();
            if username.is_empty() {
                username = "Default".to_string();
            }
            if uuid.is_empty() {
                uuid = "offline".to_string();
            }

            tracing::info!("offline name: {}", username);
            tracing::info!("offline uuid: {}", uuid);
        } else {
            username = user_config.user_profile.online.username.clone();
            uuid = user_config.user_profile.online.uuid.clone();
            access_token = silent_login(&uuid).await?;
        }

        let mut launch_context = LaunchContext {
            version_id: args.game_version.clone(),
            offline: launcher_config.offline,
            java_path: final_java_executable,
            core_jar: client_jar_path,
            user: UserContext {
                username: username,
                uuid: uuid,
                access_token: Some(access_token),
            },
            max_memory: Some(args.max_memory),

            main_class: detail.main_class.clone(),
            libraries: classpath_libs,
            asset_index_id: detail.asset_index.id.clone(),
        };

        tracing::info!("Target Version: {}", launch_context.version_id);
        tracing::info!("Player Name: {}", launch_context.user.username);
        tracing::info!("Player UUID: {}", launch_context.user.uuid);
        tracing::info!(
            "Allocated Memory: {} MB",
            launch_context.max_memory.unwrap()
        );

        start_game(launch_context)?;
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

        let mut config = LaunchConfig::load().await;
        let local_javas = java::scan_local_java_environments(None).await;

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

        // 3. Obtain Xbox Token
        let (xbox_token, uhs) = auth::utils::get_xbox_token(&ms_token.access_token).await?;

        // 4. Obtain XSTS Token
        let xsts_token = auth::utils::get_xsts_token(&xbox_token).await?;

        // 5. obtain Minecraft token
        let mc_token = auth::utils::get_minecraft_token(&xsts_token, &uhs).await?;
        tracing::info!("✅ Minecraft token successfully obtained!");
        // 6. check game ownership
        tracing::info!("Verifying game ownership...");
        let is_owner = auth::utils::check_ownership(&mc_token).await?;

        if !is_owner {
            tracing::error!(
                "❌ Verification failed: This account has not purchased Minecraft for Java Edition!"
            );
            return Err("Account does not own the game".into());
        }
        tracing::info!("✅ Permission verified: You own the game");

        // 7. Get player profile (UUID and nickname)
        let profile = auth::utils::get_minecraft_profile(&mc_token).await?;
        tracing::info!("🚀 Login successful! Welcome back, {}!", profile.name);
        tracing::info!("Player UUID: {}", profile.id);

        if let Some(refresh_token) = ms_token.refresh_token {
            // Use the player's UUID or nickname as the key
            auth::storage::save_refresh_token(&profile.id, &refresh_token)?;
            tracing::info!(
                "✅ The security credentials have been encrypted and stored in the system credential manager."
            );
        }

        let mut config = UserConfig::load().await;
        config.user_profile.online.username = profile.name.clone();
        config.user_profile.online.uuid = profile.id.clone();

        config
            .username
            .insert(profile.name.clone(), profile.id.clone());

        config.save().await?;
        tracing::info!("✅ Username has been saved in the launcher config.");
    }

    if !args.logout.is_empty() {
        let name = &args.logout;
        let mut config = UserConfig::load().await;
        let uuid = match config.username.get(name) {
            Some(u) => u,
            None => {
                return Err(format!("User {} not found", name).into());
            }
        };
        // TODO: Also need to remove user_profile.online and username from the config file
        auth::storage::delete_token(uuid).unwrap();
        config.username.remove(name);
        config.user_profile.online.username = "".to_string();
        config.user_profile.online.uuid = "".to_string();
        config.save().await.unwrap();
        tracing::info!(
            "✅ Security credentials have been deleted from the system credential manager."
        );
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

async fn handle_set(args: &SetArgs) -> Result<(), AnyError> {
    let mut config = UserConfig::load().await;
    let mut launch_config = LaunchConfig::load().await;

    if let Some(username) = args.name.as_ref() {
        config.user_profile.offline.username = username.clone();
    }

    if let Some(uuid) = args.uuid.as_ref() {
        config.user_profile.offline.uuid = uuid.clone();
    }

    if let Some(judge) = args.offline {
        launch_config.offline = judge;
    }

    if args.show {
        tracing::info!("Offline profile: {:#?}", config.user_profile.offline);
    }

    config.save().await?;
    launch_config.save().await?;
    Ok(())
}
