// src/auth.rs
use serde::Deserialize;

pub const CLIENT_ID: &str = "97b1e314-6463-4a4c-98c9-b4c5dbcc114f";

#[derive(Deserialize, Debug)]
pub struct DeviceCodeResponse {
    pub user_code: String,
    pub device_code: String,
    pub verification_uri: String,
    pub interval: u64,
    pub expires_in: u64,
}

#[derive(Deserialize, Debug)]
pub struct MicrosoftToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct XboxLiveResponse {
    pub token: String,
    pub display_claims: DisplayClaims,
}

#[derive(Deserialize, Debug)]
pub struct DisplayClaims {
    pub xui: Vec<Xui>,
}

#[derive(Deserialize, Debug)]
pub struct Xui {
    pub uhs: String,
}

#[derive(Deserialize, Debug)]
pub struct MinecraftAuthResponse {
    pub access_token: String,
    pub username: String,
    #[serde(rename = "roles")]
    pub _roles: Vec<String>,
    pub expires_in: u32,
}

#[derive(Deserialize, Debug)]
pub struct MinecraftProfile {
    pub id: String,
    pub name: String,
}
