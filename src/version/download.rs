use super::AnyError;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use sha1::{Digest, Sha1};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub async fn download_and_verify(
    url: &str,
    save_path: &PathBuf,
    expected_sha1: &str,
) -> Result<(), AnyError> {
    if save_path.exists() {
        let content = fs::read(save_path).await?;
        let mut hasher = Sha1::new();
        hasher.update(&content);
        let actual_sha1 = hex::encode(hasher.finalize());
        if actual_sha1 == expected_sha1 {
            tracing::debug!("File {} already exists, skipping", save_path.display());
            return Ok(());
        }
    }

    if let Some(parent) = save_path.parent() {
        fs::create_dir_all(parent).await?;
    }

    let response = reqwest::get(url).await?;
    let total_size = response.content_length().unwrap_or(0);

    let file_name = save_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");

    let pb = if false {
        let p = ProgressBar::new(total_size);
        if let Ok(style) = ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})",
        ) {
            p.set_style(style.progress_chars("#>-"));
        }
        p.set_message(format!("Downloading {}", file_name));
        p
    } else {
        ProgressBar::hidden() // Silent mode: Does not display, but can still receive inc() updates without errors
    };

    let mut file = fs::File::create(save_path).await?;
    let mut hasher = Sha1::new();
    let mut stream = response.bytes_stream();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;

        file.write_all(&chunk).await?;
        // Update hash
        hasher.update(&chunk);
        // Update progress bar
        pb.inc(chunk.len() as u64);
    }

    pb.finish_and_clear();

    let actual_sha1 = hex::encode(hasher.finalize());
    if actual_sha1 != expected_sha1 {
        let _ = fs::remove_file(save_path).await;
        return Err("SHA1 verification failed".into());
    }

    // tracing::info!("Successfully downloaded and verified: {}", file_name);
    Ok(())
}

pub async fn pool_download_and_link(
    url: &str,
    lib_relative_path: &str,
) -> Result<PathBuf, AnyError> {
    let libs_base = super::utils::get_minecraft_dir().join("libraries");
    let objects_base = super::utils::get_minecraft_dir().join("objects");

    let target_path = libs_base.join(lib_relative_path);

    let pool_path = objects_base.join(lib_relative_path);

    if !pool_path.exists() {
        fs::create_dir_all(pool_path.parent().unwrap()).await?;
        let resp = reqwest::get(url).await?.bytes().await?;
        fs::write(&pool_path, resp).await?;
    }

    if !target_path.exists() {
        fs::create_dir_all(target_path.parent().unwrap()).await?;
        fs::hard_link(&pool_path, &target_path).await?;
    }

    Ok(target_path)
}
