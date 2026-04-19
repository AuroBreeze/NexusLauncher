pub mod models;

use crate::models::search_mods;
use nexus_cli::cli::ModArgs;
use nexus_core::AnyError;

// TODO: will be implemented
pub async fn handle_mods(args: &ModArgs) -> Result<(), AnyError> {
    if let Some(query) = args.query.as_ref() {
        search_mods(query, args.limit.unwrap()).await?;
    }

    Ok(())
}
