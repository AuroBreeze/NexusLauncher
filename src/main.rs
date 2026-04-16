mod auth;
mod cli;
mod config;
mod java;
mod launch;
mod loader;
mod mods;
mod version;

use clap::Parser;
use std::path::PathBuf;
use version::AnyError;

use crate::auth::handle_auth;
use crate::cli::{Commands, InstallCommands};
use crate::config::config::Config;
use crate::config::handle_set;
use crate::config::models::LaunchConfig;
use crate::launch::models::{LaunchContext, UserContext};
use crate::loader::fabric::find_fabric_json;
use crate::loader::handle_loader;
use crate::loader::models::FabricProfile;
use crate::mods::handle_mods;
use crate::version::models::VersionDetail;
use crate::version::utils::{get_clients_dir, get_library_path};
use crate::version::verify_game_integrity;
use crate::{
    auth::utils::silent_login,
    cli::{JavaArgs, LaunchArgs},
    config::models::UserConfig,
    java::download_java,
    launch::launcher::start_game,
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
        // Handling top-level commands
        Some(Commands::Launch(args)) => handle_launch(&args).await?,
        Some(Commands::Java(args)) => handle_java(&args).await?,
        Some(Commands::Auth(args)) => handle_auth(&args).await?,

        // Handling nested Install command
        Some(Commands::Install(install_args)) => match install_args.command {
            InstallCommands::Mod(mod_args) => handle_mods(&mod_args).await?,
            InstallCommands::Loader(loader_args) => handle_loader(&loader_args).await?,
            InstallCommands::Core(core_args) => handle_core(&core_args).await?,
        },

        Some(Commands::Set(args)) => handle_set(&args).await?,

        // Handling the case where no command is provided
        None => {
            println!("Please specify a command. Use --help");
        }
    }

    Ok(())
}

async fn handle_core(args: &cli::CoreArgs) -> Result<(), AnyError> {
    if let Some(game_version) = &args.game_version {
        let target_version = game_version;

        let version_dir = version::utils::get_clients_dir().join(target_version);
        let local_json_path = version_dir.join("version.json");
        if local_json_path.exists() {
            tracing::warn!(
                "The game instance already exists. Please rename the existing instance (if you do not rename it, the default name will be the game version)."
            );
            return Ok(());
        }

        let detail = {
            // fetch
            let manifest = version::source::obtain_manifest().await?;

            let v_info = manifest
                .versions
                .iter()
                .find(|v| v.id == *target_version)
                .ok_or_else(|| format!("Version {} not found in manifest", target_version))?;

            tracing::info!("Fetching version data for {}...", target_version);
            let d = version::source::fetch_version_detail(&v_info.url).await?;

            // Ensure the directory exists and save the JSON for future offline use
            tokio::fs::create_dir_all(&version_dir).await?;
            let json_content = serde_json::to_string_pretty(&d)?;
            tokio::fs::write(&local_json_path, json_content).await?;

            d
        };

        // Verify and download the client JAR
        let client_jar_path = version::utils::get_clients_dir()
            .join(target_version)
            .join(format!("{}.jar", target_version));

        if !client_jar_path.exists() {
            tracing::info!("Downloading core JAR file...");
            version::download::download_and_verify(
                &detail.downloads.client.url,
                &client_jar_path,
                detail.downloads.client.sha1.as_str(),
            )
            .await?;
        }

        version::source::download_libraries(&detail).await?;
        version::source::download_assets(&detail).await?;

        tracing::info!("All core components for {} are ready!", target_version);
    }

    Ok(())
}

async fn handle_launch(args: &LaunchArgs) -> Result<(), AnyError> {
    tracing::info!("Nexus Launcher Starting...");
    // Print out the configuration we are using

    let required_java_version = 17;

    // Load the launcher and user config
    let user_config = UserConfig::load().await;
    let mut launcher_config = LaunchConfig::load().await;

    #[allow(unused_assignments)]
    let mut final_java_executable: Option<PathBuf> = None;

    // Check if we already have a valid cached path for this version
    // PERF: The code for locating, saving, and downloading Java files, as well as the code for launching the game, should remain concise. Move the code to `java.rs` and reuse it.
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

    // Identity and Access Token Handling
    let access_token;
    let (username, uuid);

    // PERF: Optimize the code here
    // TODO: Add the usercache.json file from the game instance and synchronize the game's access_token
    if let Some(offline) = args.offline {
        if offline {
            access_token = "offline_token".to_string();
            username = if user_config.user_profile.offline.username.is_empty() {
                "Default".to_string()
            } else {
                user_config.user_profile.offline.username.clone()
            };

            uuid = if user_config.user_profile.offline.uuid.is_empty() {
                "offline".to_string()
            } else {
                user_config.user_profile.offline.uuid.clone()
            };

            tracing::info!("Mode: Offline (User: {}, UUID: {})", username, uuid);
        } else {
            username = user_config.user_profile.online.username.clone();
            uuid = user_config.user_profile.online.uuid.clone();
            access_token = silent_login(&uuid).await?;
            tracing::info!("Mode: Online (User: {})", username);
        }
    } else {
        if launcher_config.offline {
            access_token = "offline_token".to_string();
            username = if user_config.user_profile.offline.username.is_empty() {
                "Default".to_string()
            } else {
                user_config.user_profile.offline.username.clone()
            };

            uuid = if user_config.user_profile.offline.uuid.is_empty() {
                "offline".to_string()
            } else {
                user_config.user_profile.offline.uuid.clone()
            };

            tracing::info!("Mode: Offline (User: {}, UUID: {})", username, uuid);
        } else {
            username = user_config.user_profile.online.username.clone();
            uuid = user_config.user_profile.online.uuid.clone();
            access_token = silent_login(&uuid).await?;
            tracing::info!("Mode: Online (User: {})", username);
        }
    }

    let game_path = &get_clients_dir().join(&args.instance_name);
    let game_version_json_path = game_path.join("version.json");

    // 1. Convert the initial Result to Option using .ok()
    // 2. Use .flatten() if find_fabric_json returns Result<Option<P>, E>
    let fabric_profile = find_fabric_json(game_path)
        .ok() // Result -> Option<Option<PathBuf>> (based on your first snippet)
        .flatten() // Option<Option<PathBuf>> -> Option<PathBuf>
        .and_then(|path| std::fs::read_to_string(path).ok()) // Try read file
        .and_then(|data| serde_json::from_str::<FabricProfile>(&data).ok()); // Try parse JSON
    // dbg!(&fabric_profile);

    let data = std::fs::read_to_string(game_version_json_path).unwrap();
    let detail: VersionDetail = serde_json::from_str(&data).unwrap();
    let version_id = detail.id;
    let client_jar_path = get_clients_dir()
        .join(&args.instance_name)
        .join(format!("{}.jar", &version_id));

    verify_game_integrity(game_path).await?;

    // TODO: Optimize LaunchContext by removing duplicate components from the assembly, such as client_core_jar, and use game_path for assembly at the start_game stage.
    //
    // Construct the launch context and start the process
    let launch_context = LaunchContext {
        game_path: PathBuf::from(game_path),
        version_id,
        java_path: final_java_executable,
        core_jar: client_jar_path,
        user: UserContext {
            username,
            uuid,
            access_token: Some(access_token),
        },
        max_memory: Some(args.max_memory),
        main_class: detail.main_class.clone(),
        libraries: detail
            .libraries
            .iter()
            .filter_map(|lib| {
                // Use filter_map to safely handle the Option and avoid unwrap()
                let artifact = lib.downloads.artifact.as_ref()?;
                Some(get_library_path(&artifact.path))
            })
            .collect(),
        asset_index_id: detail.asset_index.id.clone(),
        fabric_loader: fabric_profile,
    };

    start_game(launch_context)?;

    Ok(())
}

// TODO: Migrate code
async fn handle_java(args: &JavaArgs) -> Result<(), AnyError> {
    if args.download {
        let java_version = args.version;
        let custom_runtime_dir = version::utils::get_minecraft_dir().join("runtimes");
        download_java(java_version, custom_runtime_dir.as_path()).await?;
        test
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
