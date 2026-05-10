use crate::models::{ModVersionJson, SearchParams, SearchResult};
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
    Ok(result)
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
}
