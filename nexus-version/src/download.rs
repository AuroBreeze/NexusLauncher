use crate::AnyError;
use futures_util::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use sha1::{Digest, Sha1};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

pub struct DownloadTask {
    pub name: String,
    pub url: String,
    pub local_path: PathBuf,
    pub sha1: String,
}

pub async fn execute_downloads(
    tasks: Vec<DownloadTask>,
    progress_template: &str,
    max_concurrent: usize,
    finish_message: &str,
) -> Result<(), AnyError> {
    if tasks.is_empty() {
        return Ok(());
    }

    let total = tasks.len() as u64;
    let mp = MultiProgress::new();
    let main_pb = mp.add(ProgressBar::new(total));
    main_pb.set_style(ProgressStyle::with_template(progress_template)?);

    // ── Phase 1: parallel SHA1 cache check for existing files ──
    // Use 2x concurrency since hashing is CPU-bound and fast on cached files.
    let check_concurrency = (max_concurrent * 2).max(16);
    let cache_sem = Arc::new(Semaphore::new(check_concurrency));
    let mut cache_set: JoinSet<Result<Option<DownloadTask>, AnyError>> = JoinSet::new();

    let mut download_tasks = Vec::new();

    for task in tasks {
        if task.local_path.exists() {
            let pb = main_pb.clone();
            let sem = Arc::clone(&cache_sem);
            cache_set.spawn(async move {
                let _permit = sem.acquire_owned().await.unwrap();
                if let Ok(content) = fs::read(&task.local_path).await {
                    let mut hasher = Sha1::new();
                    hasher.update(&content);
                    if hex::encode(hasher.finalize()) == task.sha1 {
                        pb.set_message(format!("✅ Cached: {}", task.name));
                        pb.inc(1);
                        return Ok(None); // valid cache hit
                    }
                }
                pb.set_message(format!("🔄 Re-download: {}", task.name));
                Ok(Some(task)) // needs re-download
            });
        } else {
            download_tasks.push(task);
        }
    }

    // Collect cache-check results
    while let Some(res) = cache_set.join_next().await {
        match res {
            Ok(Ok(Some(task))) => download_tasks.push(task),
            Ok(Ok(None)) => {} // already counted as cached
            Ok(Err(e)) => return Err(e),
            Err(e) => return Err(e.into()),
        }
    }

    // ── Phase 2: download remaining (missing + failed-cache) ──
    if !download_tasks.is_empty() {
        let semaphore = Arc::new(Semaphore::new(max_concurrent));
        let mut dl_set = JoinSet::new();

        for task in download_tasks {
            let pb = main_pb.clone();
            let sem_clone = Arc::clone(&semaphore);

            dl_set.spawn(async move {
                let _permit = sem_clone.acquire_owned().await.unwrap();
                pb.set_message(format!("📥 {}", task.name));

                let mut attempts = 0;
                let max_retries = 5;

                loop {
                    match download_and_verify(&task.url, &task.local_path, &task.sha1).await {
                        Ok(_) => break Ok(()),
                        Err(e) => {
                            attempts += 1;
                            if attempts > max_retries {
                                tracing::error!(
                                    "❌ Download failed completely [{}]: {}",
                                    task.name,
                                    e
                                );
                                break Err(e);
                            }
                            let wait_time = 2u64.pow(attempts as u32 - 1);
                            pb.set_message(format!(
                                "⏳ Retry ({}/{}) [{}]: {}",
                                attempts, max_retries, task.name, e
                            ));
                            tokio::time::sleep(tokio::time::Duration::from_secs(wait_time)).await;
                        }
                    }
                }
            });
        }

        while let Some(res) = dl_set.join_next().await {
            res??;
            main_pb.inc(1);
        }
    }

    main_pb.set_message("Done!");
    main_pb.finish_with_message(finish_message.to_string());

    Ok(())
}
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

    let pb = ProgressBar::hidden(); // Silent mode: Does not display, but can still receive inc() updates without errors

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
    game_name: &str,
) -> Result<PathBuf, AnyError> {
    let libs_base = nexus_core::get_clients_dir()
        .join(game_name)
        .join("libraries");
    let objects_base = nexus_core::get_clients_dir()
        .join(game_name)
        .join("objects");

    let target_path = libs_base.join(lib_relative_path);

    let pool_path = objects_base.join(lib_relative_path);

    // TODO: Add SHA1 verification — exists() check alone may accept corrupt partial downloads
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
