use chrono::{DateTime, Utc};

/// Credential represents an encrypted credential entry in the database.
#[allow(dead_code)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Credential {
    pub id: i64,
    pub label: String,
    pub url: Option<String>,
    pub enc_username: String,
    pub enc_password: String,
    pub nonce_username: String,
    pub nonce_password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Decrypted credential for display
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DecryptedCredential {
    pub label: String,
    pub url: Option<String>,
    pub username: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
}
