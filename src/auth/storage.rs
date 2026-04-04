// src/auth/storage.rs
use crate::version::AnyError;
use keyring::Entry;

const SERVICE_NAME: &str = "nexus_launcher_auth";

pub fn save_token(username: &str, token: &str) -> Result<(), AnyError> {
    let entry = Entry::new(SERVICE_NAME, username)?;
    entry.set_password(token)?;
    Ok(())
}

pub fn get_token(username: &str) -> Result<String, AnyError> {
    let entry = Entry::new(SERVICE_NAME, username)?;
    let password = entry.get_password()?;
    Ok(password)
}

pub fn delete_token(username: &str) -> Result<(), AnyError> {
    let entry = Entry::new(SERVICE_NAME, username)?;
    entry.delete_credential()?;
    Ok(())
}
