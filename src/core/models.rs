use chrono::{DateTime, Utc};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum VaultItemType {
    Password,
    Note,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultItem {
    pub id: Uuid,
    pub r#type: VaultItemType,
    pub title: String,
    pub username: Option<String>,
    pub url: Option<String>,
    pub encrypted_data: Vec<u8>,
    pub nonce: [u8; 12],
    #[serde(default = "default_sync_version")]
    pub sync_version: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncTombstone {
    pub id: Uuid,
    pub deleted_at: DateTime<Utc>,
    #[serde(default = "default_sync_version")]
    pub sync_version: u64,
}

fn default_sync_version() -> u64 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedPayload {
    pub password: Option<String>,
    pub notes: Option<String>,
    pub custom_fields: serde_json::Map<String, serde_json::Value>,
}

impl EncryptedPayload {
    pub fn for_password(password: SecretString, notes: Option<String>) -> Self {
        Self {
            password: Some(password.expose_secret().to_owned()),
            notes,
            custom_fields: serde_json::Map::new(),
        }
    }

    pub fn for_note(notes: SecretString) -> Self {
        Self {
            password: None,
            notes: Some(notes.expose_secret().to_owned()),
            custom_fields: serde_json::Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRecord {
    pub salt: String,
    pub verifier: String,
    pub argon2_m_cost: u32,
    pub argon2_t_cost: u32,
    pub argon2_p_cost: u32,
}

#[derive(Debug, Clone)]
pub struct NewVaultItem {
    pub r#type: VaultItemType,
    pub title: String,
    pub username: Option<String>,
    pub url: Option<String>,
    pub payload: EncryptedPayload,
}
