use crate::version::AnyError;
use crate::version::utils::get_minecraft_dir;
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use sha2::{Digest, Sha256};
use std::fs;

/// Ensures lengths are always correct via hashing
fn derive_key_and_nonce(uuid: &str) -> ([u8; 32], [u8; 12]) {
    // Get machine ID as the base salt
    let mid = machine_uid::get().unwrap_or_else(|_| "nexus_salt".to_string());

    // Compute 32-byte Key
    let mut key_hasher = Sha256::new();
    key_hasher.update(mid.as_bytes());
    let key: [u8; 32] = key_hasher.finalize().into();

    // Compute 12-byte Nonce (take the first 12 bytes of the hash)
    let mut nonce_hasher = Sha256::new();
    nonce_hasher.update(uuid.as_bytes());
    let full_hash = nonce_hasher.finalize();
    let mut nonce_bytes = [0u8; 12];
    nonce_bytes.copy_from_slice(&full_hash[..12]); // Forcefully slice the first 12 bytes

    (key, nonce_bytes)
}

pub fn save_refresh_token(uuid: &str, token: &str) -> Result<(), AnyError> {
    let (key, nonce_bytes) = derive_key_and_nonce(uuid);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, token.as_bytes())
        .map_err(|e| format!("Encryption error: {}", e))?;

    let vault_dir = get_minecraft_dir().join("auth_vault");
    if !vault_dir.exists() {
        fs::create_dir_all(&vault_dir)?;
    }

    fs::write(vault_dir.join(uuid), ciphertext)?;
    Ok(())
}

pub fn get_refresh_token(uuid: &str) -> Result<String, AnyError> {
    let file_path = get_minecraft_dir().join("auth_vault").join(uuid);
    let ciphertext = fs::read(file_path).map_err(|_| "Credentials not found")?;

    let (key, nonce_bytes) = derive_key_and_nonce(uuid);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_slice())
        .map_err(|e| format!("Decryption error: {}. Hardware changed?", e))?;

    Ok(String::from_utf8(plaintext)?)
}

pub fn delete_token(uuid: &str) -> Result<(), AnyError> {
    let vault_dir = get_minecraft_dir().join("auth_vault");
    let file_path = vault_dir.join(uuid);

    if file_path.exists() {
        fs::remove_file(file_path)?;
        tracing::info!(
            "The hardware encryption credentials have been physically destroyed: {}",
            uuid
        );
    }
    Ok(())
}
