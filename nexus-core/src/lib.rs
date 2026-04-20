use home::home_dir;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tokio::process::Command;

static JAVA_VERSION_RE: OnceLock<Regex> = OnceLock::new();

pub type AnyError = Box<dyn std::error::Error + Send + Sync>;

pub fn get_minecraft_dir() -> PathBuf {
    let mut path = home_dir().expect("Could not get home dir");
    path.push(".minecraft");
    tracing::trace!("Minecraft directory: {}", path.display());
    path
}

pub fn get_library_path(relative_path: &str) -> PathBuf {
    let mut path = get_minecraft_dir();
    path.push("libraries");
    path.push(relative_path);
    path
}

pub fn get_clients_dir() -> PathBuf {
    get_minecraft_dir().join("clients")
}

pub fn get_servers_dir() -> PathBuf {
    get_minecraft_dir().join("servers")
}

pub fn init_workspace() -> std::io::Result<()> {
    let base = get_minecraft_dir();
    let client = get_clients_dir();
    let server = get_servers_dir();

    let folders = [
        base.clone(),
        client.clone(),
        server.clone(),
        base.join("libraries"),
        base.join("assets"),
        base.join("assets/indexes"),
        base.join("assets/objects"),
        base.join("runtimes"),
    ];

    for folder in folders.iter() {
        if !folder.exists() {
            tracing::info!("Creating workspace folder: {:?}", folder);
            fs::create_dir_all(folder)?;
        }
    }

    Ok(())
}

#[allow(dead_code)]
pub fn is_path_safe(target: &Path) -> bool {
    let base = get_minecraft_dir();
    target.starts_with(base)
}

/// Converts a maven coordinate to a path
pub fn maven_to_path(name: &str) -> String {
    let parts: Vec<&str> = name.split(':').collect();
    if parts.len() < 3 {
        return name.to_string();
    }

    let group = parts[0].replace('.', "/");
    let artifact = parts[1];
    let version = parts[2];

    format!(
        "{}/{}/{}/{}-{}.jar",
        group, artifact, version, artifact, version
    )
}

#[derive(Debug, Clone)]
pub struct JavaInfo {
    pub path: PathBuf,
    pub major_version: u32,
    pub full_version: String,
}

/// Internal logic of parsing Java version strings
/// "1.8.0_382" -> 8
/// "17.0.8" -> 17
fn parse_major_version(version_str: &str) -> Option<u32> {
    // Split the string by . or _
    let parts: Vec<&str> = version_str.split(['.', '_']).collect();
    if parts.is_empty() {
        return None;
    }

    if parts[0] == "1" && parts.len() > 1 {
        // Handle Java 8 and below (for example, 1.8 -> take 8)
        parts[1].parse().ok()
    } else {
        // Handle Java 9 and above (for example, 17.0 -> take 17)
        parts[0].parse().ok()
    }
}

/// Test the specified Java path and extract version information
pub async fn check_java_executable(java_path: &Path) -> Option<JavaInfo> {
    // Run java -version silently
    let output = Command::new(java_path)
        .arg("-version")
        .output()
        .await
        .ok()?;

    // Java's version information is always output to stderr
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Match something like: openjdk version "17.0.8" 2023-07-18
    let re = JAVA_VERSION_RE.get_or_init(|| Regex::new(r#"version "([^"]+)""#).unwrap());

    if let Some(caps) = re.captures(&stderr) {
        let full_version = caps[1].to_string();
        if let Some(major_version) = parse_major_version(&full_version) {
            return Some(JavaInfo {
                path: java_path.to_path_buf(),
                major_version,
                full_version,
            });
        }
    }

    None
}
