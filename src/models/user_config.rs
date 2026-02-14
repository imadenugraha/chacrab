use chrono::{DateTime, Utc};

/// UserConfig represents the single-user configuration stored in the database.
/// Contains the salt used for key derivation and optional verification token.
#[allow(dead_code)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserConfig {
    pub id: i64,
    pub salt: String,
    pub verification_token: Option<String>,
    pub verification_nonce: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
