// src/java.rs
use crate::version::AnyError;
use crate::version::utils::get_minecraft_dir;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use std::env;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

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
    let parts: Vec<&str> = version_str.split(|c| c == '.' || c == '_').collect();
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
    let re = Regex::new(r#"version "([^"]+)""#).unwrap();

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

/// Deep scans the specified directory looking for bin/java executables.
async fn scan_jvm_directory(dir: &Path) -> Vec<JavaInfo> {
    let mut results = Vec::new();

    // Return early to avoid unnecessary nesting
    if !dir.is_dir() {
        return results;
    }

    tracing::debug!("Deep scanning directory: {}", dir.display());

    // Handle errors when reading the directory (e.g., permission denied, missing folder)
    let mut entries = match fs::read_dir(dir).await {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("Failed to read Java directory {}: {}", dir.display(), e);
            return results;
        }
    };

    // Use loop + match instead of `while let` for precise error control
    loop {
        match entries.next_entry().await {
            Ok(Some(entry)) => {
                let java_bin = entry.path().join("bin/java");
                // is_file() is safer and more strict than exists()
                if java_bin.is_file() {
                    if let Some(info) = check_java_executable(&java_bin).await {
                        results.push(info);
                    }
                } else {
                    tracing::debug!("Not a valid Java executable: {}", java_bin.display());
                }
            }
            Ok(None) => break, // Successfully read all entries, exit the loop
            Err(e) => {
                // Log the specific I/O error without failing silently or panicking
                tracing::warn!(
                    "I/O error while reading entries in {}: {}",
                    dir.display(),
                    e
                );
                break;
            }
        }
    }

    results
}

pub async fn scan_local_java_environments(custom_scan_path: Option<&Path>) -> Vec<JavaInfo> {
    tracing::info!("Scanning local Java environment...");
    let mut javas: Vec<JavaInfo> = Vec::new();

    // Helper closure: Check for duplicates (determined by absolute path or full version number)
    let mut add_if_new = |new_java: JavaInfo| {
        let is_duplicate = javas.iter().any(|j| {
            // If the path is the same, or the major version and detailed version are exactly the same, it is considered the same
            j.path == new_java.path
                || (j.major_version == new_java.major_version
                    && j.full_version == new_java.full_version)
        });

        if !is_duplicate {
            javas.push(new_java);
        }
    };

    // 1: Check the JAVA_HOME environment variable (the most respected option)
    if let Ok(java_home) = env::var("JAVA_HOME") {
        let java_bin = Path::new(&java_home).join("bin/java");
        if java_bin.exists() {
            tracing::debug!("Detected JAVA_HOME: {}", java_home);
            if let Some(info) = check_java_executable(&java_bin).await {
                add_if_new(info);
            }
        }
    }

    // 2: Check the default java command in the system PATH
    if let Some(info) = check_java_executable(Path::new("java")).await {
        add_if_new(info);
    }

    // 3: Scan the runtimes directory
    let runtimes_dir = get_minecraft_dir().join("runtimes");
    for info in scan_jvm_directory(&runtimes_dir).await {
        add_if_new(info);
    }

    // 4: Scan a specifically designated location (for example, the runtimes folder that comes with the launcher)
    if let Some(scan_path) = custom_scan_path {
        for info in scan_jvm_directory(scan_path).await {
            add_if_new(info);
        }
    }

    // 5: Arch Linux / General Linux default installation path
    let jvm_dir = Path::new("/usr/lib/jvm");
    for info in scan_jvm_directory(jvm_dir).await {
        add_if_new(info);
    }

    tracing::info!(
        "Scan complete, a total of {} unique Java environments were found",
        javas.len()
    );
    javas
}

/// Automatically downloads and extracts the correct Java JRE from Adoptium API.
/// Returns the path to the directory where Java was extracted.
pub async fn download_java(major_version: u32, runtimes_dir: &Path) -> Result<PathBuf, AnyError> {
    tracing::info!("Preparing to download Java {}...", major_version);

    // 1. Detect OS and Architecture dynamically
    let os = match env::consts::OS {
        "windows" => "windows",
        "macos" => "mac",
        _ => "linux", // Default to linux for Arch Linux
    };

    let arch = match env::consts::ARCH {
        "aarch64" => "aarch64",
        _ => "x64", // Standard 64-bit architecture
    };

    // 2. Construct the Adoptium V3 API URL (Requesting JRE, not full JDK)
    let url = format!(
        "https://api.adoptium.net/v3/binary/latest/{}/ga/{}/{}/jre/hotspot/normal/eclipse",
        major_version, os, arch
    );

    // Create an isolated directory for this specific Java version
    let target_dir = runtimes_dir.join(format!("jre-{}", major_version));
    if !target_dir.exists() {
        fs::create_dir_all(&target_dir).await?;
    }

    // 3. Initiate the download stream
    let response = reqwest::get(&url).await?;
    let total_size = response.content_length().unwrap_or(0);

    let pb = ProgressBar::new(total_size);
    if let Ok(style) = ProgressStyle::with_template(
        "{spinner:.green} [Downloading Java] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})",
    ) {
        pb.set_style(style.progress_chars("#>-"));
    }

    // Prepare a temporary file for the archive
    let archive_ext = if os == "windows" { "zip" } else { "tar.gz" };
    let temp_archive_path =
        runtimes_dir.join(format!("temp_java_{}.{}", major_version, archive_ext));

    let mut file = fs::File::create(&temp_archive_path).await?;
    let mut stream = response.bytes_stream();

    // 4. Stream chunks to disk and update progress bar
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        file.write_all(&chunk).await?;
        pb.inc(chunk.len() as u64);
    }
    pb.finish_with_message("Java download complete!");

    // 5. Extract the archive (Only handling .tar.gz for Linux/Mac in this scope)
    tracing::info!("Extracting Java {} archive...", major_version);
    let target_dir_clone = target_dir.clone();

    // Extraction is CPU-bound and blocking, so it must run in spawn_blocking
    tokio::task::spawn_blocking(move || -> Result<(), std::io::Error> {
        if cfg!(target_os = "windows") {
            // ===== Windows: Extract zip =====
            use std::fs::{self, File};
            use std::io::copy;
            use zip::ZipArchive;

            let file = File::open(&temp_archive_path)?;
            let mut archive = ZipArchive::new(file)?;

            for i in 0..archive.len() {
                let mut entry = archive.by_index(i)?;
                let outpath = target_dir_clone.join(entry.mangled_name());

                if entry.name().ends_with('/') {
                    fs::create_dir_all(&outpath)?;
                } else {
                    if let Some(parent) = outpath.parent() {
                        fs::create_dir_all(parent)?;
                    }

                    let mut outfile = File::create(&outpath)?;
                    copy(&mut entry, &mut outfile)?;
                }
            }
        } else {
            // ===== Linux: Extract tar=====
            use flate2::read::GzDecoder;
            use std::fs::File;
            use tar::Archive;

            let tar_gz = File::open(&temp_archive_path)?;
            let tar = GzDecoder::new(tar_gz);
            let mut archive = Archive::new(tar);

            archive.unpack(&target_dir_clone)?;
        }

        std::fs::remove_file(&temp_archive_path)?;
        Ok(())
    })
    .await??;

    tracing::info!(
        "Java {} successfully installed at {}",
        major_version,
        target_dir.display()
    );

    Ok(target_dir)
}
