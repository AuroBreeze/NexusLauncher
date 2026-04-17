// src/mod_manager.rs
use nexus_version::AnyError;
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SearchResult {
    pub hits: Vec<ModHit>,
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
pub struct ModFile {
    pub url: String,
    pub filename: String,
}

pub async fn search_mods(query: &str, limit: i32) -> Result<SearchResult, AnyError> {
    let client = Client::new();
    let url = format!(
        "https://api.modrinth.com/v2/search?query={}&limit={}",
        query, limit
    );

    let resp = client
        .get(url)
        .header("User-Agent", "AuroBreeze/NexusLauncher/0.1.0")
        .send()
        .await?;
    let result = resp.json::<SearchResult>().await?;
    tracing::info!("📦 Found {} mods matching your query", result.hits.len());
    tracing::info!("📦 Mod list: {:#?}", result.hits);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_mods_real_network_return_200() {
        // Define the search parameters
        let query = "fabric-api";
        let limit = 3;

        // Call the function
        let result = search_mods(query, limit).await;

        // Ensure the request was successful
        assert!(result.is_ok(), "Failed to fetch from Modrinth API");

        let search_result = result.unwrap();

        // Check if the number of returned items does not exceed the limit
        assert!(search_result.hits.len() <= limit as usize);

        // If the API returns data, verify the fields are correctly deserialized
        if !search_result.hits.is_empty() {
            let first_hit = &search_result.hits[0];
            assert!(
                !first_hit.project_id.is_empty(),
                "Project ID should not be empty"
            );
            assert!(!first_hit.title.is_empty(), "Title should not be empty");
        }
    }
}
