use clap::Parser;
use std::path::PathBuf;

use nexus_auth::handle_auth;
use nexus_auth::utils::silent_login;

use nexus_cli::cli::*;

use nexus_config::config::Config;
use nexus_config::handle_set;
use nexus_config::models::{LaunchConfig, UserConfig};

use nexus_java::handle_java;
use nexus_java::java::resolve_java_executable;

use nexus_launch::launcher::start_game;
use nexus_launch::models::{LaunchContext, UserContext};

use nexus_loader::fabric::find_fabric_json;
use nexus_loader::handle_loader;
use nexus_loader::models::FabricProfile;

use nexus_mods::handle_mods;

use nexus_core::*;
use nexus_list::{handle_list_info, handle_list_instances, handle_list_users};
use nexus_search::{
    handle_search_core, handle_search_java, handle_search_loader, handle_search_mod,
    handle_search_user,
};
use nexus_uninstall::{handle_uninstall_instance, handle_uninstall_mod};

use nexus_version::download::download_and_verify;
use nexus_version::models::VersionDetail;
use nexus_version::source::{
    download_assets, download_libraries, fetch_version_detail, obtain_manifest,
};
use nexus_version::verify_game_integrity;

#[tokio::main]
async fn main() -> Result<(), AnyError> {
    let cli = Cli::parse();

    init_workspace()?;
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

        Some(Commands::Search(args)) => match args.command {
            SearchCommands::Mod(search_args) => handle_search_mod(&search_args).await?,
            SearchCommands::Java(search_args) => handle_search_java(&search_args).await?,
            SearchCommands::Core(search_args) => handle_search_core(&search_args).await?,
            SearchCommands::Loader(search_args) => handle_search_loader(&search_args).await?,
            SearchCommands::User(search_args) => handle_search_user(&search_args).await?,
        },

        Some(Commands::List(args)) => match args.command {
            ListCommands::Instances => handle_list_instances().await?,
            ListCommands::Users => handle_list_users().await?,
            ListCommands::Info(info_args) => handle_list_info(&info_args).await?,
        },

        Some(Commands::Uninstall(args)) => match args.command {
            UninstallCommands::Instance(inst_args) => handle_uninstall_instance(&inst_args).await?,
            UninstallCommands::Mod(mod_args) => handle_uninstall_mod(&mod_args).await?,
        },

        Some(Commands::Set(args)) => handle_set(&args).await?,

        // Handling the case where no command is provided
        None => {
            println!("Please specify a command. Use --help");
        }
    }

    Ok(())
}

async fn handle_core(args: &CoreArgs) -> Result<(), AnyError> {
    if let Some(game_version) = &args.game_version {
        let target_version = game_version;

        // Use --name if provided, otherwise fall back to the game version
        let dir_name = args.name.as_deref().unwrap_or(target_version);
        let version_dir = get_clients_dir().join(dir_name);
        let local_json_path = version_dir.join("version.json");

        if local_json_path.exists() {
            // Directory already exists — verify integrity and re-download any missing/corrupt files
            tracing::info!(
                "Game directory '{}' already exists, verifying integrity...",
                dir_name
            );
            verify_game_integrity(&version_dir).await?;
        } else {
            // Fresh download
            let manifest = obtain_manifest().await?;

            let v_info = manifest
                .versions
                .iter()
                .find(|v| v.id == *target_version)
                .ok_or_else(|| format!("Version {} not found in manifest", target_version))?;

            tracing::info!("Fetching version data for {}...", target_version);
            let detail = fetch_version_detail(&v_info.url).await?;

            tokio::fs::create_dir_all(&version_dir).await?;

            download_and_verify(
                &detail.downloads.client.url,
                &version_dir.join(format!("{}.jar", target_version)),
                detail.downloads.client.sha1.as_str(),
            )
            .await?;

            download_libraries(&detail).await?;
            download_assets(&detail).await?;

            // Save version.json last — it serves as the "download complete" sentinel.
            // If interrupted before this point, the directory won't have version.json,
            // so the next attempt will re-download (skipping already-valid files via SHA1 check).
            let json_content = serde_json::to_string_pretty(&detail)?;
            tokio::fs::write(&local_json_path, json_content).await?;
        }

        tracing::info!("All core components for {} are ready!", target_version);
    }

    Ok(())
}

async fn handle_launch(args: &LaunchArgs) -> Result<(), AnyError> {
    tracing::info!("Nexus Launcher Starting...");
    // Print out the configuration we are using

    // Load the launcher and user config
    let user_config = UserConfig::load().await;
    let mut launcher_config = LaunchConfig::load().await;

    // Identity and Access Token Handling
    let is_offline = args.offline.unwrap_or(launcher_config.offline);

    let (access_token, username, uuid) = if is_offline {
        let username = if user_config.user_profile.offline.username.is_empty() {
            "Default".to_string()
        } else {
            user_config.user_profile.offline.username.clone()
        };
        let uuid = if user_config.user_profile.offline.uuid.is_empty() {
            "offline".to_string()
        } else {
            user_config.user_profile.offline.uuid.clone()
        };
        tracing::info!("Mode: Offline (User: {}, UUID: {})", username, uuid);
        ("offline_token".to_string(), username, uuid)
    } else {
        let username = user_config.user_profile.online.username.clone();
        let uuid = user_config.user_profile.online.uuid.clone();
        let access_token = silent_login(&uuid).await?;
        tracing::info!("Mode: Online (User: {})", username);
        (access_token, username, uuid)
    };

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

    let data = std::fs::read_to_string(&game_version_json_path).map_err(|_| {
        format!(
            "Instance '{}' not found. Run `nexus install core --game-version <version>` to install it.",
            args.instance_name
        )
    })?;
    let detail: VersionDetail =
        serde_json::from_str(&data).map_err(|e| format!("Failed to parse version.json: {}", e))?;
    let version_id = detail.id;
    let required_java_version = detail.java_version.major_version as u32;

    verify_game_integrity(game_path).await?;

    let final_java_executable =
        resolve_java_executable(required_java_version, args.force_scan, &mut launcher_config)
            .await?;

    // Construct the launch context and start the process
    let launch_context = LaunchContext {
        game_path: PathBuf::from(game_path),
        version_id,
        java_path: Some(final_java_executable),
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
