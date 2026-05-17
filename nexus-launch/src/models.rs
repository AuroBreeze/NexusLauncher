// src/launch/models.rs
use std::path::PathBuf;

use nexus_core::get_library_path;
use nexus_loader::models::FabricProfile;
use nexus_version::models::VersionDetail;

#[derive(Debug, Clone, Default)]
pub struct LaunchContext {
    pub game_path: PathBuf,
    pub version_id: String,         // such as "1.20.1-fabric"
    pub java_path: Option<PathBuf>, // Path to the verified Java executable file
    pub user: UserContext,
    pub max_memory: Option<u32>,

    // The Classpath and the Main Class and other parameters
    pub main_class: String,
    pub libraries: Vec<PathBuf>,
    pub asset_index_id: String,
    pub fabric_loader: Option<FabricProfile>,
}

impl LaunchContext {
    /// Build a LaunchContext from parsed version metadata.
    pub fn from_detail(
        game_path: PathBuf,
        detail: &VersionDetail,
        user: UserContext,
        java_path: PathBuf,
        max_memory: u32,
        fabric_loader: Option<FabricProfile>,
    ) -> Self {
        LaunchContext {
            version_id: detail.id.clone(),
            game_path,
            java_path: Some(java_path),
            user,
            max_memory: Some(max_memory),
            main_class: detail.main_class.clone(),
            libraries: detail
                .libraries
                .iter()
                .filter_map(|lib| {
                    let artifact = lib.downloads.artifact.as_ref()?;
                    Some(get_library_path(&artifact.path))
                })
                .collect(),
            asset_index_id: detail.asset_index.id.clone(),
            fabric_loader,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct UserContext {
    pub username: String,
    pub uuid: String,
    pub access_token: Option<String>, // The access token for authentication
}

impl UserContext {
    pub fn new(username: String, uuid: String, access_token: String) -> Self {
        UserContext {
            username,
            uuid,
            access_token: Some(access_token),
        }
    }
}
