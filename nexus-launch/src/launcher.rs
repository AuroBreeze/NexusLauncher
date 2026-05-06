use crate::models::LaunchContext;
use nexus_core::AnyError;
use nexus_core::{self, get_minecraft_dir, maven_to_path};
use nexus_loader::models::FabricProfile;
use std::path::{Path, PathBuf};
use std::process::Command;

fn build_classpath(
    libraries: &[PathBuf],
    version_id: &str,
    fabric_profile: Option<&FabricProfile>,
    game_path: &Path,
) -> (String, Option<String>) {
    #[cfg(target_os = "windows")]
    let sep = ";";
    #[cfg(not(target_os = "windows"))]
    let sep = ":";

    let mut cp_paths: Vec<String> = libraries
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let fabric_main_class = if let Some(profile) = fabric_profile {
        tracing::info!("Fabric profile found!");
        for lib in &profile.libraries {
            let relative_path = maven_to_path(&lib.name);
            let full_path = game_path.join("objects").join(relative_path);
            if full_path.exists() {
                cp_paths.push(full_path.to_string_lossy().to_string());
            } else {
                tracing::warn!("Missing Fabric library: {:?}", full_path);
            }
        }
        Some(profile.main_class.clone())
    } else {
        None
    };

    let core_jar = game_path.join(format!("{}.jar", version_id));
    cp_paths.push(core_jar.to_string_lossy().to_string());
    (cp_paths.join(sep), fabric_main_class)
}

pub fn start_game(launch_context: LaunchContext) -> Result<(), AnyError> {
    tracing::info!("Assembling startup parameters...");

    // 0. Validation
    let java_path = launch_context
        .java_path
        .as_ref()
        .ok_or("Java executable not found")?;

    let mut cmd = Command::new(java_path);

    let (classpath, fabric_main_class) = build_classpath(
        &launch_context.libraries,
        &launch_context.version_id,
        launch_context.fabric_loader.as_ref(),
        &launch_context.game_path,
    );

    let final_main_class = fabric_main_class.unwrap_or(launch_context.main_class.clone());

    // Setup Directories
    let mc_dir = get_minecraft_dir();
    let assets_dir = mc_dir.join("assets");

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
