use anyhow::{Context, Result};
use dialoguer::Password;

use crate::crypto::{decrypt_data, derive_key};
use crate::storage::{get_user_config, has_session_key, save_session_key, Database};

/// Login to unlock the vault
pub async fn login(db: &Database) -> Result<()> {
    let pool = db.pool();

    // Check if already logged in
    if has_session_key() {
        println!("✅ Already logged in");
        return Ok(());
    }

    // Get user config (salt)
    let config = get_user_config(pool)
        .await?
        .context("Vault not initialized. Please run 'chacrab init' first.")?;

    // Prompt for master password
    let password = Password::new()
        .with_prompt("Master password")
        .interact()
        .context("Failed to read master password")?;

    // Derive key
    let key = derive_key(&password, &config.salt)
        .context("Failed to derive encryption key")?;

    // Validate password using verification token (if available)
    if let (Some(token), Some(nonce)) = (&config.verification_token, &config.verification_nonce) {
        match decrypt_data(&key, token, nonce) {
            Ok(decrypted) => {
                if decrypted != "CHACRAB_VALID_SESSION" {
                    anyhow::bail!("Internal error: Invalid verification token");
                }
                // Verification successful - password is correct
            }
            Err(_) => {
                anyhow::bail!(
                    "❌ Incorrect master password. Please try again.\n   Hint: Make sure Caps Lock is off."
                );
            }
        }
    } else {
        // No verification token (legacy vault)
        println!("⚠️  Warning: This vault was created without password verification.");
        println!("   For better security, consider re-initializing: backup your data,");
        println!("   delete the vault, and run 'chacrab init' again.\n");
    }

    // Save to keyring
    save_session_key(&key)
        .context("Failed to save session key")?;

    println!("✅ Logged in successfully");

    Ok(())
}
