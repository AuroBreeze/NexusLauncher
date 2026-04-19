use super::download::{DownloadTask, download_and_verify, execute_downloads};
use super::models::{AssetIndexManifest, VersionDetail, VersionManifest};
use nexus_core::AnyError;
use std::path::PathBuf;

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
            let local_path = nexus_core::get_library_path(&artifact.path);
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
    let mc_dir = nexus_core::get_minecraft_dir();

    // 1. download asset index
    let index_path = mc_dir
        .join("assets")
        .join("indexes")
        .join(format!("{}.json", detail.asset_index.id));

    if !index_path.exists() {
        tracing::info!("Downloading resource index: {}.json", detail.asset_index.id);
        download_and_verify(
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
