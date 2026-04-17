use crate::{cli::ModArgs, mods::models::search_mods, version::AnyError};

pub mod models;

// TODO: will be implemented
pub async fn handle_mods(args: &ModArgs) -> Result<(), AnyError> {
    if let Some(query) = args.query.as_ref() {
        search_mods(query, args.limit.unwrap()).await?;
    }

    Ok(())
}
