use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::core::{
    crypto,
    errors::{ChacrabError, ChacrabResult},
    models::VaultItem,
};

const BACKUP_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupPayload {
    pub schema_version: u32,
    pub exported_at: String,
    pub items: Vec<VaultItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedBackupFile {
    pub format_version: u32,
    pub nonce_b64: String,
    pub ciphertext_b64: String,
    pub checksum_hex: String,
}

pub fn export_encrypted(items: Vec<VaultItem>, key: &[u8; crypto::KEY_SIZE]) -> ChacrabResult<EncryptedBackupFile> {
    let payload = BackupPayload {
        schema_version: BACKUP_FORMAT_VERSION,
        exported_at: Utc::now().to_rfc3339(),
        items,
    };

    let serialized = serde_json::to_vec(&payload)?;
    let encrypted = crypto::encrypt(key, &serialized)?;

    let mut hasher = Sha256::new();
    hasher.update(encrypted.nonce);
    hasher.update(&encrypted.ciphertext);
    let checksum = hasher.finalize();

    Ok(EncryptedBackupFile {
        format_version: BACKUP_FORMAT_VERSION,
        nonce_b64: STANDARD.encode(encrypted.nonce),
        ciphertext_b64: STANDARD.encode(encrypted.ciphertext),
        checksum_hex: hex::encode(checksum),
    })
}

pub fn import_encrypted(
    backup_file: &EncryptedBackupFile,
    key: &[u8; crypto::KEY_SIZE],
) -> ChacrabResult<BackupPayload> {
    if backup_file.format_version != BACKUP_FORMAT_VERSION {
        return Err(ChacrabError::Config("unsupported backup format version".to_owned()));
    }

    let nonce_bytes = STANDARD
        .decode(backup_file.nonce_b64.as_bytes())
        .map_err(|_| ChacrabError::Serialization)?;
    if nonce_bytes.len() != crypto::NONCE_SIZE {
        return Err(ChacrabError::Serialization);
    }

    let ciphertext = STANDARD
        .decode(backup_file.ciphertext_b64.as_bytes())
        .map_err(|_| ChacrabError::Serialization)?;

    let mut hasher = Sha256::new();
    hasher.update(&nonce_bytes);
    hasher.update(&ciphertext);
    let expected = hex::encode(hasher.finalize());
    if expected != backup_file.checksum_hex {
        return Err(ChacrabError::Crypto);
    }

    let mut nonce = [0u8; crypto::NONCE_SIZE];
    nonce.copy_from_slice(&nonce_bytes);
    let plaintext = crypto::decrypt(key, &nonce, &ciphertext)?;

    let payload: BackupPayload = serde_json::from_slice(&plaintext)?;
    Ok(payload)
}
