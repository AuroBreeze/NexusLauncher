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

    let mp = MultiProgress::new();
    let main_pb = mp.add(ProgressBar::new(tasks.len() as u64));
    main_pb.set_style(ProgressStyle::with_template(progress_template)?);

    let mut set = JoinSet::new();
    // use Semaphore to control the number of concurrent downloads
    let semaphore = Arc::new(Semaphore::new(max_concurrent));

    for task in tasks {
        let pb = main_pb.clone();

        // 1. check local cach
        if task.local_path.exists() {
            main_pb.set_message(format!("✅ Cached: {}", task.name));
            main_pb.inc(1);
            continue;
        }

        let sem_clone = Arc::clone(&semaphore);

        // 2. dispatch async tasks
        set.spawn(async move {
            // Acquire the permit FIRST before doing any work or updating UI
            let _permit = sem_clone.acquire_owned().await.unwrap();

            // NOW update the message, so it reflects the file currently being downloaded
            pb.set_message(format!("📥 {}", task.name));

            let mut attempts = 0;
            let max_retries = 5;

            // 3. try to download
            loop {
                match download_and_verify(&task.url, &task.local_path, &task.sha1).await {
                    Ok(_) => break Ok(()),
                    Err(e) => {
                        attempts += 1;
                        if attempts > max_retries {
                            tracing::error!("❌ Download failed completely [{}]: {}", task.name, e);
                            break Err(e);
                        }

                        let wait_time = 2u64.pow(attempts as u32 - 1);
                        // Update message to show retry status for the current active task
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

    // wait for all tasks to complete and capture potential errors
    while let Some(res) = set.join_next().await {
        res??; // finish ? handle JoinError (panic), second ? handle AnyError
        main_pb.inc(1);
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
    let libs_base = super::utils::get_clients_dir()
        .join(game_name)
        .join("libraries");
    let objects_base = super::utils::get_clients_dir()
        .join(game_name)
        .join("objects");

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
