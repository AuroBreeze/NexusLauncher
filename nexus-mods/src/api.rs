use crate::models::{
    DepEntry, ListVersionsParams, ModEntry, ModManifest, Project, ProjectDependencies,
    SearchParams, SearchResult, Version, VersionFile,
};
use futures_util::StreamExt;
use nexus_core::AnyError;
use reqwest::Client;

// URL-encode a JSON string for use as a query parameter value
fn url_encode_json(json: &str) -> String {
    json.replace('%', "%25")
        .replace('[', "%5B")
        .replace(']', "%5D")
        .replace('"', "%22")
}

// Format a slice of strings as a JSON array for query parameter values
fn json_array(items: &[String]) -> String {
    let inner: Vec<String> = items.iter().map(|s| format!("\"{}\"", s)).collect();
    format!("[{}]", inner.join(","))
}

/// Search for projects on Modrinth.
///
/// See: <https://docs.modrinth.com/api/operations/searchprojects/>
pub async fn search_project(params: &SearchParams) -> Result<SearchResult, AnyError> {
    let limit = params.limit.unwrap_or(10).min(100);
    let index = params.index.as_deref().unwrap_or("relevance");
    let offset = params.offset.unwrap_or(0);

    let mut url = format!(
        "https://api.modrinth.com/v2/search?query={}&limit={}&index={}&offset={}",
        params.query, limit, index, offset
    );

    if let Some(ref facets) = params.facets {
        let groups: Vec<String> = facets
            .iter()
            .map(|group| {
                let items: Vec<String> = group.iter().map(|f| format!("\"{}\"", f)).collect();
                format!("[{}]", items.join(","))
            })
            .collect();
        let json = format!("[{}]", groups.join(","));
        url.push_str(&format!("&facets={}", url_encode_json(&json)));
    }

    let client = Client::new();
    let resp = client
        .get(&url)
        .header("User-Agent", "AuroBreeze/NexusLauncher/0.1.0")
        .send()
        .await?
        .error_for_status()?;
    let result = resp.json::<SearchResult>().await?;
    tracing::info!("📦 Found {} mods matching your query", result.hits.len());
    for hit in &result.hits {
        tracing::info!(
            "  • {} ({}) — {} — by {}",
            hit.title,
            hit.project_id,
            hit.description,
            hit.author
        );
    }
    tracing::debug!("📦 Mod list: {:#?}", result.hits);
    Ok(result)
}

/// Get a single project by ID or slug.
///
/// See: <https://docs.modrinth.com/api/operations/getproject/>
pub async fn get_project(id_or_slug: &str) -> Result<Project, AnyError> {
    let url = format!("https://api.modrinth.com/v2/project/{}", id_or_slug);

    let client = Client::new();
    let resp = client
        .get(&url)
        .header("User-Agent", "AuroBreeze/NexusLauncher/0.1.0")
        .send()
        .await?
        .error_for_status()?;
    let result = resp.json::<Project>().await?;
    tracing::info!("📦 Project: {} ({})", result.title, result.id);
    Ok(result)
}

/// List a project's versions.
///
/// See: <https://docs.modrinth.com/api/operations/getprojectversions/>
pub async fn list_project_versions(params: &ListVersionsParams) -> Result<Vec<Version>, AnyError> {
    let mut url = format!(
        "https://api.modrinth.com/v2/project/{}/version",
        params.id_or_slug
    );

    let mut query_params: Vec<String> = Vec::new();
    if let Some(ref loaders) = params.loaders {
        query_params.push(format!("loaders={}", json_array(loaders)));
    }
    if let Some(ref game_versions) = params.game_versions {
        query_params.push(format!("game_versions={}", json_array(game_versions)));
    }
    if let Some(featured) = params.featured {
        query_params.push(format!("featured={}", featured));
    }
    if let Some(include_changelog) = params.include_changelog {
        query_params.push(format!("include_changelog={}", include_changelog));
    }
    if !query_params.is_empty() {
        url.push('?');
        url.push_str(&query_params.join("&"));
    }

    let client = Client::new();
    let resp = client
        .get(&url)
        .header("User-Agent", "AuroBreeze/NexusLauncher/0.1.0")
        .send()
        .await?
        .error_for_status()?;
    let result = resp.json::<Vec<Version>>().await?;
    tracing::info!(
        "📦 Found {} versions for project {}",
        result.len(),
        params.id_or_slug
    );
    Ok(result)
}

/// Get all of a project's dependencies.
///
/// See: <https://docs.modrinth.com/api/operations/getprojectdependencies/>
pub async fn get_project_dependencies(id_or_slug: &str) -> Result<ProjectDependencies, AnyError> {
    let url = format!(
        "https://api.modrinth.com/v2/project/{}/dependencies",
        id_or_slug
    );

    let client = Client::new();
    let resp = client
        .get(&url)
        .header("User-Agent", "AuroBreeze/NexusLauncher/0.1.0")
        .send()
        .await?
        .error_for_status()?;
    let result = resp.json::<ProjectDependencies>().await?;
    tracing::info!(
        "📦 Found {} dependent projects and {} dependent versions",
        result.projects.len(),
        result.versions.len()
    );
    Ok(result)
}

/// Get a single version by ID.
///
/// See: <https://docs.modrinth.com/api/operations/getversion/>
pub async fn get_version(id: &str) -> Result<Version, AnyError> {
    let url = format!("https://api.modrinth.com/v2/version/{}", id);

    let client = Client::new();
    let resp = client
        .get(&url)
        .header("User-Agent", "AuroBreeze/NexusLauncher/0.1.0")
        .send()
        .await?
        .error_for_status()?;
    let result = resp.json::<Version>().await?;
    tracing::info!("📦 Version: {} — {}", result.name, result.version_number);
    Ok(result)
}

/// Get a version's files by version ID.
///
/// Convenience wrapper around [`get_version`].
pub async fn get_version_files(id: &str) -> Result<Vec<VersionFile>, AnyError> {
    let version = get_version(id).await?;
    tracing::info!(
        "📦 {} files for version {}",
        version.files.len(),
        version.name
    );
    Ok(version.files)
}

// TODO: Resolve and download mod dependencies (e.g., sodium needs fabric-api)
// TODO: Track mod download statistics (count, version, timestamp)
// TODO: Adding downloads for specific versions, etc.
/// Search for a mod and download the latest matching version.
///
/// Files are placed directly in `clients/<instance>/mods/`.
/// Uses SHA1 verification via [`nexus_version::download::download_and_verify`].
pub async fn download_mod_to_instance(
    query: &str,
    game_version: Option<&str>,
    loader: Option<&str>,
    version_type: Option<&str>,
    instance_name: &str,
) -> Result<std::path::PathBuf, AnyError> {
    nexus_core::validate_instance_name(instance_name)?;
    let mut facets = vec![vec!["project_type:mod".to_string()]];
    if let Some(gv) = game_version {
        facets.push(vec![format!("versions:{}", gv)]);
    }
    let params = SearchParams {
        query: query.to_string(),
        limit: Some(1),
        offset: None,
        index: Some("downloads".to_string()),
        facets: Some(facets),
    };
    let sr = search_project(&params).await?;
    let hit = sr
        .hits
        .first()
        .ok_or_else(|| format!("No mod found for '{}'", query))?;

    tracing::info!("📦 Found: {} ({})", hit.title, hit.project_id);

    let versions = list_project_versions(&ListVersionsParams {
        id_or_slug: hit.project_id.clone(),
        loaders: loader.map(|l| vec![l.to_string()]),
        game_versions: game_version.map(|gv| vec![gv.to_string()]),
        featured: None,
        include_changelog: Some(false),
    })
    .await?;

    // Filter by version_type if specified
    let version = if let Some(vt) = version_type {
        versions
            .iter()
            .find(|v| v.version_type == vt)
            .ok_or_else(|| {
                let available: Vec<&str> =
                    versions.iter().map(|v| v.version_type.as_str()).collect();
                format!("No {} version found. Available types: {:?}", vt, available)
            })?
    } else {
        versions.first().ok_or("No matching version found")?
    };
    tracing::info!(
        "📦 Selected: {} {} ({}, loaders: [{}], game: [{}], {} files, {} deps)",
        version.name,
        version.version_number,
        version.version_type,
        version.loaders.join(", "),
        version.game_versions.join(", "),
        version.files.len(),
        version.dependencies.len(),
    );
    tracing::debug!("📦 Version details: {:#?}", version);

    // TODO: Use loader info from API response instead of filename matching
    let primary_file = version
        .files
        .iter()
        .find(|f| {
            f.primary
                && loader
                    .map(|l| f.filename.to_lowercase().contains(&l.to_lowercase()))
                    .unwrap_or(true)
        })
        .or_else(|| {
            version.files.iter().find(|f| {
                loader
                    .map(|l| f.filename.to_lowercase().contains(&l.to_lowercase()))
                    .unwrap_or(true)
            })
        })
        .or_else(|| version.files.iter().find(|f| f.primary))
        .or_else(|| version.files.first())
        .ok_or("No matching file found")?;

    let dest_dir = nexus_core::get_clients_dir()
        .join(instance_name)
        .join("mods");

    let dest = dest_dir.join(&primary_file.filename);
    download_with_progress(&primary_file.url, &dest, &primary_file.hashes.sha1).await?;

    let mut manifest = ModManifest::load(instance_name)?;

    if !version.dependencies.is_empty() {
        tracing::info!(
            "📦 Resolving {} dependencies for {}...",
            version.dependencies.len(),
            hit.title
        );
    }

    // Resolve dependency names concurrently
    let dep_futures: Vec<_> = version
        .dependencies
        .iter()
        .map(|d| {
            let pid = d.project_id.clone();
            let vid = d.version_id.clone();
            let dtype = d.dependency_type.clone();
            async move {
                let name = if let Some(ref pid) = pid {
                    get_project(pid).await.ok().map(|p| p.title)
                } else {
                    None
                };
                DepEntry {
                    name,
                    project_id: pid,
                    version_id: vid,
                    dependency_type: dtype,
                }
            }
        })
        .collect();
    let deps = futures_util::future::join_all(dep_futures).await;

    let entry = ModEntry {
        name: hit.title.clone(),
        project_id: hit.project_id.clone(),
        version_number: version.version_number.clone(),
        version_type: version.version_type.clone(),
        filename: primary_file.filename.clone(),
        sha1: primary_file.hashes.sha1.clone(),
        loader: loader.unwrap_or("unknown").to_string(),
        game_version: game_version.unwrap_or("unknown").to_string(),
        installed_at: {
            let t = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            t.as_secs().to_string()
        },
        dependencies: deps,
    };
    if let Some(existing) = manifest
        .mods
        .iter_mut()
        .find(|m| m.project_id == hit.project_id)
    {
        *existing = entry;
        tracing::info!("📋 Updated {} in manifest", hit.title);
    } else {
        manifest.mods.push(entry);
        tracing::info!(
            "📋 Added {} to manifest ({} entries)",
            hit.title,
            manifest.mods.len()
        );
    }
    manifest.save(instance_name)?;
    Ok(dest)
}

async fn download_with_progress(
    url: &str,
    path: &std::path::Path,
    sha1: &str,
) -> Result<(), AnyError> {
    use indicatif::{ProgressBar, ProgressStyle};
    use sha1::{Digest, Sha1};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    if path.exists() {
        let mut file = tokio::fs::File::open(path).await?;
        let mut hasher = Sha1::new();
        let mut buf = [0u8; 8192];
        loop {
            let n = file.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        if hex::encode(hasher.finalize()) == sha1 {
            tracing::info!("File already exists, skipping");
            return Ok(());
        }
    }

    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let response = reqwest::get(url).await?.error_for_status()?;
    let total_size = response.content_length().unwrap_or(0);

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::with_template(
            "  {spinner:.green} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})",
        )?
        .progress_chars("#>-"),
    );

    let mut file = tokio::fs::File::create(path).await?;
    let mut hasher = Sha1::new();
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        hasher.update(&chunk);
        downloaded += chunk.len() as u64;
        pb.set_position(downloaded);
    }

    pb.finish_with_message("Done");
    let actual = hex::encode(hasher.finalize());
    if actual != sha1 {
        let _ = tokio::fs::remove_file(path).await;
        return Err("SHA1 verification failed".into());
    }
    Ok(())
}

#[cfg(test)]
#[path = "api_tests.rs"]
mod tests;
