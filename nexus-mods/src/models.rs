use serde::Deserialize;

// ============================================================
// Search endpoint — request parameters & response
// link: https://docs.modrinth.com/api/operations/searchprojects/
// ============================================================

/// Response from `GET /v2/search`.
///
/// See: <https://docs.modrinth.com/api/operations/searchprojects/>
#[derive(Deserialize, Debug)]
pub struct SearchResult {
    /// The list of search results.
    pub hits: Vec<ModHit>,

    /// The number of results that were skipped by the query.
    pub offset: i32,

    /// The number of results returned by the query.
    pub limit: i32,

    /// The total number of results that match the query.
    pub total_hits: i32,
}

/// A single project returned in search results.
#[derive(Deserialize, Debug)]
pub struct ModHit {
    /// The unique identifier of the project.
    pub project_id: String,

    /// The display name of the project.
    pub title: String,

    /// The username of the project author.
    pub author: String,

    /// A short description of the project.
    pub description: String,

    /// The categories this project belongs to.
    /// Note: mod loaders (e.g. fabric, forge, quilt) are included in
    /// categories rather than in a separate field.
    pub categories: Vec<String>,

    /// The client-side support of the project.
    /// Allowed values: `”required”`, `”optional”`, `”unsupported”`.
    pub client_side: String,

    /// The server-side support of the project.
    /// Allowed values: `”required”`, `”optional”`, `”unsupported”`.
    pub server_side: String,

    /// The type of the project.
    /// Allowed values: `”mod”`, `”modpack”`, `”resourcepack”`, `”shader”`,
    /// `”plugin”`, `”datapack”`.
    pub project_type: String,

    /// The total number of downloads across all files.
    pub downloads: i32,

    /// A URL to the project's icon image.
    pub icon_url: String,

    /// A list of the Minecraft versions supported by the project.
    pub versions: Vec<String>,

    /// The total number of users following the project.
    pub follows: i32,

    /// The date the project was added to search (ISO-8601 timestamp).
    pub date_created: String,

    /// The date the project was last modified (ISO-8601 timestamp).
    pub date_modified: String,
}

/// Error response from Modrinth API (HTTP 400).
///
/// See: <https://docs.modrinth.com/api/operations/searchprojects/>
#[derive(Deserialize, Debug)]
pub struct ModrinthError {
    /// The name of the error (e.g. `”invalid_input”`).
    pub error: String,

    /// A human-readable description of the error.
    pub description: String,
}

// ============================================================
// Version / Project endpoints
// ============================================================

/// Wrapper returned by project-version listing endpoints.
#[derive(Deserialize, Debug)]
pub struct ModVersion {
    /// The version files available for download.
    pub files: Vec<ModFile>,

    /// The total number of versions matching the request.
    pub total_hits: i32,
}

/// A single version of a project.
///
/// Returned by `GET /v2/version/{id}`.
/// See: <https://docs.modrinth.com/api/operations/getversion/>
#[derive(Deserialize, Debug)]
pub struct ModVersionJson {
    /// The display name of this version.
    pub name: String,

    /// A list of Minecraft versions that this version supports.
    pub game_version: Vec<String>,

    /// The release channel for this version.
    /// Allowed values: `”release”`, `”beta”`, `”alpha”`.
    pub version_type: String,

    /// The mod loaders that this version supports.
    /// For resource packs, use `”minecraft”`.
    pub loaders: Vec<String>,

    /// The unique identifier of this version.
    pub id: String,

    /// The ID of the project this version belongs to.
    pub project_id: String,

    /// The ID of the author who published this version.
    pub author_id: String,

    /// The date this version was published (ISO-8601 timestamp).
    pub date_publish: String,

    /// The number of times this version has been downloaded.
    pub downloads: i32,

    /// The files belonging to this version.
    pub files: Vec<ModFile>,

    /// The dependencies declared by this version.
    pub dependencies: Vec<ModDependency>,
}

// ============================================================
// Shared sub-structs
// ============================================================

/// SHA-1 and SHA-512 hashes for a file.
#[derive(Deserialize, Debug)]
pub struct Hashes {
    /// The SHA-1 hash of the file contents (hex-encoded).
    pub sha1: String,

    /// The SHA-512 hash of the file contents (hex-encoded).
    pub sha512: String,
}

/// A downloadable file belonging to a version.
#[derive(Deserialize, Debug)]
pub struct ModFile {
    /// The SHA-1 and SHA-512 hashes of the file.
    pub hash: Hashes,

    /// The direct download URL for this file.
    pub url: String,

    /// The original filename of this file.
    pub filename: String,

    /// Whether this file is the primary (recommended) download.
    pub primary: bool,

    /// The size of the file in bytes.
    pub size: i32,
}

/// A dependency declaration for a version.
#[derive(Deserialize, Debug)]
pub struct ModDependency {
    /// The ID of the depended-on project, if any.
    pub project_id: String,

    /// The ID of the depended-on version, if any.
    pub version_id: String,

    /// The filename of the dependency, if bundled.
    pub file_name: String,

    /// The type of this dependency.
    /// Allowed values: `”required”`, `”optional”`, `”incompatible”`, `”embedded”`.
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
