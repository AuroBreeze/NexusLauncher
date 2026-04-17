pub mod fabric;
pub mod models;

use crate::fabric::{get_fabric_profile, get_latest_loader, install_fabric_libraries};
use nexus_cli::cli::LoaderArgs;
use nexus_version::AnyError;
use std::path::PathBuf;

// TODO: will be implemented
// The configuration file needs to be updated; most importantly, the persistence settings for the main function need to be saved.
pub async fn handle_loader(args: &LoaderArgs) -> Result<(), AnyError> {
    let loader_verison = get_latest_loader(&args.game_name).await;
    match loader_verison {
        Ok(v) => {
            tracing::info!("Latest Fabric Loader: {}", v);
            let profile = get_fabric_profile(&args.game_name, &v, &args.game_name).await?;
            let extra_classpath: Vec<PathBuf> =
                install_fabric_libraries(&profile, &args.game_name).await?;

            let main_class = profile.main_class;
            tracing::info!("Main Class: {}", main_class);
            tracing::info!("Libraries: {:#?}", extra_classpath);
        }
        Err(e) => {
            tracing::error!("Failed to fetch Fabric Loader: {}", e);
        }
    }
    Ok(())
}
