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

#[derive(Deserialize, Debug)]
pub struct FabricProfile {
    #[serde(rename = "mainClass")]
    pub main_class: String,
    pub libraries: Vec<FabricLibrary>,
}

#[derive(Deserialize, Debug)]
pub struct FabricLibrary {
    pub name: String,
    pub url: String,
}
