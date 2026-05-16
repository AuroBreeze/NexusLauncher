use serde::{Deserialize, Serialize};

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
    /// Allowed values: `”relevance”` (default), `”downloads”`, `”follows”`,
    /// `”newest”`, `”updated”`.
    pub index: Option<String>,

    /// Facet filters for narrowing results by project metadata.
    ///
    /// Each inner `Vec<String>` is an **OR** group — a result matches the
    /// group if it matches ANY facet within it. Different groups are joined
    /// by **AND** — a result must match at least one facet from EVERY group.
    ///
    /// Each facet string follows the format `”{type}{operation}{value}”`:
    ///
    /// | Example | Meaning |
    /// |---------|---------|
    /// | `”project_type:mod”` | Project type equals “mod” |
    /// | `”versions!=1.20.1”` | Version does not equal 1.20.1 |
    /// | `”downloads<=100”` | Downloads ≤ 100 |
    /// | `”categories:forge”` | Category equals “forge” |
    ///
    /// Supported facet types: `project_type`, `categories` (includes loaders),
    /// `versions`, `client_side`, `server_side`, `open_source`, `title`,
    /// `author`, `follows`, `project_id`, `license`, `downloads`, `color`,
    /// `created_timestamp`, `modified_timestamp`.
    ///
    /// Supported operations: `:` or `=` (equals), `!=`, `>=`, `>`, `<=`, `<`.
    pub facets: Option<Vec<Vec<String>>>,
}

/// Parameters for listing a project's versions.
///
/// See: <https://docs.modrinth.com/api/operations/getprojectversions/>
pub struct ListVersionsParams {
    /// The ID or slug of the project.
    pub id_or_slug: String,

    /// Filter by mod loader types (e.g. `["fabric", "forge"]`).
    pub loaders: Option<Vec<String>>,

    /// Filter by game versions (e.g. `["1.18.1", "1.19"]`).
    pub game_versions: Option<Vec<String>>,

    /// Filter for featured (`true`) or non-featured (`false`) versions only.
    pub featured: Option<bool>,

    /// Include the changelog field in the response.
    ///
    /// Default: `true`. Set to `false` unless you specifically need all
    /// changelogs — it reduces response size significantly.
    pub include_changelog: Option<bool>,
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

/// A dependency declaration for a version.
#[derive(Deserialize, Debug)]
pub struct ModDependency {
    /// The ID of the depended-on project, if any (nullable).
    pub project_id: Option<String>,

    /// The ID of the depended-on version, if any (nullable).
    pub version_id: Option<String>,

    /// The filename of the dependency, if bundled (nullable).
    pub file_name: Option<String>,

    /// The type of this dependency.
    /// Allowed values: `”required”`, `”optional”`, `”incompatible”`, `”embedded”`.
    pub dependency_type: String,
}

// ============================================================
// Project Dependencies endpoint
// link: https://docs.modrinth.com/api/operations/getprojectdependencies/
// ============================================================

/// Response from `GET /project/{id|slug}/dependencies`.
///
/// See: <https://docs.modrinth.com/api/operations/getprojectdependencies/>
#[derive(Deserialize, Debug)]
pub struct ProjectDependencies {
    /// Projects that the project depends upon.
    pub projects: Vec<Project>,

    /// Versions that the project depends upon.
    pub versions: Vec<Version>,
}

/// A Modrinth project.
///
/// Returned by `GET /project/{id|slug}` and within
/// `GET /project/{id|slug}/dependencies`.
#[derive(Deserialize, Debug)]
pub struct Project {
    /// The slug of the project, used for vanity URLs.
    /// Regex: `^[\w!@$()`.+,"\-']{3,64}$`
    pub slug: String,

    /// The title or name of the project.
    pub title: String,

    /// A short description of the project.
    pub description: String,

    /// A list of the categories that the project has.
    pub categories: Vec<String>,

    /// The client-side support of the project.
    /// Allowed values: `"required"`, `"optional"`, `"unsupported"`, `"unknown"`.
    pub client_side: String,

    /// The server-side support of the project.
    /// Allowed values: `"required"`, `"optional"`, `"unsupported"`, `"unknown"`.
    pub server_side: String,

    /// A long-form description of the project.
    pub body: String,

    /// The status of the project.
    /// Allowed values: `"approved"`, `"archived"`, `"rejected"`, `"draft"`,
    /// `"unlisted"`, `"processing"`, `"withheld"`, `"scheduled"`,
    /// `"private"`, `"unknown"`.
    pub status: String,

    /// The requested status when submitting for review or scheduling for release.
    /// Nullable. Allowed values: `"approved"`, `"archived"`, `"unlisted"`,
    /// `"private"`, `"draft"`.
    pub requested_status: Option<String>,

    /// A list of categories which are searchable but non-primary.
    pub additional_categories: Vec<String>,

    /// An optional link to where to submit bugs or issues with the project.
    pub issues_url: Option<String>,

    /// An optional link to the source code of the project.
    pub source_url: Option<String>,

    /// An optional link to the project's wiki page or other relevant information.
    pub wiki_url: Option<String>,

    /// An optional invite link to the project's discord.
    pub discord_url: Option<String>,

    /// A list of donation links for the project.
    pub donation_urls: Vec<DonationUrl>,

    /// The project type.
    /// Allowed values: `"mod"`, `"modpack"`, `"resourcepack"`, `"shader"`.
    pub project_type: String,

    /// The total number of downloads of the project.
    pub downloads: i32,

    /// The URL of the project's icon.
    pub icon_url: Option<String>,

    /// The RGB color of the project, automatically generated from the project icon.
    pub color: Option<i32>,

    /// The ID of the moderation thread associated with this project.
    pub thread_id: String,

    /// The monetization status of the project.
    /// Allowed values: `"monetized"`, `"demonetized"`, `"force-demonetized"`.
    pub monetization_status: String,

    /// The ID of the project, encoded as a base62 string.
    pub id: String,

    /// The ID of the team that has ownership of this project.
    pub team: String,

    /// The link to the long description of the project.
    /// Always null — only kept for legacy compatibility.
    pub body_url: Option<String>,

    /// A message that a moderator sent regarding the project.
    pub moderator_message: Option<String>,

    /// The date the project was published (ISO-8601).
    pub published: String,

    /// The date the project was last updated (ISO-8601).
    pub updated: String,

    /// The date the project's status was set to an approved status (ISO-8601).
    pub approved: Option<String>,

    /// The date the project's status was submitted to moderators for review (ISO-8601).
    pub queued: Option<String>,

    /// The total number of users following the project.
    pub followers: i32,

    /// The license of the project.
    pub license: Option<License>,

    /// A list of the version IDs of the project (never empty unless draft status).
    pub versions: Vec<String>,

    /// A list of all of the game versions supported by the project.
    pub game_versions: Vec<String>,

    /// A list of all of the loaders supported by the project.
    pub loaders: Vec<String>,

    /// A list of images that have been uploaded to the project's gallery.
    pub gallery: Vec<GalleryImage>,
}

/// The license of a project.
#[derive(Deserialize, Debug)]
pub struct License {
    /// The SPDX identifier of the license (e.g. `"Apache-2.0"`, `"MIT"`).
    pub id: String,

    /// The human-readable name of the license.
    pub name: String,

    /// A URL pointing to the full license text.
    pub url: Option<String>,
}

/// A donation link for a project.
#[derive(Deserialize, Debug)]
pub struct DonationUrl {
    /// The platform identifier (e.g. `"patreon"`, `"ko-fi"`, `"github"`).
    pub platform: String,

    /// The donation URL.
    pub url: String,
}

/// An image in a project's gallery.
#[derive(Deserialize, Debug)]
pub struct GalleryImage {
    /// The URL of the gallery image.
    pub url: String,

    /// Whether this image was automatically generated.
    pub featured: bool,

    /// The title of the image.
    pub title: Option<String>,

    /// The description of the image.
    pub description: Option<String>,

    /// The date the image was created (ISO-8601).
    pub created: String,

    /// The ordering of the image in the gallery.
    pub ordering: i32,
}

/// A Modrinth project version.
///
/// Returned by `GET /project/{id|slug}/version`, `GET /version/{id}`,
/// and within `GET /project/{id|slug}/dependencies`.
#[derive(Deserialize, Debug)]
pub struct Version {
    /// The name of this version.
    pub name: String,

    /// The version number. Ideally follows semantic versioning.
    pub version_number: String,

    /// The changelog for this version.
    pub changelog: Option<String>,

    /// A list of specific versions of projects that this version depends on.
    pub dependencies: Vec<ModDependency>,

    /// A list of versions of Minecraft that this version supports.
    pub game_versions: Vec<String>,

    /// The release channel for this version.
    /// Allowed values: `"release"`, `"beta"`, `"alpha"`.
    pub version_type: String,

    /// The mod loaders that this version supports.
    /// For resource packs, use `"minecraft"`.
    pub loaders: Vec<String>,

    /// Whether the version is featured.
    pub featured: bool,

    /// The status of the version.
    /// Allowed values: `"listed"`, `"archived"`, `"draft"`, `"unlisted"`,
    /// `"scheduled"`, `"unknown"`.
    pub status: String,

    /// The requested status of the version.
    /// Nullable. Allowed values: `"listed"`, `"archived"`, `"draft"`, `"unlisted"`.
    pub requested_status: Option<String>,

    /// The ID of the version, encoded as a base62 string.
    pub id: String,

    /// The ID of the project this version is for.
    pub project_id: String,

    /// The ID of the author who published this version.
    pub author_id: String,

    /// The date this version was published (ISO-8601).
    pub date_published: String,

    /// The number of times this version has been downloaded.
    pub downloads: i32,

    /// A link to the changelog for this version. Always null — legacy compatibility.
    pub changelog_url: Option<String>,

    /// A list of files available for download for this version.
    pub files: Vec<VersionFile>,
}

/// A file belonging to a [`Version`].
#[derive(Deserialize, Debug)]
pub struct VersionFile {
    /// The unique identifier of this file.
    pub id: String,

    /// A map of hashes of the file. Keys are hashing algorithms (e.g. `"sha1"`, `"sha512"`).
    pub hashes: Hashes,

    /// A direct link to the file.
    pub url: String,

    /// The name of the file.
    pub filename: String,

    /// Whether this file is the primary one for its version.
    /// At most one file per version will have this set to true.
    pub primary: bool,

    /// The size of the file in bytes.
    pub size: i32,

    /// The type of this file.
    /// Allowed values: `"required-resource-pack"`, `"optional-resource-pack"`,
    /// `"sources-jar"`, `"dev-jar"`, `"javadoc-jar"`, `"unknown"`.
    pub file_type: Option<String>,

    /// A file signature, if available.
    pub signature: Option<String>,
}

// ============================================================
// Mod manifest (per-instance download tracking)
// ============================================================

/// A record of an installed mod.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModEntry {
    pub name: String,
    pub project_id: String,
    pub version_number: String,
    pub version_type: String,
    pub filename: String,
    pub loader: String,
    pub game_version: String,
    pub installed_at: String,
    pub dependencies: Vec<DepEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DepEntry {
    pub name: Option<String>,
    pub project_id: Option<String>,
    pub version_id: Option<String>,
    pub dependency_type: String,
}

/// Manifest of installed mods, stored as TOML in `<instance>/mods/nexus_mods.toml`.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ModManifest {
    pub mods: Vec<ModEntry>,
}

impl ModManifest {
    pub fn load(instance_name: &str) -> Self {
        let path = nexus_core::get_clients_dir()
            .join(instance_name)
            .join("mods")
            .join("nexus_mods.toml");
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| {
                toml::from_str(&s)
                    .map_err(|e| {
                        tracing::warn!("Failed to parse mod manifest at {}: {}", path.display(), e);
                    })
                    .ok()
            })
            .unwrap_or_default()
    }

    pub fn save(&self, instance_name: &str) -> std::io::Result<()> {
        let dir = nexus_core::get_clients_dir()
            .join(instance_name)
            .join("mods");
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("nexus_mods.toml");
        let content = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, content)
    }
}
