use super::AnyError;
use super::download;
use super::models::AssetIndexManifest;
use super::models::{VersionDetail, VersionManifest};
use super::utils;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tracing;

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

    let mp = MultiProgress::new();
    let tasks: Vec<_> = detail
        .libraries
        .iter()
        .filter_map(|lib| {
            lib.downloads
                .artifact
                .as_ref()
                .map(|a| (lib.name.clone(), a.clone()))
        })
        .collect();

    let main_pb = mp.add(ProgressBar::new(tasks.len() as u64));
    main_pb.set_style(ProgressStyle::with_template(
    " {spinner:.green} Overall progress: [{wide_bar:.green/white}] {pos}/{len} ({percent}%) | {msg:50!}"
    )?);

    let mut classpath_libs = Vec::new();
    let mut set = JoinSet::new();

    // 1. Create a concurrency-limiting semaphore (to limit the number of concurrent download tasks to 10)
    let semaphore = Arc::new(Semaphore::new(10));

    for (name, artifact) in tasks {
        let local_path = utils::get_library_path(&artifact.path);
        classpath_libs.push(local_path.clone());

        let pb = main_pb.clone();
        let name_clone = name.clone();
        if !local_path.exists() {
            let sem_clone = Arc::clone(&semaphore);

            set.spawn(async move {
                pb.set_message(format!("📥 {}", name_clone));
                // 2. Obtain permission; only tasks that have been granted permission can continue to be executed
                let _permit = sem_clone.acquire_owned().await.unwrap();

                // 3. Add simple retry logic
                let mut attempts = 0;
                let max_attempts = 3;
                let mut last_error = None;

                while attempts < max_attempts {
                    match download::download_and_verify(&artifact.url, &local_path, &artifact.sha1)
                        .await
                    {
                        Ok(_) => return Ok(()),
                        Err(e) => {
                            attempts += 1;
                            // tracing::warn!(
                            //     "⏳ [{}] Download failed (Attempt {}/{}): {}",
                            //     name,
                            //     attempts,
                            //     max_attempts,
                            //     e
                            // );
                            pb.set_message(format!(
                                "⏳ [{}] Download failed (Attempt {}/{}): {}",
                                name, attempts, max_attempts, e
                            ));
                            last_error = Some(e);

                            if attempts < max_attempts {
                                let wait_time = 2u64.pow(attempts - 1);
                                tokio::time::sleep(tokio::time::Duration::from_secs(wait_time))
                                    .await;
                            }
                        }
                    }
                }
                Err(last_error.unwrap())
            });
        } else {
            main_pb.set_message(format!("✅ Cached: {}", name));
            main_pb.inc(1);
        }
    }

    while let Some(res) = set.join_next().await {
        res??;
        main_pb.inc(1);
    }

    main_pb.set_message("Done!");
    main_pb.finish_with_message("All dependent libraries are ready");
    Ok(classpath_libs)
}

pub async fn download_assets(detail: &VersionDetail) -> Result<(), AnyError> {
    tracing::info!("Start processing the asset files...");
    let mc_dir = utils::get_minecraft_dir();

    // 1. Download the Asset Index (for example, 1.20.json)
    let index_path = mc_dir
        .join("assets")
        .join("indexes")
        .join(format!("{}.json", detail.asset_index.id));

    if !index_path.exists() {
        tracing::info!("Downloading resource index: {}.json", detail.asset_index.id);
        // It is recommended to use retryable download here as well, but keep it simple for now
        download::download_and_verify(
            &detail.asset_index.url,
            &index_path,
            &detail.asset_index.sha1,
        )
        .await?;
    }

    // 2. Read and parse the Index
    let index_content = fs::read_to_string(&index_path).await?;
    let asset_manifest: AssetIndexManifest = serde_json::from_str(&index_content)?;

    // 3. Prepare for concurrent downloads
    let mp = MultiProgress::new();
    let tasks = asset_manifest.objects;
    let main_pb = mp.add(ProgressBar::new(tasks.len() as u64));
    main_pb.set_style(ProgressStyle::with_template(
        " {spinner:.yellow} Resource file: [{wide_bar:.yellow/white}] {pos}/{len} ({percent}%) | {msg:50!}",
    )?);

    let mut set = JoinSet::new();
    let objects_dir = mc_dir.join("assets").join("objects");

    // Limit the concurrency number
    let max_concurrent = 24;

    for (path_name, object) in tasks {
        let base_url = "https://resources.download.minecraft.net";
        let hash = object.hash.clone(); // Clone to move into the async block
        let prefix = &hash[0..2];
        let local_path = objects_dir.join(prefix).join(&hash);
        let download_url = format!("{}/{}/{}", base_url, prefix, hash);

        let pb = main_pb.clone();
        let name_clone = path_name.clone();

        if !local_path.exists() {
            // Control concurrency: If the limit is reached, wait for one to finish
            while set.len() >= max_concurrent {
                if let Some(res) = set.join_next().await {
                    res??;
                    main_pb.inc(1);
                }
            }

            let name_for_log = path_name.clone();
            // Launch the task with retry logic
            set.spawn(async move {
                pb.set_message(format!("📥 {}", name_clone));
                let mut attempts = 0;
                let max_retries = 3; // Maximum number of retries

                loop {
                    match download::download_and_verify(&download_url, &local_path, &hash).await {
                        Ok(_) => break Ok(()), // Download successful, break the loop
                        Err(e) => {
                            attempts += 1;
                            if attempts > max_retries {
                                tracing::error!(
                                    "❌ Resource download failed completely [{}]: {}",
                                    name_for_log,
                                    e
                                );
                                break Err(e); // Exceeded retry limit, return error
                            }

                            // Exponential backoff wait: 1s, 2s, 4s
                            let wait_time = 2u64.pow(attempts - 1);
                            // tracing::warn!(
                            //     "⏳ Resource download retry ({}/{}) [{}]: {}",
                            //     attempts,
                            //     max_retries,
                            //     name_for_log,
                            //     e
                            // );
                            pb.set_message(format!(
                                "⏳ Resource download retry ({}/{}) [{}]: {}",
                                attempts, max_retries, name_for_log, e
                            ));
                            tokio::time::sleep(tokio::time::Duration::from_secs(wait_time)).await;
                        }
                    }
                }
            });
        } else {
            main_pb.set_message(format!("✅ Cached: {}", path_name));

            main_pb.inc(1);
        }
    }

    // Handle the cleanup
    while let Some(res) = set.join_next().await {
        res??;
        main_pb.inc(1);
    }

    main_pb.finish_with_message("All resource files are ready!");
    Ok(())
}
