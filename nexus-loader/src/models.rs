// src/loader/fabric.rs
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct FabricLoaderResponse {
    pub loader: FabricLoader,
}

#[derive(Deserialize, Debug)]
pub struct FabricLoader {
    pub version: String,
    pub stable: bool,
}

/// Response from /v2/versions/loader (all versions, no game version filter).
#[derive(Deserialize, Debug)]
pub struct FabricLoaderVersion {
    pub version: String,
    pub stable: bool,
}

#[derive(Deserialize, Debug)]
pub struct QuiltLoaderVersion {
    pub version: String,
    pub maven: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FabricProfile {
    #[serde(rename = "mainClass")]
    pub main_class: String,
    pub libraries: Vec<FabricLibrary>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FabricLibrary {
    pub name: String,
    pub url: String,
}
