pub mod api;
pub mod models;

use crate::api::search_project;
use crate::models::SearchParams;
use nexus_cli::cli::ModArgs;
use nexus_core::AnyError;

pub async fn handle_mods(args: &ModArgs) -> Result<(), AnyError> {
    if let Some(query) = args.query.as_ref() {
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

    Ok(())
}
