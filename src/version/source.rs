use super::AnyError;
use super::download;
use super::models::AssetIndexManifest;
use super::models::{VersionDetail, VersionManifest};
use super::utils;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tracing;

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
            pb.set_message(format!("📥 {}", task.name));

            // get the permit
            let _permit = sem_clone.acquire_owned().await.unwrap();

            let mut attempts = 0;
            let max_retries = 5;

            // 3. try to download
            loop {
                match download::download_and_verify(&task.url, &task.local_path, &task.sha1).await {
                    Ok(_) => break Ok(()),
                    Err(e) => {
                        attempts += 1;
                        if attempts > max_retries {
                            tracing::error!("❌ Download failed completely [{}]: {}", task.name, e);
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

    // wait for all tasks to complete and capture potential errors
    while let Some(res) = set.join_next().await {
        res??; // finish ? handle JoinError (panic), second ? handle AnyError
        main_pb.inc(1);
    }

    main_pb.set_message("Done!");
    main_pb.finish_with_message(finish_message.to_string());

    Ok(())
}

/// obtain_manifest
pub async fn obtain_manifest() -> Result<VersionManifest, AnyError> {
    let url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
    tracing::info!("Obtaining version manifest from {}", url);

    let response = reqwest::get(url).await?;

    let manifest = response.json::<VersionManifest>().await?;

    tracing::info!("Latest release: {}", manifest.latest.release);
    tracing::info!("Latest snapshot: {}", manifest.latest.snapshot);

    Ok(manifest)
}

/// fetch_version_detail
pub async fn fetch_version_detail(url: &str) -> Result<VersionDetail, AnyError> {
    tracing::trace!("Fetching version detail from {}", url);
    let response = reqwest::get(url).await?;
    let detail = response.json::<VersionDetail>().await?;
    tracing::trace!("Version detail: {:#?}", detail);
    Ok(detail)
}

pub async fn download_libraries(detail: &VersionDetail) -> Result<Vec<PathBuf>, AnyError> {
    tracing::info!("Start preparing the dependency library...");

    let mut tasks = Vec::new();
    let mut classpath_libs = Vec::new();

    // Parse out the content and path
    for lib in &detail.libraries {
        if let Some(artifact) = &lib.downloads.artifact {
            let local_path = utils::get_library_path(&artifact.path);
            classpath_libs.push(local_path.clone());

            tasks.push(DownloadTask {
                name: lib.name.clone(),
                url: artifact.url.clone(),
                local_path,
                sha1: artifact.sha1.clone(),
            });
        }
    }

    execute_downloads(
        tasks,
        " {spinner:.green} Overall progress: [{wide_bar:.green/white}] {pos}/{len} ({percent}%) | {msg:50!}",
        10,
        "All dependent libraries are ready",
    ).await?;

    Ok(classpath_libs)
}

pub async fn download_assets(detail: &VersionDetail) -> Result<(), AnyError> {
    tracing::info!("Start processing the asset files...");
    let mc_dir = utils::get_minecraft_dir();

    // 1. download asset index
    let index_path = mc_dir
        .join("assets")
        .join("indexes")
        .join(format!("{}.json", detail.asset_index.id));

    if !index_path.exists() {
        tracing::info!("Downloading resource index: {}.json", detail.asset_index.id);
        download::download_and_verify(
            &detail.asset_index.url,
            &index_path,
            &detail.asset_index.sha1,
        )
        .await?;
    }

    // 2. Read and parse Index
    let index_content = tokio::fs::read_to_string(&index_path).await?;
    let asset_manifest: AssetIndexManifest = serde_json::from_str(&index_content)?;

    // 3. Prepare concurrent download tasks
    let objects_dir = mc_dir.join("assets").join("objects");
    let base_url = "https://resources.download.minecraft.net";
    let mut tasks = Vec::new();

    for (path_name, object) in asset_manifest.objects {
        let hash = object.hash;
        let prefix = &hash[0..2];
        let local_path = objects_dir.join(prefix).join(&hash);
        let download_url = format!("{}/{}/{}", base_url, prefix, hash);

        tasks.push(DownloadTask {
            name: path_name,
            url: download_url,
            local_path,
            sha1: hash,
        });
    }

    // 4. Call the generic downloader and set concurrency to 16.
    execute_downloads(
        tasks,
        " {spinner:.yellow} Resource file: [{wide_bar:.yellow/white}] {pos}/{len} ({percent}%) | {msg:50!}",
        16,
        "All resource files are ready!",
    ).await?;

    Ok(())
}
