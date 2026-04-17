use crate::launch::models::LaunchContext;
use crate::version::AnyError;
use crate::version::utils::{self, maven_to_path};
use std::process::Command;

pub fn start_game(launch_context: LaunchContext) -> Result<(), AnyError> {
    tracing::info!("Assembling startup parameters...");

    // 0. Validation
    let java_path = launch_context
        .java_path
        .as_ref()
        .ok_or("Java executable not found")?;

    let mut cmd = Command::new(java_path);

    // Build the Classpath
    // PERF: Migrate the code and optimize the code structure
    #[cfg(target_os = "windows")]
    let sep = ";";
    #[cfg(not(target_os = "windows"))]
    let sep = ":";

    let mut cp_paths: Vec<String> = launch_context
        .libraries
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    // Determine Main Class and add Fabric libraries if needed
    let final_main_class = if let Some(fabric_profile) = &launch_context.fabric_loader {
        tracing::info!("Fabric profile found!");
        let libs_base_dir = &launch_context.game_path;

        for p in &fabric_profile.libraries {
            let relative_path = maven_to_path(&p.name);
            let full_path = libs_base_dir.join("objects").join(relative_path);

            if full_path.exists() {
                cp_paths.push(full_path.to_string_lossy().to_string());
            } else {
                tracing::warn!("Missing Fabric library: {:?}", full_path);
            }
        }
        fabric_profile.main_class.clone()
    } else {
        launch_context.main_class.clone()
    };

    // Add the game's core jar
    cp_paths.push(launch_context.core_jar.to_string_lossy().to_string());
    let classpath = cp_paths.join(sep);

    // Setup Directories
    let mc_dir = utils::get_minecraft_dir();
    let assets_dir = mc_dir.join("assets");
    // let version_isolated_dir = get_clients_dir().join(&launch_context.version_id);

    // if !version_isolated_dir.exists() {
    //     std::fs::create_dir_all(&version_isolated_dir)?;
    // }

    // JVM Runtime Parameters (MUST be before Main Class)
    if let Some(max_memory) = launch_context.max_memory {
        cmd.arg(format!("-Xmx{}M", max_memory));
    }
    cmd.arg("-XX:+UseG1GC");
    cmd.arg("-cp").arg(classpath);

    // Main Class (The divider)
    cmd.arg(final_main_class);

    // Game Parameters (MUST be after Main Class)
    cmd.arg("--username").arg(&launch_context.user.username);
    cmd.arg("--version").arg(&launch_context.version_id);
    cmd.arg("--uuid").arg(&launch_context.user.uuid);
    cmd.arg("--gameDir").arg(&launch_context.game_path);
    cmd.arg("--assetsDir").arg(&assets_dir);
    cmd.arg("--assetIndex").arg(&launch_context.asset_index_id);

    if let Some(access_token) = &launch_context.user.access_token {
        cmd.arg("--accessToken").arg(access_token);
    }

    cmd.arg("--userType").arg("mojang");
    cmd.arg("--versionType").arg("release");

    // 3. Logging and Execution
    let args_preview: Vec<String> = cmd
        .get_args()
        .map(|arg| {
            let s = arg.to_string_lossy();
            // Sensitive info mask
            if s.len() > 20 && !s.contains('/') && !s.contains('\\') {
                "********".to_string()
            } else {
                s.into_owned()
            }
        })
        .collect();

    tracing::info!("Execute command: {:?}", args_preview);

    let mut child = cmd.spawn()?;
    tracing::info!("🚀 The game has successfully started! PID: {}", child.id());

    let status = child.wait()?;
    tracing::info!("The game has exited, status code: {}", status);

    Ok(())
}
