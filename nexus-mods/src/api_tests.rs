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

// ============================================================
// Search tests
// ============================================================

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
    assert!(sr.offset >= 6, "offset should skip a reasonable number");
    assert!(sr.hits.len() <= 3);
    assert!(sr.total_hits > sr.offset + sr.hits.len() as i32);
}

#[tokio::test]
async fn test_search_project_no_results() {
    let p = SearchParams {
        query: "xyznonexistentmod12345".to_string(),
        limit: Some(5),
        offset: None,
        index: None,
        facets: None,
    };
    let result = search_project(&p).await;
    assert!(result.is_ok());
    let sr = result.unwrap();
    assert!(sr.hits.is_empty());
    assert_eq!(sr.total_hits, 0);
}

// ============================================================
// Get project / dependencies tests
// ============================================================

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
async fn test_get_project_by_slug() {
    let result = get_project("fabric-api").await;
    assert!(result.is_ok(), "Should resolve project by slug");
    let project = result.unwrap();
    assert_eq!(project.id, "P7dR8mSH");
    assert_eq!(project.slug, "fabric-api");
}

#[tokio::test]
async fn test_get_project_invalid_id() {
    let result = get_project("nonexistent-slug-12345").await;
    assert!(result.is_err(), "Invalid project should return an error");
}

#[tokio::test]
async fn test_search_then_get_project() {
    let p = params("fabric-api", 1);
    let search_result = search_project(&p).await;
    assert!(search_result.is_ok(), "Search failed");
    let sr = search_result.unwrap();
    assert!(!sr.hits.is_empty(), "Search should return at least one hit");
    let hit = &sr.hits[0];
    assert_eq!(hit.project_id, "P7dR8mSH");

    let project = get_project(&hit.project_id).await;
    assert!(project.is_ok(), "get_project by search ID failed");
    let project = project.unwrap();
    assert_eq!(project.id, hit.project_id);
    assert_eq!(project.title, hit.title);
    assert!(project.followers > 0);
    assert!(project.license.is_some());
}

#[tokio::test]
async fn test_get_project_dependencies_real_network_return_200() {
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
async fn test_get_project_dependencies_empty() {
    let result = get_project_dependencies("P7dR8mSH").await;
    assert!(result.is_ok(), "Request should succeed even with no deps");
    let deps = result.unwrap();
    assert!(deps.projects.is_empty());
    assert!(deps.versions.is_empty());
}

#[tokio::test]
async fn test_get_project_dependencies_by_slug() {
    let result = get_project_dependencies("iris").await;
    assert!(result.is_ok(), "Should resolve dependencies by slug");
    let deps = result.unwrap();
    assert!(!deps.projects.is_empty(), "Iris should depend on Sodium");
}

#[tokio::test]
async fn test_get_project_dependencies_invalid_id() {
    let result = get_project_dependencies("nonexistent-slug-12345").await;
    assert!(result.is_err(), "Invalid project should return an error");
}

// ============================================================
// Version tests
// ============================================================

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

// ============================================================
// Pure function tests
// ============================================================

#[test]
fn test_url_encode_json_brackets_and_quotes() {
    let input = r#"[["categories:fabric"],["versions:1.17.1"]]"#;
    let encoded = url_encode_json(input);
    assert!(!encoded.contains('['));
    assert!(!encoded.contains(']'));
    assert!(!encoded.contains('"'));
    assert!(encoded.contains("%5B"));
    assert!(encoded.contains("%5D"));
    assert!(encoded.contains("%22"));
}

#[test]
fn test_url_encode_json_percent() {
    assert_eq!(url_encode_json("%"), "%25");
    assert_eq!(url_encode_json("%5B"), "%255B");
}

#[test]
fn test_url_encode_json_no_special_chars() {
    assert_eq!(url_encode_json("hello"), "hello");
}

#[test]
fn test_json_array_single() {
    assert_eq!(json_array(&["fabric".to_string()]), "[\"fabric\"]");
}

#[test]
fn test_json_array_multiple() {
    assert_eq!(
        json_array(&["fabric".to_string(), "forge".to_string()]),
        "[\"fabric\",\"forge\"]"
    );
}

#[test]
fn test_json_array_empty() {
    assert_eq!(json_array(&[]), "[]");
}

// ============================================================
// Model deserialization smoke tests
// ============================================================

#[tokio::test]
async fn test_mod_hit_fields_deserialize() {
    let p = params("fabric-api", 1);
    let sr = search_project(&p).await.unwrap();
    let hit = &sr.hits[0];
    assert!(!hit.project_id.is_empty());
    assert!(!hit.title.is_empty());
    assert!(!hit.author.is_empty());
    assert!(!hit.description.is_empty());
    assert!(!hit.categories.is_empty());
    assert!(!hit.client_side.is_empty());
    assert!(!hit.server_side.is_empty());
    assert!(!hit.project_type.is_empty());
    assert!(hit.downloads > 0);
    assert!(!hit.icon_url.is_empty());
    assert!(!hit.versions.is_empty());
    assert!(hit.follows > 0);
    assert!(!hit.date_created.is_empty());
    assert!(!hit.date_modified.is_empty());
}

#[tokio::test]
async fn test_project_fields_deserialize() {
    let project = get_project("P7dR8mSH").await.unwrap();
    assert_eq!(project.id, "P7dR8mSH");
    assert!(!project.slug.is_empty());
    assert!(!project.title.is_empty());
    assert!(!project.description.is_empty());
    assert!(!project.categories.is_empty());
    assert!(!project.client_side.is_empty());
    assert!(!project.server_side.is_empty());
    assert!(!project.body.is_empty());
    assert!(!project.status.is_empty());
    assert!(!project.project_type.is_empty());
    assert!(project.downloads > 0);
    assert!(!project.team.is_empty());
    assert!(!project.published.is_empty());
    assert!(!project.updated.is_empty());
    assert!(project.followers > 0);
    assert!(project.license.is_some());
    assert!(!project.game_versions.is_empty());
    assert!(!project.loaders.is_empty());
    assert!(!project.versions.is_empty());
}

#[tokio::test]
async fn test_version_fields_deserialize() {
    let params = ListVersionsParams {
        id_or_slug: "P7dR8mSH".to_string(),
        loaders: None,
        game_versions: None,
        featured: None,
        include_changelog: Some(false),
    };
    let versions = list_project_versions(&params).await.unwrap();
    let v = &versions[0];
    assert!(!v.id.is_empty());
    assert!(!v.project_id.is_empty());
    assert!(!v.author_id.is_empty());
    assert!(!v.name.is_empty());
    assert!(!v.version_number.is_empty());
    assert!(!v.version_type.is_empty());
    assert!(!v.game_versions.is_empty());
    assert!(!v.loaders.is_empty());
    assert!(!v.date_published.is_empty());
    assert!(v.downloads > 0);
    assert!(!v.files.is_empty());
    let f = &v.files[0];
    assert!(!f.id.is_empty());
    assert!(!f.url.is_empty());
    assert!(!f.filename.is_empty());
    assert!(f.size > 0);
}
