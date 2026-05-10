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

    fn json_array(items: &[String]) -> String {
        let inner: Vec<String> = items.iter().map(|s| format!("\"{}\"", s)).collect();
        format!("[{}]", inner.join(","))
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SearchParams;

    fn params(query: &str, limit: i32) -> SearchParams {
        SearchParams {
            query: query.to_string(),
            limit: Some(limit),
            offset: None,
            index: None,
            facets: None,
        }
    }

    #[tokio::test]
    async fn test_search_mods_real_network_return_200() {
        let p = params("fabric-api", 3);
        let result = search_project(&p).await;
        assert!(result.is_ok(), "Failed to fetch from Modrinth API");
        let sr = result.unwrap();
        assert!(sr.hits.len() <= 3);
        if !sr.hits.is_empty() {
            let first_hit = &sr.hits[0];
            assert!(!first_hit.project_id.is_empty());
            assert!(!first_hit.title.is_empty());
        }
    }

    #[tokio::test]
    async fn test_search_with_default_limit() {
        let p = SearchParams {
            query: "sodium".to_string(),
            limit: None,
            offset: None,
            index: None,
            facets: None,
        };
        let result = search_project(&p).await;
        assert!(result.is_ok());
        let sr = result.unwrap();
        assert!(sr.hits.len() <= 10);
        assert_eq!(sr.limit, 10);
    }

    #[tokio::test]
    async fn test_search_with_index_downloads() {
        let p = SearchParams {
            query: "fabric-api".to_string(),
            limit: Some(5),
            offset: None,
            index: Some("downloads".to_string()),
            facets: None,
        };
        let result = search_project(&p).await;
        assert!(result.is_ok());
        let sr = result.unwrap();
        assert!(sr.hits.len() <= 5);
        if sr.hits.len() >= 2 {
            assert!(sr.hits[0].downloads >= sr.hits[1].downloads);
        }
    }

    #[tokio::test]
    async fn test_search_with_facets() {
        let p = SearchParams {
            query: "jei".to_string(),
            limit: Some(5),
            offset: None,
            index: None,
            facets: Some(vec![vec!["project_type:mod".to_string()]]),
        };
        let result = search_project(&p).await;
        assert!(result.is_ok());
        let sr = result.unwrap();
        for hit in &sr.hits {
            assert_eq!(hit.project_type, "mod");
        }
    }

    #[tokio::test]
    async fn test_search_with_or_facets() {
        let p = SearchParams {
            query: "sodium".to_string(),
            limit: Some(5),
            offset: None,
            index: None,
            facets: Some(vec![vec![
                "categories:fabric".to_string(),
                "categories:quilt".to_string(),
            ]]),
        };
        let result = search_project(&p).await;
        assert!(result.is_ok());
        let sr = result.unwrap();
        for hit in &sr.hits {
            let has_fabric_or_quilt = hit.categories.iter().any(|c| c == "fabric" || c == "quilt");
            assert!(has_fabric_or_quilt);
        }
    }

    #[tokio::test]
    async fn test_search_limit_clamping() {
        let p = SearchParams {
            query: "a".to_string(),
            limit: Some(200),
            offset: None,
            index: None,
            facets: None,
        };
        let result = search_project(&p).await;
        assert!(result.is_ok());
        let sr = result.unwrap();
        assert!(sr.hits.len() <= 100);
    }

    #[tokio::test]
    async fn test_search_result_fields_present() {
        let p = params("fabric-api", 3);
        let result = search_project(&p).await;
        assert!(result.is_ok());
        let sr = result.unwrap();
        assert!(sr.offset >= 0);
        assert!(sr.limit > 0);
        assert!(sr.total_hits >= sr.hits.len() as i32);
    }

    #[tokio::test]
    async fn test_get_project_dependencies_real_network_return_200() {
        // Iris depends on Sodium
        let result = get_project_dependencies("iris").await;
        assert!(result.is_ok(), "Failed to fetch project dependencies");
        let deps = result.unwrap();
        assert!(!deps.projects.is_empty(), "Iris should depend on Sodium");
        if let Some(project) = deps.projects.first() {
            assert!(!project.id.is_empty());
            assert!(!project.title.is_empty());
            assert!(!project.slug.is_empty());
        }
        if let Some(version) = deps.versions.first() {
            assert!(!version.id.is_empty());
            assert!(!version.project_id.is_empty());
            assert!(!version.files.is_empty());
        }
    }

    #[tokio::test]
    async fn test_get_project_real_network_return_200() {
        let result = get_project("P7dR8mSH").await;
        assert!(result.is_ok(), "Failed to fetch project");
        let project = result.unwrap();
        assert!(!project.id.is_empty());
        assert!(!project.title.is_empty());
        assert!(!project.slug.is_empty());
        assert_eq!(project.project_type, "mod");
        assert!(project.downloads > 0);
        assert!(!project.game_versions.is_empty());
    }

    #[tokio::test]
    async fn test_search_then_get_project() {
        // Search for fabric-api, get its project_id, then fetch the full project
        let p = params("fabric-api", 1);
        let search_result = search_project(&p).await;
        assert!(search_result.is_ok(), "Search failed");
        let sr = search_result.unwrap();
        assert!(!sr.hits.is_empty(), "Search should return at least one hit");
        let hit = &sr.hits[0];
        assert_eq!(hit.project_id, "P7dR8mSH");

        // Now fetch the full project by ID from search result
        let project = get_project(&hit.project_id).await;
        assert!(project.is_ok(), "get_project by search ID failed");
        let project = project.unwrap();
        assert_eq!(project.id, hit.project_id);
        assert_eq!(project.title, hit.title);
        assert!(project.followers > 0);
        assert!(project.license.is_some());
    }

    #[tokio::test]
    async fn test_search_with_offset() {
        let p = SearchParams {
            query: "a".to_string(),
            limit: Some(3),
            offset: Some(10),
            index: None,
            facets: None,
        };
        let result = search_project(&p).await;
        assert!(result.is_ok());
        let sr = result.unwrap();
        // Modrinth rounds offset down to the nearest multiple of limit
        assert!(sr.offset >= 6, "offset should skip a reasonable number");
        assert!(sr.hits.len() <= 3);
        assert!(sr.total_hits > sr.offset + sr.hits.len() as i32);
    }

    #[tokio::test]
    async fn test_get_project_dependencies_empty() {
        // fabric-api is a base library — it has no dependencies
        let result = get_project_dependencies("P7dR8mSH").await;
        assert!(result.is_ok(), "Request should succeed even with no deps");
        let deps = result.unwrap();
        assert!(deps.projects.is_empty());
        assert!(deps.versions.is_empty());
    }

    #[tokio::test]
    async fn test_get_project_by_slug() {
        let result = get_project("fabric-api").await;
        assert!(result.is_ok(), "Should resolve project by slug");
        let project = result.unwrap();
        assert_eq!(project.id, "P7dR8mSH");
        assert_eq!(project.slug, "fabric-api");
    }

    #[tokio::test]
    async fn test_get_project_dependencies_by_slug() {
        // Iris (slug "iris") depends on Sodium
        let result = get_project_dependencies("iris").await;
        assert!(result.is_ok(), "Should resolve dependencies by slug");
        let deps = result.unwrap();
        assert!(!deps.projects.is_empty(), "Iris should depend on Sodium");
    }

    #[tokio::test]
    async fn test_list_project_versions() {
        let params = ListVersionsParams {
            id_or_slug: "P7dR8mSH".to_string(),
            loaders: None,
            game_versions: None,
            featured: None,
            include_changelog: Some(false),
        };
        let result = list_project_versions(&params).await;
        assert!(result.is_ok(), "Failed to list versions");
        let versions = result.unwrap();
        assert!(!versions.is_empty(), "Fabric API should have versions");
        let first = &versions[0];
        assert!(!first.id.is_empty());
        assert!(!first.name.is_empty());
        assert!(!first.files.is_empty());
        assert!(!first.game_versions.is_empty());
    }

    #[tokio::test]
    async fn test_list_project_versions_filtered() {
        let params = ListVersionsParams {
            id_or_slug: "P7dR8mSH".to_string(),
            loaders: Some(vec!["fabric".to_string()]),
            game_versions: Some(vec!["1.21.4".to_string()]),
            featured: Some(true),
            include_changelog: Some(false),
        };
        let result = list_project_versions(&params).await;
        assert!(result.is_ok(), "Failed to list filtered versions");
        let versions = result.unwrap();
        assert!(
            !versions.is_empty(),
            "Should have at least one matching version"
        );
        for v in &versions {
            assert!(v.loaders.contains(&"fabric".to_string()));
            assert!(v.game_versions.contains(&"1.21.4".to_string()));
        }
    }

    #[tokio::test]
    async fn test_get_version_by_id() {
        let params = ListVersionsParams {
            id_or_slug: "P7dR8mSH".to_string(),
            loaders: None,
            game_versions: None,
            featured: None,
            include_changelog: Some(false),
        };
        let versions = list_project_versions(&params).await.unwrap();
        let version_id = &versions[0].id;

        let result = get_version(version_id).await;
        assert!(result.is_ok(), "Failed to get version by ID");
        let v = result.unwrap();
        assert_eq!(v.id, *version_id);
        assert!(!v.name.is_empty());
        assert!(!v.version_number.is_empty());
        assert!(!v.files.is_empty());
        assert!(!v.game_versions.is_empty());
    }

    #[tokio::test]
    async fn test_get_version_files() {
        let params = ListVersionsParams {
            id_or_slug: "P7dR8mSH".to_string(),
            loaders: None,
            game_versions: None,
            featured: None,
            include_changelog: Some(false),
        };
        let versions = list_project_versions(&params).await.unwrap();
        let version_id = &versions[0].id;

        let result = get_version_files(version_id).await;
        assert!(result.is_ok(), "Failed to download mod");
        let files = result.unwrap();
        assert!(!files.is_empty());
        let primary = files.iter().find(|f| f.primary).or(files.first());
        assert!(primary.is_some(), "Should have at least one file");
        assert!(!primary.unwrap().url.is_empty());
    }
}
