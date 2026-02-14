use anyhow::{Context, Result};
use dialoguer::{Input, Password};

use crate::crypto::encrypt_data;
use crate::storage::{get_session_key, insert_credential, Database};

/// Add a new credential
pub async fn add_credential(
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

    // Prompt for missing fields
    let label = if let Some(l) = label {
        l
    } else {
        Input::new()
            .with_prompt("Label (e.g., 'GitHub', 'Gmail')")
            .interact_text()
            .context("Failed to read label")?
    };

    let username = if let Some(u) = username {
        u
    } else {
        Input::new()
            .with_prompt("Username/Email")
            .interact_text()
            .context("Failed to read username")?
    };

    let password = if let Some(p) = password {
        p
    } else {
        Password::new()
            .with_prompt("Password")
            .interact()
            .context("Failed to read password")?
    };

    let url = if url.is_some() {
        url
    } else {
        let url_input: String = Input::new()
            .with_prompt("URL (optional, press Enter to skip)")
            .allow_empty(true)
            .interact_text()
            .context("Failed to read URL")?;
        
        if url_input.is_empty() {
            None
        } else {
            Some(url_input)
        }
    };

    // Encrypt username and password
    let (enc_username, nonce_username) = encrypt_data(&key, &username)
        .context("Failed to encrypt username")?;

    let (enc_password, nonce_password) = encrypt_data(&key, &password)
        .context("Failed to encrypt password")?;

    // Insert into database
    let result = insert_credential(
        pool,
        &label,
        url.as_deref(),
        &enc_username,
        &enc_password,
        &nonce_username,
        &nonce_password,
    )
    .await;

    // Handle duplicate label error
    if let Err(e) = result {
        let error_msg = e.to_string();
        if error_msg.contains("UNIQUE constraint") || error_msg.contains("unique constraint") {
            anyhow::bail!(
                "A credential with label '{}' already exists.\n   Use 'chacrab list' to see all labels, or\n   Use 'chacrab delete --label \"{}\"' to remove the existing one.",
                label, label
            );
        } else {
            return Err(e).context("Failed to save credential");
        }
    }

    println!("✅ Credential '{}' added successfully", label);

    Ok(())
}
