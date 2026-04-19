pub mod config;
pub mod launchconfig;
pub mod models;
pub mod userconfig;

use crate::config::Config;
use crate::models::{LaunchConfig, UserConfig};
use nexus_cli::cli::SetArgs;
use nexus_core::AnyError;

pub async fn handle_set(args: &SetArgs) -> Result<(), AnyError> {
    let mut config = UserConfig::load().await;
    let mut launch_config = LaunchConfig::load().await;

    if let Some(username) = args.name.as_ref() {
        config.user_profile.offline.username = username.clone();
    }

    if let Some(uuid) = args.uuid.as_ref() {
        config.user_profile.offline.uuid = uuid.clone();
    }

    if let Some(judge) = args.offline {
        launch_config.offline = judge;
    }

    if args.show {
        tracing::info!("Offline profile: {:#?}", config.user_profile.offline);
    }

    config.save().await?;
    launch_config.save().await?;
    Ok(())
}
