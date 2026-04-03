use super::models::{CLIENT_ID, DeviceCodeResponse, MicrosoftToken};
use crate::version::AnyError;
use reqwest::Client;
use std::error::Error;

// src/auth/utils.rs

pub async fn get_device_code() -> Result<DeviceCodeResponse, AnyError> {
    let client = Client::new();
    let res = client
        .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode")
        .form(&[
            ("client_id", CLIENT_ID),
            ("scope", "XboxLive.signin offline_access"),
        ])
        .send()
        .await?;

    let status = res.status();
    let text = res.text().await?;
    if !status.is_success() {
        tracing::error!("Microsoft interface error ({}): {}", status, text);
        return Err(format!("Microsoft Error: {}", text).into());
    }
    // ------------------

    let device_code_res: DeviceCodeResponse = serde_json::from_str(&text)?;
    Ok(device_code_res)
}

pub async fn poll_for_ms_token(
    device_code: &str,
    interval: u64,
) -> Result<MicrosoftToken, AnyError> {
    let client = Client::new();
    loop {
        let res = client
            .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("client_id", CLIENT_ID),
                ("device_code", device_code),
            ])
            .send()
            .await?;

        let body: serde_json::Value = res.json().await?;
        if let Some(token) = body.get("access_token") {
            return Ok(MicrosoftToken {
                access_token: token.as_str().unwrap().to_string(),
                refresh_token: body
                    .get("refresh_token")
                    .map(|v| v.as_str().unwrap().to_string()),
            });
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
    }
}

pub async fn get_xbox_token(ms_token: &str) -> Result<(String, String), AnyError> {
    let client = Client::new();
    let body = serde_json::json!({
        "Properties": {
            "AuthMethod": "RPS", "SiteName": "user.auth.xboxlive.com",
            "RpsTicket": format!("d={}", ms_token)
        },
        "RelyingParty": "http://auth.xboxlive.com", "TokenType": "JWT"
    });

    let res = client
        .post("https://user.auth.xboxlive.com/user/authenticate")
        .json(&body)
        .send()
        .await?;
    let status = res.status();
    let text = res.text().await?;

    if !status.is_success() {
        return Err(format!("Xbox Live Auth Failed ({}): {}", status, text).into());
    }

    let val: super::models::XboxLiveResponse = serde_json::from_str(&text)?;
    Ok((val.token, val.display_claims.xui[0].uhs.clone()))
}

///  Exchange for XSTS Tokens
pub async fn get_xsts_token(xbox_token: &str) -> Result<String, AnyError> {
    let client = Client::new();
    let body = serde_json::json!({
        "Properties": { "SandboxId": "RETAIL", "UserTokens": [xbox_token] },
        "RelyingParty": "rp://api.minecraftservices.com/", "TokenType": "JWT"
    });

    let res = client
        .post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .json(&body)
        .send()
        .await?;
    let status = res.status();
    let text = res.text().await?;

    if !status.is_success() {
        return Err(format!("XSTS Auth Failed ({}): {}", status, text).into());
    }

    let val: serde_json::Value = serde_json::from_str(&text)?;
    Ok(val["Token"].as_str().unwrap().to_string())
}

pub async fn get_minecraft_token(xsts_token: &str, uhs: &str) -> Result<String, AnyError> {
    let client = Client::new();
    let body = serde_json::json!({
        "identityToken": format!("XBL3.0 x={};{}", uhs, xsts_token)
    });

    let res = client
        .post("https://api.minecraftservices.com/authentication/login_with_xbox")
        .json(&body)
        .send()
        .await?;
    let status = res.status();
    let text = res.text().await?;
    tracing::info!("Minecraft Login Response: {}", text);
    tracing::info!("Minecraft Login Response Status: {}", status);

    if !status.is_success() {
        return Err(format!("Minecraft Login Failed ({}): {}", status, text).into());
    }

    let val: serde_json::Value = serde_json::from_str(&text)?;
    Ok(val["access_token"]
        .as_str()
        .ok_or("No access_token in response")?
        .to_string())
}
