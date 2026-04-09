use crate::{cli::ModArgs, mods::models::search_mods, version::AnyError};

pub mod models;

// TODO: will be implemented
pub async fn handle_mods(args: &ModArgs) -> Result<(), AnyError> {
    if args.download {
        search_mods(&args.query).await?;
    }
    Ok(())
}
