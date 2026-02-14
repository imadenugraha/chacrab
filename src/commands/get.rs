use anyhow::{Context, Result};
use cli_clipboard::{ClipboardContext, ClipboardProvider};
use dialoguer::{Input, Select};
use std::io::Write;
use std::process::{Command, Stdio};

use crate::crypto::decrypt_data;
use crate::storage::{get_credential_by_label, get_session_key, Database};

fn copy_to_clipboard(text: &str) -> Result<()> {
    if let Ok(mut ctx) = ClipboardContext::new() {
        if ctx.set_contents(text.to_string()).is_ok() {
            return Ok(());
        }
    }

    let fallback_commands: Vec<(&str, Vec<&str>)> = vec![
        ("wl-copy", vec![]),
        ("xclip", vec!["-selection", "clipboard"]),
        ("xsel", vec!["--clipboard", "--input"]),
    ];

    for (program, args) in fallback_commands {
        let child = Command::new(program)
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        let mut child = match child {
            Ok(child) => child,
            Err(_) => continue,
        };

        if let Some(stdin) = child.stdin.as_mut() {
            if stdin.write_all(text.as_bytes()).is_err() {
                let _ = child.wait();
                continue;
            }
        } else {
            let _ = child.wait();
            continue;
        }

        if child.wait().map(|status| status.success()).unwrap_or(false) {
            return Ok(());
        }
    }

    Err(anyhow::anyhow!(
        "Clipboard unavailable. Install one of: wl-copy (Wayland), xclip, or xsel"
    ))
}

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
            if let Err(e) = copy_to_clipboard(&password) {
                eprintln!("⚠️  Failed to copy to clipboard: {}", e);
                println!("   Password: {}", password);
            } else {
                println!("✅ Password copied to clipboard");
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
