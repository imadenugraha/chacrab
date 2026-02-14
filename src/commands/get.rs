use anyhow::{Context, Result};
use cli_clipboard::{ClipboardContext, ClipboardProvider};
use dialoguer::{Input, Select};

use crate::crypto::decrypt_data;
use crate::storage::{get_credential_by_label, get_session_key, Database};

/// Get and display/copy a credential
pub async fn get_credential(db: &Database, label: Option<String>) -> Result<()> {
    let pool = db.pool();

    // Get session key
    let key = get_session_key()
        .context("Not logged in. Please run 'chacrab login' first.")?;

    // Prompt for label if not provided
    let label = if let Some(l) = label {
        l
    } else {
        Input::new()
            .with_prompt("Label")
            .interact_text()
            .context("Failed to read label")?
    };

    // Fetch credential
    let credential = get_credential_by_label(pool, &label)
        .await?
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Credential '{}' not found.\n   Use 'chacrab list' to see all stored credentials.",
                label
            )
        })?;

    // Decrypt username and password
    let username = decrypt_data(&key, &credential.enc_username, &credential.nonce_username)
        .context("Failed to decrypt username.\n   Your session may be corrupted. Try: chacrab logout && chacrab login")?;

    let password = decrypt_data(&key, &credential.enc_password, &credential.nonce_password)
        .context("Failed to decrypt password.\n   Your session may be corrupted. Try: chacrab logout && chacrab login")?;

    // Display credential info
    println!("\n🔐 Credential: {}", credential.label);
    if let Some(url) = &credential.url {
        println!("   URL: {}", url);
    }
    println!("   Username: {}", username);

    // Ask what to do with password
    let options = vec!["Copy password to clipboard", "Show password", "Cancel"];
    let selection = Select::new()
        .with_prompt("Password")
        .items(&options)
        .default(0)
        .interact()
        .context("Failed to read selection")?;

    match selection {
        0 => {
            // Copy to clipboard
            match ClipboardContext::new() {
                Ok(mut ctx) => {
                    if let Err(e) = ctx.set_contents(password.clone()) {
                        eprintln!("⚠️  Failed to copy to clipboard: {}", e);
                        println!("   Password: {}", password);
                    } else {
                        println!("✅ Password copied to clipboard");
                    }
                }
                Err(e) => {
                    eprintln!("⚠️  Clipboard not available: {}", e);
                    println!("   Password: {}", password);
                }
            }
        }
        1 => {
            // Show password
            println!("   Password: {}", password);
        }
        _ => {
            println!("Cancelled");
        }
    }

    println!();

    Ok(())
}
