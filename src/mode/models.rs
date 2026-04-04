// src/mod_manager.rs
use crate::AnyError;
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
}

#[derive(Deserialize, Debug)]
pub struct ModVersion {
    pub files: Vec<ModFile>,
}

#[derive(Deserialize, Debug)]
pub struct ModFile {
    pub url: String,
    pub filename: String,
}

pub async fn search_mods(query: &str) -> Result<SearchResult, AnyError> {
    let client = Client::new();
    let url = format!("https://api.modrinth.com/v2/search?query={}&limit={}", query, 3);

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
