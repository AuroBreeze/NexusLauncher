pub mod models;
pub mod storage;
pub mod utils;

use crate::storage::*;
use crate::utils::*;
use nexus_cli::cli::AuthArgs;
use nexus_config::config::Config;
use nexus_config::models::UserConfig;
use nexus_core::AnyError;

pub async fn handle_auth(args: &AuthArgs) -> Result<(), AnyError> {
    if args.login {
        //  Retrieve the device code and display it
        let device_resp = get_device_code().await?;
        tracing::info!(
            "Please open in your browser: {}",
            device_resp.verification_uri
        );
        tracing::info!("Enter the code: {}", device_resp.user_code);

        // Poll Microsoft Token
        let ms_token = poll_for_ms_token(&device_resp.device_code, device_resp.interval).await?;
        tracing::info!("✅ Microsoft authentication successful");

        // 3. Obtain Xbox Token
        let (xbox_token, uhs) = get_xbox_token(&ms_token.access_token).await?;

        // 4. Obtain XSTS Token
        let xsts_token = get_xsts_token(&xbox_token).await?;

        // 5. obtain Minecraft token
        let mc_token = get_minecraft_token(&xsts_token, &uhs).await?;
        tracing::info!("✅ Minecraft token successfully obtained!");
        // 6. check game ownership
        tracing::info!("Verifying game ownership...");
        let is_owner = check_ownership(&mc_token).await?;

        if !is_owner {
            tracing::error!(
                "❌ Verification failed: This account has not purchased Minecraft for Java Edition!"
            );
            return Err("Account does not own the game".into());
        }
        tracing::info!("✅ Permission verified: You own the game");

        // 7. Get player profile (UUID and nickname)
        let profile = get_minecraft_profile(&mc_token).await?;
        tracing::info!("🚀 Login successful! Welcome back, {}!", profile.name);
        tracing::info!("Player UUID: {}", profile.id);

        if let Some(refresh_token) = ms_token.refresh_token {
            // Use the player's UUID or nickname as the key
            save_refresh_token(&profile.id, &refresh_token)?;
            tracing::info!(
                "✅ The security credentials have been encrypted and stored in the system credential manager."
            );
        }

        let mut config = UserConfig::load().await;
        config.user_profile.online.username = profile.name.clone();
        config.user_profile.online.uuid = profile.id.clone();

        config
            .username
            .insert(profile.name.clone(), profile.id.clone());

        config.save().await?;
        tracing::info!("✅ Username has been saved in the launcher config.");
    }

    if let Some(name) = &args.logout {
        let mut config = UserConfig::load().await;
        let uuid = match config.username.get(name) {
            Some(u) => u,
            None => {
                return Err(format!("User {} not found", name).into());
            }
        };

        delete_token(uuid).unwrap();
        config.username.remove(name);
        config.user_profile.online.username = "".to_string();
        config.user_profile.online.uuid = "".to_string();
        config.save().await.unwrap();
        tracing::info!(
            "✅ Security credentials have been deleted from the system credential manager."
        );
    }

    Ok(())
}
