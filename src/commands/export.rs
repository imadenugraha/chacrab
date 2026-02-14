use anyhow::{bail, Context, Result};
use dialoguer::{Confirm, Password};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use serde::Serialize;

use crate::commands::verify_sentinel_constant_time;
use crate::crypto::{decrypt_data, derive_key};
use crate::models::DecryptedCredential;
use crate::storage::{get_user_config, list_all_credentials, Database};
use crate::ui::{is_test_mode, test_env};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Encrypted,
    Plaintext,
}

impl std::str::FromStr for ExportFormat {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value.to_lowercase().as_str() {
            "encrypted" | "enc" => Ok(Self::Encrypted),
            "plaintext" | "plain" => Ok(Self::Plaintext),
            _ => bail!("Invalid export format '{}'. Use 'encrypted' or 'plaintext'.", value),
        }
    }
}

#[derive(Serialize)]
struct PlaintextExport {
    format: &'static str,
    warning: &'static str,
    exported_at: chrono::DateTime<chrono::Utc>,
    credentials: Vec<DecryptedCredential>,
}

/// Export all credentials to a JSON file
///
/// Supports:
/// - encrypted (default): exports encrypted credentials
/// - plaintext: decrypts credentials and exports readable values (requires master password)
/// File permissions are set to 0600 (read/write owner only) for security.
pub async fn export_credentials(db: &Database, output_path: Option<PathBuf>, format: &str) -> Result<()> {
    let pool = db.pool();
    let format: ExportFormat = format.parse()?;

    // Determine output path
    let output_path = output_path.unwrap_or_else(|| {
        let suffix = match format {
            ExportFormat::Encrypted => "encrypted",
            ExportFormat::Plaintext => "plaintext",
        };
        PathBuf::from(format!(
            "chacrab-export-{}-{}.json",
            suffix,
            chrono::Utc::now().format("%Y%m%d-%H%M%S")
        ))
    });

    // Check if file exists
    if output_path.exists() {
        let overwrite = if is_test_mode() {
            true
        } else {
            Confirm::new()
                .with_prompt(format!("File '{}' already exists. Overwrite?", output_path.display()))
                .default(false)
                .interact()
                .context("Failed to read confirmation")?
        };

        if !overwrite {
            println!("❌ Export cancelled");
            return Ok(());
        }
    }

    let (json, exported_count) = match format {
        ExportFormat::Encrypted => {
            let credentials = list_all_credentials(pool)
                .await
                .context("Failed to fetch credentials from database")?;

            if credentials.is_empty() {
                println!("⚠️  No credentials to export");
                return Ok(());
            }

            let count = credentials.len();
            let json = serde_json::to_string_pretty(&credentials)
                .context("Failed to serialize credentials to JSON")?;
            (json, count)
        }
        ExportFormat::Plaintext => {
            println!("⚠️  Plaintext export selected.");
            println!("   This will export decrypted usernames and passwords.");

            let master_password = if is_test_mode() {
                test_env("CHACRAB_MASTER_PASSWORD").ok_or_else(|| {
                    anyhow::anyhow!(
                        "CHACRAB_TEST_MODE is enabled but CHACRAB_MASTER_PASSWORD is not set"
                    )
                })?
            } else {
                Password::new()
                    .with_prompt("Enter master password to decrypt for export")
                    .interact()
                    .context("Failed to read master password")?
            };

            let user_config = get_user_config(pool)
                .await
                .context("Failed to read vault configuration")?
                .ok_or_else(|| anyhow::anyhow!("Vault is not initialized"))?;

            let verification_token = user_config
                .verification_token
                .ok_or_else(|| anyhow::anyhow!("Vault verification token missing. Please re-initialize or login again."))?;
            let verification_nonce = user_config
                .verification_nonce
                .ok_or_else(|| anyhow::anyhow!("Vault verification nonce missing. Please re-initialize or login again."))?;

            let key = derive_key(&master_password, &user_config.salt)
                .context("Failed to derive key from master password")?;

            let verification = decrypt_data(&key, &verification_token, &verification_nonce)
                .context("Master password verification failed")?;
            if !verify_sentinel_constant_time(&verification) {
                bail!("Vault verification token mismatch. Cannot proceed with plaintext export.");
            }

            let proceed = if is_test_mode() {
                test_env("CHACRAB_ALLOW_PLAINTEXT_EXPORT")
                    .map(|value| {
                        matches!(
                            value.trim().to_ascii_lowercase().as_str(),
                            "1" | "true" | "yes" | "on"
                        )
                    })
                    .unwrap_or(false)
            } else {
                Confirm::new()
                    .with_prompt("I understand this file will contain PLAINTEXT credentials. Continue?")
                    .default(false)
                    .interact()
                    .context("Failed to read confirmation")?
            };

            if !proceed {
                println!("❌ Plaintext export cancelled");
                return Ok(());
            }

            let encrypted_credentials = list_all_credentials(pool)
                .await
                .context("Failed to fetch credentials from database")?;

            if encrypted_credentials.is_empty() {
                println!("⚠️  No credentials to export");
                return Ok(());
            }

            let mut plaintext_credentials = Vec::with_capacity(encrypted_credentials.len());
            for credential in encrypted_credentials {
                let username = decrypt_data(&key, &credential.enc_username, &credential.nonce_username)
                    .with_context(|| format!("Failed to decrypt username for '{}'", credential.label))?;
                let password = decrypt_data(&key, &credential.enc_password, &credential.nonce_password)
                    .with_context(|| format!("Failed to decrypt password for '{}'", credential.label))?;

                plaintext_credentials.push(DecryptedCredential {
                    label: credential.label,
                    url: credential.url,
                    username,
                    password,
                    created_at: credential.created_at,
                });
            }

            let count = plaintext_credentials.len();
            let payload = PlaintextExport {
                format: "plaintext",
                warning: "This file contains PLAINTEXT credentials. Keep it secure and delete it as soon as possible.",
                exported_at: chrono::Utc::now(),
                credentials: plaintext_credentials,
            };

            let json = serde_json::to_string_pretty(&payload)
                .context("Failed to serialize plaintext credentials to JSON")?;
            (json, count)
        }
    };

    // Write to file
    fs::write(&output_path, &json)
        .context(format!("Failed to write export to '{}'", output_path.display()))?;

    // Set file permissions to 0600 (read/write owner only)
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&output_path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&output_path, perms)
            .context("Failed to set file permissions")?;
    }

    println!("✅ Exported {} credential(s) to '{}'", exported_count, output_path.display());
    println!("🔒 File permissions set to 0600 (owner read/write only)");
    println!();
    match format {
        ExportFormat::Encrypted => {
            println!("⚠️  Important: This file contains ENCRYPTED credentials.");
            println!("   You still need your master password to decrypt and import them.");
        }
        ExportFormat::Plaintext => {
            println!("⚠️  CRITICAL: This file contains PLAINTEXT credentials.");
            println!("   Store it safely and delete it securely after use.");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::db::init_db;
    use crate::storage::{create_user_config, insert_credential};
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_export_creates_valid_json() {
        let db = init_db("sqlite::memory:").await.unwrap();
        let pool = db.pool();

        // Create user config and add test credentials
        create_user_config(pool).await.unwrap();
        insert_credential(
            pool,
            "TestExport",
            Some("https://example.com"),
            "encrypted_username",
            "encrypted_password",
            "nonce_username",
            "nonce_password",
        )
        .await
        .unwrap();

        // Export to temp file (non-interactive path)
        let temp_file = NamedTempFile::new().unwrap();
        let export_path = temp_file.path().to_path_buf();

        // Note: This test path avoids interactive confirmation
        // by using a new file that doesn't exist
        let credentials = list_all_credentials(pool).await.unwrap();
        let json = serde_json::to_string_pretty(&credentials).unwrap();
        std::fs::write(&export_path, json).unwrap();

        // Verify file exists and contains valid JSON
        assert!(export_path.exists());
        let content = std::fs::read_to_string(&export_path).unwrap();
        
        // Parse to verify it's valid JSON
        let parsed: Vec<crate::models::Credential> = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].label, "TestExport");
        assert_eq!(parsed[0].url, Some("https://example.com".to_string()));
    }

    #[tokio::test]
    async fn test_export_json_format() {
        let db = init_db("sqlite::memory:").await.unwrap();
        let pool = db.pool();

        create_user_config(pool).await.unwrap();
        insert_credential(
            pool,
            "Label1",
            None,
            "enc_u",
            "enc_p",
            "n_u",
            "n_p",
        )
        .await
        .unwrap();

        let credentials = list_all_credentials(pool).await.unwrap();
        let json = serde_json::to_string_pretty(&credentials).unwrap();

        // Verify JSON contains expected fields
        assert!(json.contains("Label1"));
        assert!(json.contains("enc_u"));
        assert!(json.contains("enc_p"));
        assert!(json.contains("n_u"));
        assert!(json.contains("n_p"));
    }

    #[tokio::test]
    async fn test_export_multiple_credentials() {
        let db = init_db("sqlite::memory:").await.unwrap();
        let pool = db.pool();

        create_user_config(pool).await.unwrap();
        
        // Add multiple credentials
        for i in 1..=5 {
            insert_credential(
                pool,
                &format!("Cred{}", i),
                Some(&format!("https://example{}.com", i)),
                &format!("enc_u{}", i),
                &format!("enc_p{}", i),
                &format!("n_u{}", i),
                &format!("n_p{}", i),
            )
            .await
            .unwrap();
        }

        let credentials = list_all_credentials(pool).await.unwrap();
        assert_eq!(credentials.len(), 5);

        let json = serde_json::to_string_pretty(&credentials).unwrap();
        let parsed: Vec<crate::models::Credential> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 5);
    }

    #[test]
    fn test_export_format_parse() {
        assert_eq!("encrypted".parse::<ExportFormat>().unwrap(), ExportFormat::Encrypted);
        assert_eq!("enc".parse::<ExportFormat>().unwrap(), ExportFormat::Encrypted);
        assert_eq!("plaintext".parse::<ExportFormat>().unwrap(), ExportFormat::Plaintext);
        assert_eq!("plain".parse::<ExportFormat>().unwrap(), ExportFormat::Plaintext);
        assert!("other".parse::<ExportFormat>().is_err());
    }
}
