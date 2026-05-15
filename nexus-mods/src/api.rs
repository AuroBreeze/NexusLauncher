use crate::models::{
    ListVersionsParams, Project, ProjectDependencies, SearchParams, SearchResult, Version,
    VersionFile,
};
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
        .await?;
    let result = resp.json::<SearchResult>().await?;
    tracing::info!("📦 Found {} mods matching your query", result.hits.len());
    tracing::info!("📦 Mod list: {:#?}", result.hits);
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
        .await?;
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
        .await?;
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
        .await?;
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
        .await?;
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

// TODO: Adding downloads for specific versions, etc.
/// Search for a mod and download the latest matching version.
///
/// Files are placed directly in `clients/<instance>/mods/`.
/// Uses SHA1 verification via [`nexus_version::download::download_and_verify`].
pub async fn download_mod_to_instance(
    query: &str,
    game_version: Option<&str>,
    loader: Option<&str>,
    instance_name: &str,
) -> Result<std::path::PathBuf, AnyError> {
    let facets = game_version.map(|gv| vec![vec![format!("versions:{}", gv)]]);
    let params = SearchParams {
        query: query.to_string(),
        limit: Some(1),
        offset: None,
        index: Some("downloads".to_string()),
        facets,
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

    let version = versions.first().ok_or("No matching version found")?;
    tracing::info!("📦 Latest version: {}", version.version_number);

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
        .ok_or("No matching file found")?;

    let dest_dir = nexus_core::get_clients_dir()
        .join(instance_name)
        .join("mods");

    let dest = dest_dir.join(&primary_file.filename);
    nexus_version::download::download_and_verify(
        &primary_file.url,
        &dest,
        &primary_file.hashes.sha1,
    )
    .await?;
    Ok(dest)
}

#[cfg(test)]
#[path = "api_tests.rs"]
mod tests;
