use home::home_dir;
use std::fs;
use std::path::{Path, PathBuf};

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
