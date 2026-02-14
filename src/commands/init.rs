use anyhow::{Context, Result};
use dialoguer::{Confirm, Password};

use crate::commands::VERIFICATION_SENTINEL;
use crate::crypto::{derive_key, encrypt_data};
use crate::storage::{create_user_config, get_user_config, save_session_key, update_verification_token, Database};
use crate::ui::{is_test_mode, test_env, validate_password, StrengthLevel};

/// Initialize the ChaCrab vault with a master password
pub async fn init_vault(db: &Database) -> Result<()> {
    let pool = db.pool();

    // Check if already initialized
    if get_user_config(pool).await?.is_some() {
        anyhow::bail!(
            "Vault already initialized. Use 'chacrab login' to unlock."
        );
    }

    println!("🔐 Initializing ChaCrab vault\n");
    println!("Your master password will be used to encrypt all your credentials.");
    println!("⚠️  Important: There is NO way to recover your data if you forget this password!\n");

    // Get master password
    let password = if is_test_mode() {
        test_env("CHACRAB_MASTER_PASSWORD").ok_or_else(|| {
            anyhow::anyhow!(
                "CHACRAB_TEST_MODE is enabled but CHACRAB_MASTER_PASSWORD is not set"
            )
        })?
    } else {
        Password::new()
            .with_prompt("Enter master password")
            .with_confirmation("Confirm master password", "Passwords do not match")
            .interact()
            .context("Failed to read master password")?
    };

    // Validate password strength
    let strength = validate_password(&password);
    
    // Enforce minimum length
    if password.len() < 8 {
        anyhow::bail!("Master password must be at least 8 characters long");
    }

    // Display strength feedback
    match strength.level {
        StrengthLevel::Weak => {
            println!("\n{} Password Strength: {}", strength.level.emoji(), strength.level.as_str());
            println!("\n⚠️  WARNING: This password is weak and easily guessed!");
            
            if !strength.warnings.is_empty() {
                println!("\nIssues detected:");
                for warning in &strength.warnings {
                    println!("   • {}", warning);
                }
            }
            
            if !strength.suggestions.is_empty() {
                println!("\nSuggestions:");
                for suggestion in &strength.suggestions {
                    println!("   • {}", suggestion);
                }
            }
            
            println!("\nRecommendations:");
            println!("   • Use at least 12 characters");
            println!("   • Mix uppercase, lowercase, numbers, and symbols");
            println!("   • Consider a passphrase: 4+ random words");
            println!("   • Avoid common passwords and personal information");
            
            let proceed = if is_test_mode() {
                true
            } else {
                Confirm::new()
                    .with_prompt("\nUse this weak password anyway? (NOT RECOMMENDED)")
                    .default(false)
                    .interact()
                    .context("Failed to read confirmation")?
            };
            
            if !proceed {
                anyhow::bail!("Password rejected. Please run 'chacrab init' again with a stronger password.");
            }
            println!("\n⚠️  Proceeding with weak password (not recommended)...\n");
        },
        StrengthLevel::Fair => {
            println!("\n{} Password Strength: {}", strength.level.emoji(), strength.level.as_str());
            
            if !strength.warnings.is_empty() || !strength.suggestions.is_empty() {
                println!("\nYour password could be stronger:");
                for suggestion in &strength.suggestions {
                    println!("   • {}", suggestion);
                }
                
                if !strength.warnings.is_empty() {
                    for warning in &strength.warnings {
                        println!("   • {}", warning);
                    }
                }
            }
            println!();
        },
        StrengthLevel::Strong => {
            println!("\n{} Password Strength: {}", strength.level.emoji(), strength.level.as_str());
            println!("   Your password is strong!\n");
        },
        StrengthLevel::Excellent => {
            println!("\n{} Password Strength: {}", strength.level.emoji(), strength.level.as_str());
            println!("   Your password is excellent!\n");
        },
    }

    // Confirm understanding
    let confirmed = if is_test_mode() {
        true
    } else {
        Confirm::new()
            .with_prompt("I understand that forgetting my master password means losing all my data")
            .default(false)
            .interact()
            .context("Failed to read confirmation")?
    };

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
    let (verification_token, verification_nonce) = encrypt_data(&key, VERIFICATION_SENTINEL)
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
