pub mod models;

use crate::models::search_project;
use nexus_cli::cli::ModArgs;
use nexus_core::AnyError;

// TODO: will be implemented
pub async fn handle_mods(args: &ModArgs) -> Result<(), AnyError> {
    if let Some(query) = args.query.as_ref() {
        search_project(query, args.limit.unwrap()).await?;
    }

    Ok(())
}
