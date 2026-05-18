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

use nexus_version::models::VersionDetail;
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
        let dir_name = args.name.as_deref().unwrap_or(game_version);
        nexus_version::install_game_core(game_version, dir_name).await?;
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
    let required_java_version = detail.java_version.major_version as u32;

    verify_game_integrity(game_path).await?;

    let final_java_executable =
        resolve_java_executable(required_java_version, args.force_scan, &mut launcher_config)
            .await?;

    // Construct the launch context and start the process
    let user = UserContext::new(username, uuid, access_token);
    let launch_context = LaunchContext::from_detail(
        PathBuf::from(game_path),
        &detail,
        user,
        final_java_executable,
        args.max_memory,
        fabric_profile,
    );

    let monitor = start_game(launch_context)?;

    // TODO: Use OS-level file redirection (Stdio::from) so the game
    // output survives launcher exit. Then the monitor thread can be
    // fully detached and the CLI returns immediately — crash detection
    // would be handled by a separate `check-crash` command that reads
    // the persisted output file.
    //
    // Block until the game exits — the game runs as an independent OS
    // process, so this only blocks the launcher's monitor thread.
    // If the game crashes, stderr is captured and written to
    // clients/<instance>/crash-<timestamp>.log by the monitor.
    let _ = monitor.join();
    Ok(())
}
