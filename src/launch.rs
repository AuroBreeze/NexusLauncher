// src/launch.rs
use crate::cli::LaunchArgs;
use crate::version::AnyError;
use crate::version::models::VersionDetail;
use crate::version::utils::{self, get_clients_dir};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn start_game(
    detail: &VersionDetail,
    client_jar: &Path,
    libraries: Vec<PathBuf>,
    java_executable: &Path,
    cli: LaunchArgs,
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
    cmd.arg("--username").arg(cli.player_name.clone());
    cmd.arg("--version").arg(cli.game_version.clone());

    // Point gameDir to the version-specific isolated directory!
    cmd.arg("--gameDir").arg(&version_isolated_dir);

    // Keep assetsDir unchanged and continue using the shared global resource library
    cmd.arg("--assetsDir").arg(&assets_dir);

    cmd.arg("--assetIndex").arg(&detail.asset_index.id);
    cmd.arg("--uuid")
        .arg("00000000-0000-0000-0000-000000000000");
    cmd.arg("--accessToken").arg("offline_token");
    cmd.arg("--userType").arg("mojang");
    cmd.arg("--versionType").arg("release");
    tracing::info!("Execute command: {:?}", cmd);

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
