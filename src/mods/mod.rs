use crate::{cli::ModeArgs, mods::models::search_mods, version::AnyError};

pub mod models;

// TODO: will be implemented
pub async fn handle_mods(args: &ModeArgs) -> Result<(), AnyError> {
    if args.download {
        search_mods(&args.query).await?;
    }
    Ok(())
}
