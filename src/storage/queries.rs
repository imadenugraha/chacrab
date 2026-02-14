use anyhow::{Context, Result};
use argon2::password_hash::SaltString;
use rand::rngs::OsRng;
use sqlx::{Postgres, Sqlite};

use crate::models::{Credential, UserConfig};
use crate::storage::db::DatabasePool;

/// Get user configuration (salt and verification token)
pub(crate) async fn get_user_config(pool: &DatabasePool) -> Result<Option<UserConfig>> {
    let config = match pool {
        DatabasePool::Sqlite(p) => {
            sqlx::query_as::<Sqlite, UserConfig>(
                "SELECT id, salt, verification_token, verification_nonce, created_at, updated_at FROM user_config LIMIT 1"
            )
            .fetch_optional(p)
            .await
            .context("Failed to fetch user config")?
        }
        DatabasePool::Postgres(p) => {
            sqlx::query_as::<Postgres, UserConfig>(
                "SELECT id, salt, verification_token, verification_nonce, created_at, updated_at FROM user_config LIMIT 1"
            )
            .fetch_optional(p)
            .await
            .context("Failed to fetch user config")?
        }
    };

    Ok(config)
}

/// Create initial user configuration with random salt
pub(crate) async fn create_user_config(pool: &DatabasePool) -> Result<UserConfig> {
    // Generate random salt
    let salt = SaltString::generate(&mut OsRng);
    let salt_str = salt.as_str();

    // Insert into database
    let config = match pool {
        DatabasePool::Sqlite(p) => {
            let result = sqlx::query("INSERT INTO user_config (salt) VALUES (?)")
                .bind(salt_str)
                .execute(p)
                .await
                .context("Failed to create user config")?;

            let id = result.last_insert_rowid();

            sqlx::query_as::<Sqlite, UserConfig>(
                "SELECT id, salt, verification_token, verification_nonce, created_at, updated_at FROM user_config WHERE id = ?"
            )
            .bind(id)
            .fetch_one(p)
            .await
            .context("Failed to fetch created user config")?
        }
        DatabasePool::Postgres(p) => {
            sqlx::query_as::<Postgres, UserConfig>(
                "INSERT INTO user_config (salt) VALUES ($1) 
                 RETURNING id, salt, verification_token, verification_nonce, created_at, updated_at"
            )
            .bind(salt_str)
            .fetch_one(p)
            .await
            .context("Failed to create user config")?
        }
    };

    Ok(config)
}

/// Update verification token for user config
pub(crate) async fn update_verification_token(
    pool: &DatabasePool,
    verification_token: &str,
    verification_nonce: &str,
) -> Result<()> {
    match pool {
        DatabasePool::Sqlite(p) => {
            sqlx::query(
                "UPDATE user_config SET verification_token = ?, verification_nonce = ?, updated_at = datetime('now')"
            )
            .bind(verification_token)
            .bind(verification_nonce)
            .execute(p)
            .await
            .context("Failed to update verification token")?;
        }
        DatabasePool::Postgres(p) => {
            sqlx::query(
                "UPDATE user_config SET verification_token = $1, verification_nonce = $2, updated_at = CURRENT_TIMESTAMP"
            )
            .bind(verification_token)
            .bind(verification_nonce)
            .execute(p)
            .await
            .context("Failed to update verification token")?;
        }
    }

    Ok(())
}

/// Insert a new credential
pub(crate) async fn insert_credential(
    pool: &DatabasePool,
    label: &str,
    url: Option<&str>,
    enc_username: &str,
    enc_password: &str,
    nonce_username: &str,
    nonce_password: &str,
) -> Result<i64> {
    let id = match pool {
        DatabasePool::Sqlite(p) => {
            let result = sqlx::query(
                "INSERT INTO credentials (label, url, enc_username, enc_password, nonce_username, nonce_password) 
                 VALUES (?, ?, ?, ?, ?, ?)"
            )
            .bind(label)
            .bind(url)
            .bind(enc_username)
            .bind(enc_password)
            .bind(nonce_username)
            .bind(nonce_password)
            .execute(p)
            .await
            .context("Failed to insert credential")?;

            result.last_insert_rowid()
        }
        DatabasePool::Postgres(p) => {
            let row: (i64,) = sqlx::query_as(
                "INSERT INTO credentials (label, url, enc_username, enc_password, nonce_username, nonce_password) 
                 VALUES ($1, $2, $3, $4, $5, $6) RETURNING id"
            )
            .bind(label)
            .bind(url)
            .bind(enc_username)
            .bind(enc_password)
            .bind(nonce_username)
            .bind(nonce_password)
            .fetch_one(p)
            .await
            .context("Failed to insert credential")?;

            row.0
        }
    };

    Ok(id)
}

/// Get a credential by label
pub(crate) async fn get_credential_by_label(pool: &DatabasePool, label: &str) -> Result<Option<Credential>> {
    let credential = match pool {
        DatabasePool::Sqlite(p) => {
            sqlx::query_as::<Sqlite, Credential>(
                "SELECT id, label, url, enc_username, enc_password, nonce_username, nonce_password, created_at, updated_at 
                 FROM credentials WHERE label = ?"
            )
            .bind(label)
            .fetch_optional(p)
            .await
            .context("Failed to fetch credential")?
        }
        DatabasePool::Postgres(p) => {
            sqlx::query_as::<Postgres, Credential>(
                "SELECT id, label, url, enc_username, enc_password, nonce_username, nonce_password, created_at, updated_at 
                 FROM credentials WHERE label = $1"
            )
            .bind(label)
            .fetch_optional(p)
            .await
            .context("Failed to fetch credential")?
        }
    };

    Ok(credential)
}

/// List all credentials (returns label, url, created_at only - no decryption needed)
pub(crate) async fn list_all_credentials(pool: &DatabasePool) -> Result<Vec<Credential>> {
    let credentials = match pool {
        DatabasePool::Sqlite(p) => {
            sqlx::query_as::<Sqlite, Credential>(
                "SELECT id, label, url, enc_username, enc_password, nonce_username, nonce_password, created_at, updated_at 
                 FROM credentials ORDER BY label ASC"
            )
            .fetch_all(p)
            .await
            .context("Failed to list credentials")?
        }
        DatabasePool::Postgres(p) => {
            sqlx::query_as::<Postgres, Credential>(
                "SELECT id, label, url, enc_username, enc_password, nonce_username, nonce_password, created_at, updated_at 
                 FROM credentials ORDER BY label ASC"
            )
            .fetch_all(p)
            .await
            .context("Failed to list credentials")?
        }
    };

    Ok(credentials)
}

/// Delete a credential by label
pub(crate) async fn delete_credential_by_label(pool: &DatabasePool, label: &str) -> Result<bool> {
    let rows_affected = match pool {
        DatabasePool::Sqlite(p) => {
            let result = sqlx::query("DELETE FROM credentials WHERE label = ?")
                .bind(label)
                .execute(p)
                .await
                .context("Failed to delete credential")?;
            result.rows_affected()
        }
        DatabasePool::Postgres(p) => {
            let result = sqlx::query("DELETE FROM credentials WHERE label = $1")
                .bind(label)
                .execute(p)
                .await
                .context("Failed to delete credential")?;
            result.rows_affected()
        }
    };

    Ok(rows_affected > 0)
}

/// Update a credential
pub(crate) async fn update_credential(
    pool: &DatabasePool,
    label: &str,
    url: Option<&str>,
    enc_username: &str,
    enc_password: &str,
    nonce_username: &str,
    nonce_password: &str,
) -> Result<bool> {
    let rows_affected = match pool {
        DatabasePool::Sqlite(p) => {
            let result = sqlx::query(
                "UPDATE credentials 
                 SET url = ?, enc_username = ?, enc_password = ?, nonce_username = ?, nonce_password = ?, updated_at = datetime('now')
                 WHERE label = ?"
            )
            .bind(url)
            .bind(enc_username)
            .bind(enc_password)
            .bind(nonce_username)
            .bind(nonce_password)
            .bind(label)
            .execute(p)
            .await
            .context("Failed to update credential")?;
            result.rows_affected()
        }
        DatabasePool::Postgres(p) => {
            let result = sqlx::query(
                "UPDATE credentials 
                 SET url = $1, enc_username = $2, enc_password = $3, nonce_username = $4, nonce_password = $5, updated_at = CURRENT_TIMESTAMP
                 WHERE label = $6"
            )
            .bind(url)
            .bind(enc_username)
            .bind(enc_password)
            .bind(nonce_username)
            .bind(nonce_password)
            .bind(label)
            .execute(p)
            .await
            .context("Failed to update credential")?;
            result.rows_affected()
        }
    };

    Ok(rows_affected > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::db::init_db;

    #[tokio::test]
    async fn test_user_config_operations() {
        let db = init_db("sqlite::memory:").await.unwrap();
        let pool = db.pool();

        // Initially no config
        let config = get_user_config(pool).await.unwrap();
        assert!(config.is_none());

        // Create config
        let config = create_user_config(pool).await.unwrap();
        assert!(!config.salt.is_empty());

        // Fetch again
        let fetched = get_user_config(pool).await.unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().salt, config.salt);
    }

    #[tokio::test]
    async fn test_credential_operations() {
        let db = init_db("sqlite::memory:").await.unwrap();
        let pool = db.pool();

        // Create user config first
        create_user_config(pool).await.unwrap();

        // Insert credential
        let id = insert_credential(
            pool,
            "GitHub",
            Some("https://github.com"),
            "enc_user",
            "enc_pass",
            "nonce_u",
            "nonce_p",
        )
        .await
        .unwrap();
        assert!(id > 0);

        // Get by label
        let cred = get_credential_by_label(pool, "GitHub").await.unwrap();
        assert!(cred.is_some());
        let cred = cred.unwrap();
        assert_eq!(cred.label, "GitHub");
        assert_eq!(cred.url, Some("https://github.com".to_string()));

        // List all
        let all = list_all_credentials(pool).await.unwrap();
        assert_eq!(all.len(), 1);

        // Delete
        let deleted = delete_credential_by_label(pool, "GitHub").await.unwrap();
        assert!(deleted);

        // Verify deleted
        let cred = get_credential_by_label(pool, "GitHub").await.unwrap();
        assert!(cred.is_none());
    }
}
