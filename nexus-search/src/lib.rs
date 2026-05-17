use nexus_cli::cli::{SearchCoreArgs, SearchJavaArgs, SearchModArgs, SearchUserArgs};
use nexus_config::config::Config;
use nexus_config::models::LaunchConfig;
use nexus_core::{AnyError, get_clients_dir};
use nexus_java::java::scan_local_java_environments;
use nexus_mods::api::search_project;
use nexus_mods::models::SearchParams;
use nexus_version::source::obtain_manifest;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct UserCacheEntry {
    name: String,
    uuid: String,
    #[serde(rename = "expiresOn")]
    expires_on: String,
}

pub async fn handle_search_core(args: &SearchCoreArgs) -> Result<(), AnyError> {
    let manifest = obtain_manifest().await?;
    let mut results: Vec<_> = manifest
        .versions
        .iter()
        .filter(|v| {
            if args.stable && v.version_type != "release" {
                return false;
            }
            if let Some(filter) = &args.version {
                return v.id.starts_with(filter);
            }
            true
        })
        .take(args.limit)
        .collect();

    results.sort_by(|a, b| b.id.cmp(&a.id));

    tracing::info!(
        "Found {} version(s){}:",
        results.len(),
        if args.stable { " (stable only)" } else { "" }
    );
    for v in &results {
        tracing::info!("  {} ({})", v.id, v.version_type);
    }

    Ok(())
}

pub async fn handle_search_mod(args: &SearchModArgs) -> Result<(), AnyError> {
    let facets = args
        .game_version
        .as_ref()
        .map(|gv| vec![vec![format!("versions:{}", gv)]]);

    let params = SearchParams {
        query: args.query.clone(),
        limit: Some(args.limit),
        offset: args.offset,
        index: args.index.clone(),
        facets,
    };
    search_project(&params).await?;
    Ok(())
}

pub async fn handle_search_java(args: &SearchJavaArgs) -> Result<(), AnyError> {
    if args.scan {
        tracing::info!("Scanning for installed Java runtimes...");
        let javas = scan_local_java_environments(None).await;

        let mut config = LaunchConfig::load().await;
        for j in &javas {
            config.java_paths.insert(j.major_version, j.path.clone());
        }
        config.save().await?;

        tracing::info!("Found {} Java installation(s):", javas.len());
        for j in &javas {
            if let Some(filter) = args.version
                && j.major_version != filter
            {
                continue;
            }
            tracing::info!(
                "  Java {} ({}) → {}",
                j.major_version,
                j.full_version,
                j.path.display()
            );
        }
    } else {
        let config = LaunchConfig::load().await;
        if config.java_paths.is_empty() {
            tracing::warn!("No cached Java installations. Run with --scan to search the system.");
            return Ok(());
        }

        let mut versions: Vec<_> = config.java_paths.iter().collect();
        versions.sort_by_key(|(v, _)| *v);

        tracing::info!("Cached Java installation(s):",);
        for (major_version, path) in &versions {
            if let Some(filter) = args.version
                && *major_version != &filter
            {
                continue;
            }
            tracing::info!("  Java {} → {}", major_version, path.display());
        }
    }

    Ok(())
}

pub async fn handle_search_user(args: &SearchUserArgs) -> Result<(), AnyError> {
    let path = get_clients_dir()
        .join(&args.instance)
        .join("usercache.json");

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => {
            tracing::warn!(
                "usercache.json not found in instance '{}'. \
                 Launch the game and join a world first.",
                args.instance
            );
            return Ok(());
        }
    };

    let entries: Vec<UserCacheEntry> = serde_json::from_str(&content)?;

    if entries.is_empty() {
        tracing::info!("No cached user profiles in instance '{}'.", args.instance);
        return Ok(());
    }

    tracing::info!(
        "Found {} cached profile(s) in instance '{}':",
        entries.len(),
        args.instance
    );
    for (i, entry) in entries.iter().enumerate() {
        tracing::info!(
            "  [{}] name: {}, uuid: {}, expires: {}",
            i + 1,
            entry.name,
            entry.uuid,
            entry.expires_on
        );
    }

    Ok(())
}
