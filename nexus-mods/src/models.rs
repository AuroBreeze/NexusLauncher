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
