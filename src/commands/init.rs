use anyhow::{Context, Result};
use dialoguer::{Confirm, Password};

use crate::crypto::{derive_key, encrypt_data};
use crate::storage::{create_user_config, get_user_config, save_session_key, update_verification_token, Database};

/// Initialize the ChaCrab vault with a master password
pub async fn init_vault(db: &Database) -> Result<()> {
    let pool = db.pool();

    // Check if already initialized
    if let Some(_) = get_user_config(pool).await? {
        anyhow::bail!(
            "Vault already initialized. Use 'chacrab login' to unlock."
        );
    }

    println!("🔐 Initializing ChaCrab vault\n");
    println!("Your master password will be used to encrypt all your credentials.");
    println!("⚠️  Important: There is NO way to recover your data if you forget this password!\n");

    // Get master password
    let password = Password::new()
        .with_prompt("Enter master password")
        .with_confirmation("Confirm master password", "Passwords do not match")
        .interact()
        .context("Failed to read master password")?;

    if password.len() < 8 {
        anyhow::bail!("Master password must be at least 8 characters long");
    }

    // Confirm understanding
    let confirmed = Confirm::new()
        .with_prompt("I understand that forgetting my master password means losing all my data")
        .default(false)
        .interact()
        .context("Failed to read confirmation")?;

    if !confirmed {
        anyhow::bail!("Initialization cancelled");
    }

    // Create user config with random salt
    let config = create_user_config(pool)
        .await
        .context("Failed to create vault configuration")?;

    // Derive key and save to keyring for immediate use
    let key = derive_key(&password, &config.salt)
        .context("Failed to derive encryption key")?;

    // Create verification token for future login validation
    let verification_string = "CHACRAB_VALID_SESSION";
    let (verification_token, verification_nonce) = encrypt_data(&key, verification_string)
        .context("Failed to create verification token")?;
    
    update_verification_token(pool, &verification_token, &verification_nonce)
        .await
        .context("Failed to save verification token")?;

    save_session_key(&key)
        .context("Failed to save session key")?;

    println!("\n✅ Vault initialized successfully!");
    println!("   You are now logged in.");
    println!("\n💡 Tip: Use 'chacrab add' to add your first credential");

    Ok(())
}
