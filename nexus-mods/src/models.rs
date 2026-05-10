use nexus_core::AnyError;
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SearchResult {
    pub hits: Vec<ModHit>,
    pub offset: i32,
    pub limit: i32,
    pub total_hits: i32,
}

#[derive(Deserialize, Debug)]
pub struct ModHit {
    pub project_id: String,
    pub title: String,
    pub author: String,
    pub description: String,
    pub categories: Vec<String>,
    pub client_side: String,
    pub server_side: String,
    pub project_type: String,
    pub downloads: i32,
    pub icon_url: String,
    /// A list of the minecraft versions supported by the project
    pub versions: Vec<String>,
    /// The total number of users following the project
    pub follows: i32,
    /// The date the project was added to search
    pub date_created: String,
    /// The date the project was last modified
    pub date_modified: String,
}

#[derive(Deserialize, Debug)]
pub struct ModVersion {
    pub files: Vec<ModFile>,
    pub total_hits: i32,
}

#[derive(Deserialize, Debug)]
pub struct ModVersionJson {
    /// The name of this version
    pub name: String,

    /// A list of versions of Minecraft that this version supports
    pub game_version: Vec<String>,

    /// The release channel for the version
    /// Allowed values: "release", "beta", "alpha"
    pub version_type: String,

    /// The mod loaders that this version supports. In case of resource packs, use “minecraft”
    pub loaders: Vec<String>,

    /// The ID of version
    pub id: String,

    /// The ID of the project this version is for
    pub project_id: String,

    /// The ID of the author who published this version
    pub author_id: String,

    pub date_publish: String,

    /// The number of times this version has been downloaded
    pub downloads: i32,

    pub files: Vec<ModFile>,
    pub dependencies: Vec<ModDependency>,
}

#[derive(Deserialize, Debug)]
pub struct Hashes {
    pub sha1: String,
    pub sha512: String,
}

#[derive(Deserialize, Debug)]
pub struct ModFile {
    pub hash: Hashes,
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: i32,
}

#[derive(Deserialize, Debug)]
pub struct ModDependency {
    pub project_id: String,
    pub version_id: String,
    pub file_name: String,
    /// Allowed values: "required", "optional", "incompatible", "embedded"
    pub dependency_type: String,
}

/// Parameters for searching mods on Modrinth.
///
/// See: <https://docs.modrinth.com/api/operations/searchprojects/>
pub struct SearchParams {
    /// The query string to search for.
    pub query: String,

    /// The number of results to return.
    ///
    /// Default: `10`. Max: `100` (values above 100 are clamped).
    pub limit: Option<i32>,

    /// The number of results to skip, for pagination.
    pub offset: Option<i32>,

    /// The sorting method for results.
    ///
    /// Allowed values: `"relevance"` (default), `"downloads"`, `"follows"`,
    /// `"newest"`, `"updated"`.
    pub index: Option<String>,

    /// Facet filters for narrowing results by project metadata.
    ///
    /// Each inner `Vec<String>` is an **OR** group — a result matches the
    /// group if it matches ANY facet within it. Different groups are joined
    /// by **AND** — a result must match at least one facet from EVERY group.
    ///
    /// Each facet string follows the format `"{type}{operation}{value}"`:
    ///
    /// | Example | Meaning |
    /// |---------|---------|
    /// | `"project_type:mod"` | Project type equals "mod" |
    /// | `"versions!=1.20.1"` | Version does not equal 1.20.1 |
    /// | `"downloads<=100"` | Downloads ≤ 100 |
    /// | `"categories:forge"` | Category equals "forge" |
    ///
    /// Supported facet types: `project_type`, `categories` (includes loaders),
    /// `versions`, `client_side`, `server_side`, `open_source`, `title`,
    /// `author`, `follows`, `project_id`, `license`, `downloads`, `color`,
    /// `created_timestamp`, `modified_timestamp`.
    ///
    /// Supported operations: `:` or `=` (equals), `!=`, `>=`, `>`, `<=`, `<`.
    pub facets: Option<Vec<Vec<String>>>,
}

/// Search for projects on Modrinth.
///
/// See: <https://docs.modrinth.com/api/operations/searchprojects/>
pub async fn search_project(params: &SearchParams) -> Result<SearchResult, AnyError> {
    let limit = params.limit.unwrap_or(10).min(100);
    let index = params.index.as_deref().unwrap_or("relevance");
    let offset = params.offset.unwrap_or(0);

    // URL-encode a JSON string for use as a query parameter value
    fn url_encode_json(json: &str) -> String {
        json.replace('%', "%25")
            .replace('[', "%5B")
            .replace(']', "%5D")
            .replace('"', "%22")
    }

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

// link: https://docs.modrinth.com/api/operations/getproject/
// TODO: Will be implemented
pub async fn get_version(_id: &String) {
    unimplemented!()
}

// link : https://docs.modrinth.com/api/operations/getprojectversions/
// TODO: Will be implemented
pub async fn get_project_versions(_id: &String) {
    unimplemented!()
}

// link: https://docs.modrinth.com/api/operations/getversion/
// SIrB5bCM
// TODO: Will be implemented
pub async fn download_mod(id: &String) -> Result<Vec<ModVersionJson>, AnyError> {
    let client = Client::new();
    let _url = format!("https://api.modrinth.com/v2/version/{}", id);

    let resp = client
        .get(_url)
        .header("User-Agent", "AuroBreeze/NexusLauncher/0.1.0")
        .send()
        .await?;

    let result = resp.json::<Vec<ModVersionJson>>().await?;
    tracing::info!("📦 Mod version: {:#?}", result);
    // unimplemented!()
    // Ok(result)
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

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

    // #[tokio::test]
    // async fn test_download_mods_real_network_return_200() {
    //     let id = "AANobbMI".to_string();
    //     let result = download_mod(&id).await;
    //     dbg!(&result);
    //     assert!(result.is_ok());
    // }

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
        // OR within a single group: (supports fabric OR quilt)
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
}
