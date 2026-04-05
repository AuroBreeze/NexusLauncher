// src/launch.rs

use crate::auth::storage::get_refresh_token;
use crate::auth::utils::refresh_ms_token;
use crate::cli::LaunchArgs;
use crate::config::models::LauncherConfig;
use crate::version::models::VersionDetail;
use crate::version::utils::{self, get_clients_dir};
use crate::version::AnyError;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn start_game(
    detail: &VersionDetail,
    client_jar: &Path,
    libraries: Vec<PathBuf>,
    java_executable: &Path,
    cli: &LaunchArgs,
    access_token: &str,
) -> Result<(), AnyError> {
    tracing::info!("Assembling startup parameters...");

    // 1. Build the Classpath (on Linux, you must use a colon : to connect)
    let mut cp_paths: Vec<String> = libraries
        .into_iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    // Also add the game's core (client.jar) to the Classpath
    cp_paths.push(client_jar.to_string_lossy().to_string());

    let classpath = cp_paths.join(":");

    // 2. Obtain the base path
    let mc_dir = utils::get_minecraft_dir();
    let assets_dir = mc_dir.join("assets");

    // Calculate the exclusive isolation directory for this version
    let version_isolated_dir = get_clients_dir().join(&detail.id);

    // Ensure that the isolation directory exists (it is usually created when downloading client.jar, this is just a precaution here).
    if !version_isolated_dir.exists() {
        std::fs::create_dir_all(&version_isolated_dir)?;
    }

    // 3. Build and execute Java commands
    let mut cmd = Command::new(java_executable);

    // === A. JVM Runtime Parameters ===
    let max_memory = format!("-Xmx{}M", cli.max_memory);

    cmd.arg(max_memory);
    cmd.arg("-XX:+UseG1GC");
    cmd.arg("-cp").arg(classpath);
    cmd.arg(&detail.main_class);

    // === B. Core Game Parameters ===
    let path = LauncherConfig::get_config_path();
    let content = fs::read_to_string(path)?;
    let config: LauncherConfig = toml::from_str(&content)?;

    let username = &config.user_profile.online.username;
    let uuid = &config.user_profile.online.username;

    cmd.arg("--username").arg(username);
    cmd.arg("--version").arg(cli.game_version.clone());

    // Point gameDir to the version-specific isolated directory!
    cmd.arg("--gameDir").arg(&version_isolated_dir);

    // Keep assetsDir unchanged and continue using the shared global resource library
    cmd.arg("--assetsDir").arg(&assets_dir);

    cmd.arg("--assetIndex").arg(&detail.asset_index.id);
    cmd.arg("--uuid").arg(uuid);
    cmd.arg("--accessToken").arg(access_token);
    cmd.arg("--userType").arg("mojang");
    cmd.arg("--versionType").arg("release");

    // protect the access token
    let args_preview: Vec<String> = cmd
        .get_args()
        .map(|arg| {
            let s = arg.to_string_lossy();
            // If the parameter is `accessToken`, hide its contents
            if s.len() > 20 && !s.contains('/') {
                "********".to_string()
            } else {
                s.into_owned()
            }
        })
        .collect();

    tracing::info!("Execute command: {:?}", args_preview);

    // 4. Start a child process
    let mut child = cmd.spawn()?;
    tracing::info!(
        "🚀 The game has successfully started! Process PID: {}",
        child.id()
    );

    // Let the launcher wait for the game to end. If you don't write this line, the launcher will exit immediately after the popup appears.
    let status = child.wait()?;
    tracing::info!("The game has exited, status code: {}", status);

    Ok(())
}
