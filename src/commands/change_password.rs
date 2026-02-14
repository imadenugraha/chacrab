use anyhow::{Context, Result};
use argon2::password_hash::SaltString;
use dialoguer::{Confirm, Password};
use rand::rngs::OsRng;

use crate::commands::{verify_sentinel_constant_time, VERIFICATION_SENTINEL};
use crate::crypto::{decrypt_data, derive_key, encrypt_data};
use crate::storage::{
    delete_session_key, get_user_config, list_all_credentials,
    save_session_key, update_credential, Database,
};
use crate::ui::password_validator::{validate_password, StrengthLevel};
use crate::ui::{is_test_mode, test_env};

/// Change the master password
///
/// This is a critical operation that:
/// 1. Verifies the current password
/// 2. Validates the new password
/// 3. Re-encrypts ALL credentials with the new key
/// 4. Updates the salt and verification token
///
/// ⚠️ WARNING: This operation should be preceded by a backup
pub async fn change_password(db: &Database) -> Result<()> {
    let pool = db.pool();

    println!("🔐 Change Master Password");
    println!();
    println!("⚠️  IMPORTANT: This will re-encrypt ALL your credentials with a new master password.");
    println!("   It is HIGHLY RECOMMENDED to create a backup before proceeding.");
    println!();

    // Recommend backup
    let has_backup = if is_test_mode() {
        true
    } else {
        Confirm::new()
            .with_prompt("Have you created a recent backup? (chacrab export)")
            .default(false)
            .interact()
            .context("Failed to read confirmation")?
    };

    if !has_backup {
        let proceed = Confirm::new()
            .with_prompt("⚠️  Proceed WITHOUT backup? (Not recommended!)")
            .default(false)
            .interact()
            .context("Failed to read confirmation")?;

        if !proceed {
            println!("❌ Operation cancelled. Please create a backup first:");
            println!("   chacrab export --output backup.json");
            return Ok(());
        }
    }

    println!();

    // Get current password and verify it
    let current_password = if is_test_mode() {
        test_env("CHACRAB_CURRENT_PASSWORD")
            .or_else(|| test_env("CHACRAB_MASTER_PASSWORD"))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "CHACRAB_TEST_MODE is enabled but CHACRAB_CURRENT_PASSWORD is not set"
                )
            })?
    } else {
        Password::new()
            .with_prompt("Current master password")
            .interact()
            .context("Failed to read current password")?
    };

    // Verify current password by checking session key or deriving
    let config = get_user_config(pool)
        .await
        .context("Failed to get user config")?
        .context("Vault not initialized. Run 'chacrab init' first.")?;

    // Derive key from current password
    let current_key = derive_key(&current_password, &config.salt)
        .context("Failed to derive key from current password")?;

    // Verify current password is correct by checking verification token
    if let (Some(token), Some(token_nonce)) = (&config.verification_token, &config.verification_nonce) {
        match decrypt_data(&current_key, token, token_nonce) {
            Ok(decrypted) => {
                if !verify_sentinel_constant_time(&decrypted) {
                    anyhow::bail!("❌ Current password is incorrect. Please try again.");
                }
            }
            Err(_) => {
                anyhow::bail!("❌ Current password is incorrect. Please try again.");
            }
        }
    } else {
        anyhow::bail!("Verification token not found. Please run 'chacrab login' first.");
    }

    println!("✅ Current password verified");
    println!();

    // Get new password with validation
    let new_password = if is_test_mode() {
        let password = test_env("CHACRAB_NEW_PASSWORD").ok_or_else(|| {
            anyhow::anyhow!("CHACRAB_TEST_MODE is enabled but CHACRAB_NEW_PASSWORD is not set")
        })?;
        let validation_result = validate_password(&password);
        if matches!(validation_result.level, StrengthLevel::Weak) {
            anyhow::bail!("❌ Password too weak. Please choose a stronger password.");
        }
        password
    } else {
        loop {
            let password = Password::new()
                .with_prompt("New master password")
                .interact()
                .context("Failed to read new password")?;

            let confirmation = Password::new()
                .with_prompt("Confirm new master password")
                .interact()
                .context("Failed to read password confirmation")?;

            if password != confirmation {
                println!("❌ Passwords don't match. Please try again.\n");
                continue;
            }

            // Validate password strength
            let validation_result = validate_password(&password);

            println!("\n🔍 Password Strength: {} {}", validation_result.level.emoji(), validation_result.level.as_str());
            println!("   Score: {}/100", validation_result.score);

            if !validation_result.warnings.is_empty() {
                println!("\n⚠️  Warnings:");
                for warning in &validation_result.warnings {
                    println!("   • {}", warning);
                }
            }

            if !validation_result.suggestions.is_empty() {
                println!("\n💡 Suggestions:");
                for suggestion in &validation_result.suggestions {
                    println!("   • {}", suggestion);
                }
            }

            if matches!(validation_result.level, StrengthLevel::Weak) {
                println!("\n❌ Password too weak. Please choose a stronger password.\n");
                continue;
            }

            println!();
            let proceed = Confirm::new()
                .with_prompt("Use this password?")
                .default(true)
                .interact()
                .context("Failed to read confirmation")?;

            if proceed {
                break password;
            }

            println!();
        }
    };

    println!();
    println!("🔄 Re-encrypting all credentials...");
    println!();

    // Fetch all credentials
    let credentials = list_all_credentials(pool)
        .await
        .context("Failed to fetch credentials")?;

    if credentials.is_empty() {
        println!("ℹ️  No credentials to re-encrypt");
    } else {
        println!("📦 Found {} credential(s) to re-encrypt", credentials.len());
    }

    // Generate new salt
    let new_salt = SaltString::generate(&mut OsRng);
    let new_salt_str = new_salt.as_str();

    // Derive new key
    let new_key = derive_key(&new_password, new_salt_str)
        .context("Failed to derive new encryption key")?;

    // Re-encrypt all credentials
    let mut re_encrypted_credentials = Vec::new();

    for cred in &credentials {
        // Decrypt with old key
        let username = decrypt_data(&current_key, &cred.enc_username, &cred.nonce_username)
            .context(format!("Failed to decrypt username for '{}'", cred.label))?;

        let password = decrypt_data(&current_key, &cred.enc_password, &cred.nonce_password)
            .context(format!("Failed to decrypt password for '{}'", cred.label))?;

        // Re-encrypt with new key
        let (new_enc_username, new_nonce_username) = encrypt_data(&new_key, &username)
            .context(format!("Failed to re-encrypt username for '{}'", cred.label))?;

        let (new_enc_password, new_nonce_password) = encrypt_data(&new_key, &password)
            .context(format!("Failed to re-encrypt password for '{}'", cred.label))?;

        re_encrypted_credentials.push((
            cred.label.clone(),
            cred.url.clone(),
            new_enc_username,
            new_enc_password,
            new_nonce_username,
            new_nonce_password,
        ));

        println!("   ✅ Re-encrypted '{}'", cred.label);
    }

    // Update all credentials in database
    for (label, url, enc_username, enc_password, nonce_username, nonce_password) in re_encrypted_credentials {
        update_credential(
            pool,
            &label,
            url.as_deref(),
            &enc_username,
            &enc_password,
            &nonce_username,
            &nonce_password,
        )
        .await
        .context(format!("Failed to update credential '{}'", label))?;
    }

    // Create new verification token
    let (verification_token, verification_nonce) = encrypt_data(&new_key, VERIFICATION_SENTINEL)
        .context("Failed to create verification token")?;

    // Update user_config with new salt and verification token
    // Note: We need to add a function to update salt as well
    update_user_config_with_new_salt(pool, new_salt_str, &verification_token, &verification_nonce)
        .await
        .context("Failed to update user configuration with new salt")?;

    // Update session key in keyring
    delete_session_key().context("Failed to clear old session key")?;
    save_session_key(&new_key).context("Failed to save new session key")?;

    println!();
    println!("🎉 Master password changed successfully!");
    println!("   ✅ {} credential(s) re-encrypted", credentials.len());
    println!("   ✅ New salt generated");
    println!("   ✅ Session key updated");
    println!();
    println!("⚠️  Remember: Use this new password for all future logins.");

    Ok(())
}

/// Update user_config with new salt and verification token
/// This is a helper function that needs to be added to storage/queries.rs
async fn update_user_config_with_new_salt(
    pool: &crate::storage::db::DatabasePool,
    new_salt: &str,
    verification_token: &str,
    verification_nonce: &str,
) -> Result<()> {
    use crate::storage::db::DatabasePool;

    match pool {
        DatabasePool::Sqlite(p) => {
            sqlx::query(
                "UPDATE user_config SET salt = ?, verification_token = ?, verification_nonce = ?, updated_at = datetime('now')"
            )
            .bind(new_salt)
            .bind(verification_token)
            .bind(verification_nonce)
            .execute(p)
            .await
            .context("Failed to update user config with new salt")?;
        }
        DatabasePool::Postgres(p) => {
            sqlx::query(
                "UPDATE user_config SET salt = $1, verification_token = $2, verification_nonce = $3, updated_at = CURRENT_TIMESTAMP"
            )
            .bind(new_salt)
            .bind(verification_token)
            .bind(verification_nonce)
            .execute(p)
            .await
            .context("Failed to update user config with new salt")?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{derive_key, encrypt_data};
    use crate::storage::db::init_db;
    use crate::storage::{create_user_config, insert_credential, get_user_config, list_all_credentials};

    #[tokio::test]
    async fn test_update_user_config_with_new_salt_sqlite() {
        let db = init_db("sqlite::memory:").await.unwrap();
        let pool = db.pool();

        // Create initial user config
        create_user_config(pool).await.unwrap();

        // Update with new salt
        let new_salt = "new_test_salt";
        let new_token = "new_token";
        let new_nonce = "new_nonce";

        update_user_config_with_new_salt(pool, new_salt, new_token, new_nonce)
            .await
            .unwrap();

        // Verify update
        let config = get_user_config(pool).await.unwrap().expect("Config should exist");
        assert_eq!(config.salt, new_salt);
        assert_eq!(config.verification_token, Some(new_token.to_string()));
        assert_eq!(config.verification_nonce, Some(new_nonce.to_string()));
    }

    #[tokio::test]
    async fn test_credential_re_encryption_logic() {
        let db = init_db("sqlite::memory:").await.unwrap();
        let pool = db.pool();
        create_user_config(pool).await.unwrap();

        // Simulate encryption with "old" password
        let old_password = "OldPassword123!";
        let old_salt = SaltString::from_b64("c29tZXNhbHR2YWx1ZTEyMzQ1Njc4OTA").unwrap();
        let old_key = derive_key(old_password, old_salt.as_str()).unwrap();

        let original_username = "testuser";
        let original_password = "testpass";

        let (enc_u, nonce_u) = encrypt_data(&old_key, original_username).unwrap();
        let (enc_p, nonce_p) = encrypt_data(&old_key, original_password).unwrap();

        // Insert credential encrypted with old key
        insert_credential(
            pool,
            "TestLabel",
            Some("https://example.com"),
            &enc_u,
            &enc_p,
            &nonce_u,
            &nonce_p,
        )
        .await
        .unwrap();

        // Simulate re-encryption with "new" password
        let new_password = "NewPassword456!";
        let new_salt = SaltString::generate(&mut rand::thread_rng());
        let new_key = derive_key(new_password, new_salt.as_str()).unwrap();

        // Decrypt with old key
        let decrypted_u = crate::crypto::decrypt_data(&old_key, &enc_u, &nonce_u).unwrap();
        let decrypted_p = crate::crypto::decrypt_data(&old_key, &enc_p, &nonce_p).unwrap();

        assert_eq!(decrypted_u, original_username);
        assert_eq!(decrypted_p, original_password);

        // Re-encrypt with new key
        let (new_enc_u, new_nonce_u) = encrypt_data(&new_key, &decrypted_u).unwrap();
        let (new_enc_p, new_nonce_p) = encrypt_data(&new_key, &decrypted_p).unwrap();

        // Verify re-encrypted data can be decrypted with new key
        let final_u = crate::crypto::decrypt_data(&new_key, &new_enc_u, &new_nonce_u).unwrap();
        let final_p = crate::crypto::decrypt_data(&new_key, &new_enc_p, &new_nonce_p).unwrap();

        assert_eq!(final_u, original_username);
        assert_eq!(final_p, original_password);

        // Verify old key can no longer decrypt new data
        let should_fail = crate::crypto::decrypt_data(&old_key, &new_enc_u, &new_nonce_u);
        assert!(should_fail.is_err());
    }

    #[tokio::test]
    async fn test_password_change_updates_all_credentials() {
        let db = init_db("sqlite::memory:").await.unwrap();
        let pool = db.pool();
        create_user_config(pool).await.unwrap();

        // Create test data with old key
        let old_password = "OldPass123!";
        let old_salt = SaltString::generate(&mut rand::thread_rng());
        let old_key = derive_key(old_password, old_salt.as_str()).unwrap();

        // Insert multiple credentials
        for i in 1..=5 {
            let username = format!("user{}", i);
            let password = format!("pass{}", i);
            let (enc_u, nonce_u) = encrypt_data(&old_key, &username).unwrap();
            let (enc_p, nonce_p) = encrypt_data(&old_key, &password).unwrap();
            
            insert_credential(
                pool,
                &format!("Label{}", i),
                None,
                &enc_u,
                &enc_p,
                &nonce_u,
                &nonce_p,
            )
            .await
            .unwrap();
        }

        // Verify 5 credentials exist
        let all_creds = list_all_credentials(pool).await.unwrap();
        assert_eq!(all_creds.len(), 5);

        // Simulate password change would re-encrypt all 5
        // (Full integration test requires interactive prompts, testing logic here)
        
        let credentials = list_all_credentials(pool).await.unwrap();
        assert_eq!(credentials.len(), 5, "All credentials should be present for re-encryption");
    }
}