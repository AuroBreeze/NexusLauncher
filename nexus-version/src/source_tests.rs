use super::*;

#[tokio::test]
async fn test_version_manifest_deserialization() {
    let manifest = obtain_manifest().await.unwrap();
    assert!(
        !manifest.latest.release.is_empty(),
        "latest.release missing"
    );
    assert!(
        !manifest.latest.snapshot.is_empty(),
        "latest.snapshot missing"
    );
    assert!(!manifest.versions.is_empty(), "versions empty");
    let first = &manifest.versions[0];
    assert!(!first.id.is_empty(), "version.id missing");
    assert!(!first.url.is_empty(), "version.url missing");
    assert!(
        !first.release_time.is_empty(),
        "version.releaseTime missing"
    );
}

#[tokio::test]
async fn test_version_detail_deserialization() {
    let manifest = obtain_manifest().await.unwrap();
    let v = manifest
        .versions
        .iter()
        .find(|v| v.id == "1.21.4")
        .expect("1.21.4 not found");
    let detail = fetch_version_detail(&v.url).await.unwrap();
    assert_eq!(detail.id, "1.21.4");
    assert!(!detail.main_class.is_empty());
    assert!(!detail.downloads.client.url.is_empty());
    assert!(!detail.downloads.client.sha1.is_empty());
    assert!(detail.java_version.major_version > 0);
    assert!(!detail.libraries.is_empty());
    let lib = &detail.libraries[0];
    assert!(!lib.name.is_empty());
    assert!(lib.downloads.artifact.is_some());
}

#[tokio::test]
async fn test_asset_index_deserialization() {
    let manifest = obtain_manifest().await.unwrap();
    let v = manifest
        .versions
        .iter()
        .find(|v| v.id == "1.21.4")
        .expect("1.21.4 not found");
    let detail = fetch_version_detail(&v.url).await.unwrap();
    let response = reqwest::get(&detail.asset_index.url).await.unwrap();
    let index: AssetIndexManifest = response.json().await.unwrap();
    assert!(!index.objects.is_empty());
    let (_, obj) = index.objects.iter().next().unwrap();
    assert!(!obj.hash.is_empty());
    assert!(obj.size > 0);
}
