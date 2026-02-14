use anyhow::{Context, Result};
use dialoguer::{Select, theme::ColorfulTheme};
use std::fs;
use std::path::PathBuf;

use crate::crypto::encrypt_data;
use crate::models::{Credential, DecryptedCredential};
use crate::storage::{get_credential_by_label, get_session_key, insert_credential, Database};
use crate::ui::{is_test_mode, test_env};

#[derive(Debug)]
struct ImportedCredential {
    label: String,
    url: Option<String>,
    enc_username: String,
    enc_password: String,
    nonce_username: String,
    nonce_password: String,
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum ImportPayload {
    Encrypted(Vec<Credential>),
    PlaintextWrapped {
        format: Option<String>,
        warning: Option<String>,
        exported_at: Option<serde_json::Value>,
        credentials: Vec<DecryptedCredential>,
    },
    Plaintext(Vec<DecryptedCredential>),
}

/// Import credentials from a JSON file
/// 
/// Handles duplicate labels with user choice:
/// - Skip: Don't import duplicates
/// - Overwrite: Replace existing with imported
/// - Rename: Add suffix to imported label
pub async fn import_credentials(db: &Database, input_path: PathBuf) -> Result<()> {
    let pool = db.pool();

    // Read and parse JSON file
    let json = fs::read_to_string(&input_path)
        .context(format!("Failed to read import file '{}'", input_path.display()))?;

    let payload: ImportPayload = serde_json::from_str(&json)
        .context("Failed to parse JSON. Ensure the file is a valid ChaCrab export.")?;

    let credentials: Vec<ImportedCredential> = match payload {
        ImportPayload::Encrypted(items) => items
            .into_iter()
            .map(|item| ImportedCredential {
                label: item.label,
                url: item.url,
                enc_username: item.enc_username,
                enc_password: item.enc_password,
                nonce_username: item.nonce_username,
                nonce_password: item.nonce_password,
            })
            .collect(),
        ImportPayload::PlaintextWrapped {
            format,
            warning,
            exported_at,
            credentials,
        } => {
            if let Some(fmt) = format {
                println!("ℹ️  Detected import format: {}", fmt);
            }
            if warning.is_some() {
                println!("ℹ️  Importing plaintext credentials (they will be encrypted before storage)");
            }
            if exported_at.is_some() {
                println!("ℹ️  Plaintext export metadata detected");
            }

            let key = get_session_key()
                .context("Plaintext import requires active session. Run: chacrab login")?;

            let mut items = Vec::with_capacity(credentials.len());
            for credential in credentials {
                let (enc_username, nonce_username) = encrypt_data(&key, &credential.username)
                    .with_context(|| format!("Failed to encrypt username for '{}'", credential.label))?;
                let (enc_password, nonce_password) = encrypt_data(&key, &credential.password)
                    .with_context(|| format!("Failed to encrypt password for '{}'", credential.label))?;

                items.push(ImportedCredential {
                    label: credential.label,
                    url: credential.url,
                    enc_username,
                    enc_password,
                    nonce_username,
                    nonce_password,
                });
            }
            items
        }
        ImportPayload::Plaintext(credentials) => {
            println!("ℹ️  Detected plaintext credential list import");
            let key = get_session_key()
                .context("Plaintext import requires active session. Run: chacrab login")?;

            let mut items = Vec::with_capacity(credentials.len());
            for credential in credentials {
                let (enc_username, nonce_username) = encrypt_data(&key, &credential.username)
                    .with_context(|| format!("Failed to encrypt username for '{}'", credential.label))?;
                let (enc_password, nonce_password) = encrypt_data(&key, &credential.password)
                    .with_context(|| format!("Failed to encrypt password for '{}'", credential.label))?;

                items.push(ImportedCredential {
                    label: credential.label,
                    url: credential.url,
                    enc_username,
                    enc_password,
                    nonce_username,
                    nonce_password,
                });
            }
            items
        }
    };

    if credentials.is_empty() {
        println!("⚠️  No credentials found in import file");
        return Ok(());
    }

    println!("📦 Found {} credential(s) in import file\n", credentials.len());

    let mut imported = 0;
    let mut skipped = 0;
    let mut errors = 0;

    for cred in credentials {
        // Check if label already exists
        let existing = get_credential_by_label(pool, &cred.label).await?;

        if existing.is_some() {
            println!("⚠️  Credential '{}' already exists", cred.label);
            
            let selection = if is_test_mode() {
                match test_env("CHACRAB_IMPORT_DUPLICATE")
                    .unwrap_or_else(|| "skip".to_string())
                    .to_ascii_lowercase()
                    .as_str()
                {
                    "skip" => 0,
                    "overwrite" => 1,
                    "rename" => 2,
                    _ => 0,
                }
            } else {
                let choices = vec!["Skip", "Overwrite", "Rename (add suffix)"];
                Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("What would you like to do?")
                    .items(&choices)
                    .default(0)
                    .interact()
                    .context("Failed to read user choice")?
            };

            match selection {
                0 => {
                    // Skip
                    println!("   ⏭️  Skipped\n");
                    skipped += 1;
                    continue;
                }
                1 => {
                    // Overwrite - delete existing and insert new
                    use crate::storage::delete_credential_by_label;
                    delete_credential_by_label(pool, &cred.label)
                        .await
                        .context(format!("Failed to delete existing credential '{}'", cred.label))?;
                    
                    // Insert the imported credential
                    match insert_credential(
                        pool,
                        &cred.label,
                        cred.url.as_deref(),
                        &cred.enc_username,
                        &cred.enc_password,
                        &cred.nonce_username,
                        &cred.nonce_password,
                    )
                    .await
                    {
                        Ok(_) => {
                            println!("   ✅ Overwritten\n");
                            imported += 1;
                        }
                        Err(e) => {
                            println!("   ❌ Failed to import: {}\n", e);
                            errors += 1;
                        }
                    }
                }
                2 => {
                    // Rename - add suffix
                    let mut new_label = format!("{}_imported", cred.label);
                    let mut suffix = 1;
                    
                    // Keep trying until we find a unique label
                    while get_credential_by_label(pool, &new_label).await?.is_some() {
                        new_label = format!("{}_{}", cred.label, suffix);
                        suffix += 1;
                    }
                    
                    match insert_credential(
                        pool,
                        &new_label,
                        cred.url.as_deref(),
                        &cred.enc_username,
                        &cred.enc_password,
                        &cred.nonce_username,
                        &cred.nonce_password,
                    )
                    .await
                    {
                        Ok(_) => {
                            println!("   ✅ Imported as '{}'\n", new_label);
                            imported += 1;
                        }
                        Err(e) => {
                            println!("   ❌ Failed to import: {}\n", e);
                            errors += 1;
                        }
                    }
                }
                _ => unreachable!(),
            }
        } else {
            // No duplicate, insert directly
            match insert_credential(
                pool,
                &cred.label,
                cred.url.as_deref(),
                &cred.enc_username,
                &cred.enc_password,
                &cred.nonce_username,
                &cred.nonce_password,
            )
            .await
            {
                Ok(_) => {
                    println!("✅ Imported '{}'", cred.label);
                    imported += 1;
                }
                Err(e) => {
                    println!("❌ Failed to import '{}': {}", cred.label, e);
                    errors += 1;
                }
            }
        }
    }

    println!("\n📊 Import Summary:");
    println!("   ✅ Imported: {}", imported);
    if skipped > 0 {
        println!("   ⏭️  Skipped: {}", skipped);
    }
    if errors > 0 {
        println!("   ❌ Errors: {}", errors);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::storage::db::init_db;
    use crate::storage::{create_user_config, insert_credential, get_credential_by_label};
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_import_parses_valid_json() {
        let db = init_db("sqlite::memory:").await.unwrap();
        let pool = db.pool();
        create_user_config(pool).await.unwrap();

        // Create temporary JSON file
        let mut temp_file = NamedTempFile::new().unwrap();
        let json_data = json!([
            {
                "id": 1,
                "label": "ImportTest1",
                "url": "https://test1.com",
                "enc_username": "enc_u1",
                "enc_password": "enc_p1",
                "nonce_username": "n_u1",
                "nonce_password": "n_p1",
                "created_at": "2024-01-01T00:00:00",
                "updated_at": "2024-01-01T00:00:00"
            }
        ]);
        write!(temp_file, "{}", serde_json::to_string_pretty(&json_data).unwrap()).unwrap();

        // Parse the file
        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        let credentials: Vec<crate::models::Credential> = serde_json::from_str(&content).unwrap();
        
        assert_eq!(credentials.len(), 1);
        assert_eq!(credentials[0].label, "ImportTest1");
        assert_eq!(credentials[0].url, Some("https://test1.com".to_string()));
    }

    #[tokio::test]
    async fn test_import_detect_duplicate() {
        let db = init_db("sqlite::memory:").await.unwrap();
        let pool = db.pool();
        create_user_config(pool).await.unwrap();

        // Insert existing credential
        insert_credential(pool, "Duplicate", None, "old_u", "old_p", "n1", "n2").await.unwrap();

        // Check if detected as duplicate
        let existing = get_credential_by_label(pool, "Duplicate").await.unwrap();
        assert!(existing.is_some());
    }

    #[tokio::test]
    async fn test_import_multiple_credentials() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let json_data = json!([
            {
                "id": 1,
                "label": "Multi1",
                "url": null,
                "enc_username": "e1",
                "enc_password": "p1",
                "nonce_username": "n1",
                "nonce_password": "n2",
                "created_at": "2024-01-01T00:00:00",
                "updated_at": "2024-01-01T00:00:00"
            },
            {
                "id": 2,
                "label": "Multi2",
                "url": "https://example.com",
                "enc_username": "e2",
                "enc_password": "p2",
                "nonce_username": "n3",
                "nonce_password": "n4",
                "created_at": "2024-01-01T00:00:00",
                "updated_at": "2024-01-01T00:00:00"
            }
        ]);
        write!(temp_file, "{}", serde_json::to_string_pretty(&json_data).unwrap()).unwrap();

        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        let credentials: Vec<crate::models::Credential> = serde_json::from_str(&content).unwrap();
        
        assert_eq!(credentials.len(), 2);
        assert_eq!(credentials[0].label, "Multi1");
        assert_eq!(credentials[1].label, "Multi2");
    }
}
