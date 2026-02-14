use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use keyring::Entry;

const SERVICE_NAME: &str = "chacrab";
const USERNAME: &str = "default";

fn keyring_service_name() -> String {
    std::env::var("CHACRAB_KEYRING_SERVICE").unwrap_or_else(|_| SERVICE_NAME.to_string())
}

fn keyring_username() -> String {
    std::env::var("CHACRAB_KEYRING_USERNAME").unwrap_or_else(|_| USERNAME.to_string())
}

/// Save the derived session key to OS keyring
pub fn save_session_key(key: &[u8; 32]) -> Result<()> {
    let service = keyring_service_name();
    let username = keyring_username();
    let entry = Entry::new(&service, &username)
        .context("Failed to create keyring entry")?;

    let key_base64 = BASE64.encode(key);

    entry
        .set_password(&key_base64)
        .context("Failed to save session key to keyring")?;

    Ok(())
}

/// Retrieve the session key from OS keyring
pub fn get_session_key() -> Result<[u8; 32]> {
    let service = keyring_service_name();
    let username = keyring_username();
    let entry = Entry::new(&service, &username)
        .context("Failed to create keyring entry")?;

    let key_base64 = entry
        .get_password()
        .context("No active session found. Please run 'chacrab login' first.")?;

    let key_bytes = BASE64
        .decode(&key_base64)
        .context("Failed to decode session key")?;

    if key_bytes.len() != 32 {
        anyhow::bail!("Invalid session key length in keyring");
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&key_bytes);

    Ok(key)
}

/// Delete the session key from OS keyring
pub fn delete_session_key() -> Result<()> {
    let service = keyring_service_name();
    let username = keyring_username();
    let entry = Entry::new(&service, &username)
        .context("Failed to create keyring entry")?;

    entry
        .delete_password()
        .context("Failed to delete session key from keyring")?;

    Ok(())
}

/// Check if a session key exists
pub fn has_session_key() -> bool {
    let service = keyring_service_name();
    let username = keyring_username();
    if let Ok(entry) = Entry::new(&service, &username) {
        entry.get_password().is_ok()
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyring_roundtrip() {
        let test_key = [42u8; 32];

        // Save
        let save_result = save_session_key(&test_key);
        if save_result.is_err() {
            // Skip test if keyring is not available (CI environments)
            eprintln!("Skipping keyring test: keyring not available");
            return;
        }

        // Check exists
        assert!(has_session_key());

        // Retrieve
        let retrieved = get_session_key().unwrap();
        assert_eq!(test_key, retrieved);

        // Delete
        delete_session_key().unwrap();

        // Check doesn't exist
        assert!(!has_session_key());
    }
}
