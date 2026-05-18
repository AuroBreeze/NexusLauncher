use crate::models::LaunchContext;
use nexus_core::AnyError;
use nexus_core::{self, get_minecraft_dir, maven_to_path};
use nexus_loader::models::FabricProfile;
use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

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

fn monitor_child(
    mut child: Child,
    stderr: std::process::ChildStderr,
    stdout: std::process::ChildStdout,
    instance_name: String,
) {
    const STDERR_CAP: usize = 500;
    const STDOUT_CAP: usize = 100;

    let stderr_buf: Arc<Mutex<VecDeque<String>>> =
        Arc::new(Mutex::new(VecDeque::with_capacity(STDERR_CAP)));
    let stderr_buf_clone = Arc::clone(&stderr_buf);

    // Read stderr in a background thread — push directly into the bounded
    // buffer to keep the 500-line cap effective at all times.
    let stderr_reader = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            match line {
                Ok(l) => {
                    if l.contains("Exception")
                        || l.contains("FATAL")
                        || l.contains("ERROR")
                        || l.contains("crash")
                    {
                        tracing::error!("[game] {}", l);
                    } else {
                        tracing::debug!("[game] {}", l);
                    }
                    let mut buf = stderr_buf_clone.lock().unwrap();
                    if buf.len() >= STDERR_CAP {
                        buf.pop_front();
                    }
                    buf.push_back(l);
                }
                Err(_) => break,
            }
        }
    });

    // Read stdout on this thread
    let mut stdout_buf: VecDeque<String> = VecDeque::with_capacity(STDOUT_CAP);
    for line in BufReader::new(stdout).lines() {
        match line {
            Ok(l) => {
                tracing::trace!("[game:stdout] {}", l);
                if stdout_buf.len() >= STDOUT_CAP {
                    stdout_buf.pop_front();
                }
                stdout_buf.push_back(l);
            }
            Err(_) => break,
        }
    }

    let _ = stderr_reader.join();
    let stderr_buf = match Arc::try_unwrap(stderr_buf) {
        Ok(m) => m.into_inner().unwrap(),
        Err(a) => a.lock().unwrap().clone(),
    };

    match child.wait() {
        Ok(status) => {
            if status.success() {
                tracing::info!("The game has exited normally.");
            } else {
                tracing::warn!(
                    "The game exited abnormally with code {}. Writing crash log...",
                    status
                );
                write_crash_log(&instance_name, &status, &stderr_buf, &stdout_buf);
            }
        }
        Err(e) => {
            tracing::error!("Failed to wait on game process: {}", e);
        }
    }
}

const CRASH_LOG_DIR: &str = "crash_logs";
const MAX_CRASH_LOGS: usize = 20;

fn crash_log_dir() -> PathBuf {
    nexus_core::get_minecraft_dir().join(CRASH_LOG_DIR)
}

/// Remove oldest crash logs when the directory exceeds MAX_CRASH_LOGS.
fn prune_old_logs() {
    let dir = crash_log_dir();
    let mut entries: Vec<_> = match std::fs::read_dir(&dir) {
        Ok(d) => d.filter_map(|e| e.ok()).collect(),
        Err(_) => return,
    };
    if entries.len() <= MAX_CRASH_LOGS {
        return;
    }
    // Sort by modification time, oldest first
    entries.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
    let excess = entries.len() - MAX_CRASH_LOGS;
    for entry in entries.iter().take(excess) {
        let _ = std::fs::remove_file(entry.path());
    }
    if excess > 0 {
        tracing::debug!("Pruned {} old crash log(s)", excess);
    }
}

fn write_crash_log(
    instance_name: &str,
    status: &std::process::ExitStatus,
    stderr_buf: &VecDeque<String>,
    stdout_buf: &VecDeque<String>,
) {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let dir = crash_log_dir();
    if let Err(e) = std::fs::create_dir_all(&dir) {
        tracing::error!("Failed to create crash log dir {}: {}", dir.display(), e);
        return;
    }

    // TODO: Add more log control — configurable max file count and total size
    // limit, log level filter per instance, and rotation by age.
    prune_old_logs();

    let log_path = dir.join(format!("{}-{}.log", instance_name, ts));

    let mut f = match std::fs::File::create(&log_path) {
        Ok(f) => f,
        Err(e) => {
            tracing::error!(
                "Failed to create crash log at {}: {}",
                log_path.display(),
                e
            );
            return;
        }
    };

    let _ = writeln!(f, "=== Crash Report ===");
    let _ = writeln!(f, "Instance: {}", instance_name);
    let _ = writeln!(f, "Exit code: {}", status);
    let _ = writeln!(f, "Timestamp: {}", ts);
    let _ = writeln!(f);

    if !stderr_buf.is_empty() {
        let _ = writeln!(f, "--- Last {} stderr lines ---", stderr_buf.len());
        for line in stderr_buf {
            let _ = writeln!(f, "{}", line);
        }
    }

    if !stdout_buf.is_empty() {
        let _ = writeln!(f);
        let _ = writeln!(f, "--- Last {} stdout lines ---", stdout_buf.len());
        for line in stdout_buf {
            let _ = writeln!(f, "{}", line);
        }
    }

    tracing::info!("Crash log written to {}", log_path.display());
}

pub fn start_game(launch_context: LaunchContext) -> Result<JoinHandle<()>, AnyError> {
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

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn()?;
    let pid = child.id();
    tracing::info!("🚀 The game has successfully started! PID: {}", pid);

    let instance_name = launch_context
        .game_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Detach monitoring to a background thread.
    // If the game crashes, stderr is captured and written
    // to clients/<instance>/crash-<timestamp>.log.
    let stderr = child.stderr.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let handle = thread::spawn(move || {
        monitor_child(child, stderr, stdout, instance_name);
    });

    Ok(handle)
}
