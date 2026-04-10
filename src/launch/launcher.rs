use crate::launch::models::LaunchContext;
use crate::version::AnyError;
use crate::version::utils::{self, get_clients_dir};
use std::process::Command;

pub fn start_game(launch_context: LaunchContext) -> Result<(), AnyError> {
    tracing::info!("Assembling startup parameters...");

    // 1. Build the Classpath (on Linux, you must use a colon : to connect)
    let mut cp_paths: Vec<String> = launch_context
        .libraries
        .into_iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    // Also add the game's core (client.jar) to the Classpath
    cp_paths.push(launch_context.core_jar.to_string_lossy().to_string());

    let classpath = cp_paths.join(":");

    // 2. Obtain the base path
    let mc_dir = utils::get_minecraft_dir();
    let assets_dir = mc_dir.join("assets");

    // Calculate the exclusive isolation directory for this version
    let version_isolated_dir = get_clients_dir().join(&launch_context.version_id);

    // Ensure that the isolation directory exists (it is usually created when downloading client.jar, this is just a precaution here).
    if !version_isolated_dir.exists() {
        std::fs::create_dir_all(&version_isolated_dir)?;
    }

    // 3. Build and execute Java commands
    if launch_context.java_path.is_none() {
        tracing::error!("Java executable not found");
        return Err("Java executable not found".into());
    }
    let mut cmd = Command::new(launch_context.java_path.unwrap());

    // === A. JVM Runtime Parameters ===
    if let Some(max_memory) = launch_context.max_memory {
        let max_memory = format!("-Xmx{}M", max_memory);
        cmd.arg(max_memory);
    }

    cmd.arg("-XX:+UseG1GC");
    cmd.arg("-cp").arg(classpath);
    // let main_class = "net.fabricmc.loader.impl.launch.knot.KnotClient";
    // cmd.arg(main_class);
    cmd.arg(launch_context.main_class);

    // === B. Core Game Parameters ===

    cmd.arg("--username").arg(launch_context.user.username);
    cmd.arg("--version").arg(launch_context.version_id);

    // Point gameDir to the version-specific isolated directory!
    cmd.arg("--gameDir").arg(&version_isolated_dir);

    // Keep assetsDir unchanged and continue using the shared global resource library
    cmd.arg("--assetsDir").arg(&assets_dir);

    cmd.arg("--assetIndex").arg(launch_context.asset_index_id);
    cmd.arg("--uuid").arg(launch_context.user.uuid);

    if let Some(access_token) = &launch_context.user.access_token {
        cmd.arg("--accessToken").arg(access_token);
    }

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
