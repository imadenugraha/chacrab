use anyhow::{Context, Result};
use dialoguer::{Input, MultiSelect, Password};

use crate::crypto::{decrypt_data, encrypt_data};
use crate::storage::{get_credential_by_label, get_session_key, update_credential, Database};

/// Update an existing credential
pub async fn update_credential_cmd(
    db: &Database,
    label: Option<String>,
    username: Option<String>,
    password: Option<String>,
    url: Option<String>,
) -> Result<()> {
    let pool = db.pool();

    // Get session key
    let key = get_session_key()
        .context("Not logged in. Please run 'chacrab login' first.")?;

    // Prompt for label if not provided
    let label = if let Some(l) = label {
        l
    } else {
        Input::new()
            .with_prompt("Label of credential to update")
            .interact_text()
            .context("Failed to read label")?
    };

    // Fetch existing credential
    let credential = get_credential_by_label(pool, &label)
        .await?
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Credential '{}' not found.\n   Use 'chacrab list' to see all stored credentials.",
                label
            )
        })?;

    // Decrypt current values
    let current_username = decrypt_data(&key, &credential.enc_username, &credential.nonce_username)
        .context("Failed to decrypt username")?;
    let current_password = decrypt_data(&key, &credential.enc_password, &credential.nonce_password)
        .context("Failed to decrypt password")?;
    let current_url = credential.url.clone();

    println!("\n🔐 Updating credential: {}", label);
    println!("   Current username: {}", current_username);
    if let Some(url) = &current_url {
        println!("   Current URL: {}", url);
    }
    println!("   Current password: [hidden]\n");

    // Determine which fields to update
    let (new_username, new_password, new_url) = if username.is_some() || password.is_some() || url.is_some() {
        // If flags provided, use them (None means keep current)
        (
            username.unwrap_or(current_username.clone()),
            password.unwrap_or(current_password.clone()),
            url.or(current_url.clone()),
        )
    } else {
        // Interactive mode: ask which fields to update
        let fields = vec!["Username", "Password", "URL"];
        let selections = MultiSelect::new()
            .with_prompt("Select fields to update (Space to select, Enter to confirm)")
            .items(&fields)
            .interact()
            .context("Failed to read field selection")?;

        if selections.is_empty() {
            println!("No fields selected. Aborting update.");
            return Ok(());
        }

        let mut new_username = current_username.clone();
        let mut new_password = current_password.clone();
        let mut new_url = current_url.clone();

        // Prompt for each selected field
        for &idx in &selections {
            match idx {
                0 => {
                    // Update username
                    new_username = Input::new()
                        .with_prompt("New username")
                        .with_initial_text(&current_username)
                        .interact_text()
                        .context("Failed to read username")?;
                }
                1 => {
                    // Update password
                    new_password = Password::new()
                        .with_prompt("New password")
                        .interact()
                        .context("Failed to read password")?;
                }
                2 => {
                    // Update URL
                    let url_input: String = Input::new()
                        .with_prompt("New URL (press Enter to keep current, or type 'none' to remove)")
                        .with_initial_text(current_url.as_deref().unwrap_or(""))
                        .allow_empty(true)
                        .interact_text()
                        .context("Failed to read URL")?;
                    
                    if url_input.to_lowercase() == "none" {
                        new_url = None;
                    } else if url_input.is_empty() {
                        new_url = current_url.clone();
                    } else {
                        new_url = Some(url_input);
                    }
                }
                _ => {}
            }
        }

        (new_username, new_password, new_url)
    };

    // Re-encrypt username and password with new nonces
    let (enc_username, nonce_username) = encrypt_data(&key, &new_username)
        .context("Failed to encrypt username")?;

    let (enc_password, nonce_password) = encrypt_data(&key, &new_password)
        .context("Failed to encrypt password")?;

    // Update in database
    let updated = update_credential(
        pool,
        &label,
        new_url.as_deref(),
        &enc_username,
        &enc_password,
        &nonce_username,
        &nonce_password,
    )
    .await
    .context("Failed to update credential")?;

    if updated {
        println!("✅ Credential '{}' updated successfully", label);
    } else {
        println!("⚠️  No changes were made to credential '{}'", label);
    }

    Ok(())
}
