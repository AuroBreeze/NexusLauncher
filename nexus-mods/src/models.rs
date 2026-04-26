use nexus_core::AnyError;
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

// link: https://docs.modrinth.com/api/operations/searchprojects/
// TODO: Make `limit` optional, and add requests for `feature` and `index`
pub async fn search_project(query: &str, limit: i32) -> Result<SearchResult, AnyError> {
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

    #[tokio::test]
    async fn test_search_mods_real_network_return_200() {
        // Define the search parameters
        let query = "fabric-api";
        let limit = 3;

        // Call the function
        let result = search_project(query, limit).await;

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

    // #[tokio::test]
    // async fn test_download_mods_real_network_return_200() {
    //     let id = "AANobbMI".to_string();
    //     let result = download_mod(&id).await;
    //     dbg!(&result);
    //     assert!(result.is_ok());
    // }
}
