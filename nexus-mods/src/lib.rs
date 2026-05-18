pub mod api;
pub mod models;

use crate::api::{download_mod_to_instance, search_project};
use crate::models::SearchParams;
use nexus_cli::cli::ModArgs;
use nexus_core::AnyError;

pub async fn handle_mods(args: &ModArgs) -> Result<(), AnyError> {
    tracing::debug!(
        "handle_mods: query={:?}, download={}, limit={:?}, game_version={:?}, loader={:?}, instance_name={:?}",
        args.query,
        args.download,
        args.limit,
        args.game_version,
        args.loader,
        args.instance_name
    );
    if let Some(query) = args.query.as_ref() {
        if args.download {
            let instance = args
                .instance_name
                .as_deref()
                .ok_or("--instance-name is required with --download")?;
            nexus_core::validate_instance_name(instance)?;
            let loader = args
                .loader
                .as_ref()
                .map(|l| l.to_string())
                .ok_or("--loader is required with --download")?;

            let game_version = match args.game_version.as_deref() {
                Some(gv) => Some(gv.to_string()),
                None => {
                    let version_json = nexus_core::get_clients_dir()
                        .join(instance)
                        .join("version.json");
                    tracing::debug!("Reading game version from {}", version_json.display());
                    let content = std::fs::read_to_string(&version_json)?;
                    let detail: serde_json::Value = serde_json::from_str(&content)?;
                    let resolved = detail["id"].as_str().map(|s| s.to_string());
                    tracing::debug!("Resolved game version: {:?}", resolved);
                    resolved
                }
            };

            tracing::info!(
                "Downloading mod '{}' (loader={}, game={}) to instance '{}'...",
                query,
                loader,
                game_version.as_deref().unwrap_or("latest"),
                instance
            );
            let dest = download_mod_to_instance(
                query,
                game_version.as_deref(),
                Some(&loader),
                args.version_type.as_deref(),
                instance,
            )
            .await?;
            tracing::info!("Mod installed to {}", dest.display());
        } else {
            let facets = args
                .game_version
                .as_ref()
                .map(|gv| vec![vec![format!("versions:{}", gv)]]);

            let params = SearchParams {
                query: query.clone(),
                limit: args.limit,
                offset: None,
                index: None,
                facets,
            };
            search_project(&params).await?;
        }
    }

    Ok(())
}
