use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::core::{
    errors::{ChacrabError, ChacrabResult},
    models::{AuthRecord, VaultItem, VaultItemType},
};
use crate::storage::r#trait::VaultRepository;

const SCHEMA_VERSION: i64 = 1;

#[derive(Clone)]
pub struct SqliteRepository {
    pool: SqlitePool,
}

impl SqliteRepository {
    pub async fn connect(database_url: &str) -> ChacrabResult<Self> {
        let normalized_url = Self::normalize_sqlite_url(database_url);
        let pool = SqlitePool::connect(&normalized_url).await?;
        Ok(Self { pool })
    }

    fn normalize_sqlite_url(database_url: &str) -> String {
        if !database_url.starts_with("sqlite://") {
            return database_url.to_owned();
        }

        if database_url.contains("mode=") {
            return database_url.to_owned();
        }

        if database_url.contains('?') {
            format!("{database_url}&mode=rwc")
        } else {
            format!("{database_url}?mode=rwc")
        }
    }

    fn parse_item_type(value: &str) -> ChacrabResult<VaultItemType> {
        match value {
            "password" => Ok(VaultItemType::Password),
            "note" => Ok(VaultItemType::Note),
            _ => Err(ChacrabError::Storage),
        }
    }

    fn item_type_to_str(item_type: &VaultItemType) -> &'static str {
        match item_type {
            VaultItemType::Password => "password",
            VaultItemType::Note => "note",
        }
    }
}

#[async_trait]
impl VaultRepository for SqliteRepository {
    async fn init(&self) -> ChacrabResult<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS schema_meta (
                id INTEGER PRIMARY KEY,
                schema_version INTEGER NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "INSERT INTO schema_meta (id, schema_version)
             VALUES (1, ?1)
             ON CONFLICT(id) DO UPDATE SET schema_version = excluded.schema_version",
        )
        .bind(SCHEMA_VERSION)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS auth (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                salt TEXT NOT NULL,
                verifier TEXT NOT NULL,
                argon2_m_cost INTEGER NOT NULL,
                argon2_t_cost INTEGER NOT NULL,
                argon2_p_cost INTEGER NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS vault_items (
                id TEXT PRIMARY KEY,
                item_type TEXT NOT NULL,
                title TEXT NOT NULL,
                username TEXT,
                url TEXT,
                encrypted_data BLOB NOT NULL,
                nonce BLOB NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn upsert_item(&self, item: &VaultItem) -> ChacrabResult<()> {
        sqlx::query(
            "INSERT INTO vault_items (id, item_type, title, username, url, encrypted_data, nonce, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
               item_type=excluded.item_type,
               title=excluded.title,
               username=excluded.username,
               url=excluded.url,
               encrypted_data=excluded.encrypted_data,
               nonce=excluded.nonce,
               created_at=excluded.created_at,
               updated_at=excluded.updated_at",
        )
        .bind(item.id.to_string())
        .bind(Self::item_type_to_str(&item.r#type))
        .bind(&item.title)
        .bind(&item.username)
        .bind(&item.url)
        .bind(&item.encrypted_data)
        .bind(item.nonce.to_vec())
        .bind(item.created_at.to_rfc3339())
        .bind(item.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn list_items(&self) -> ChacrabResult<Vec<VaultItem>> {
        let rows = sqlx::query(
            "SELECT id, item_type, title, username, url, encrypted_data, nonce, created_at, updated_at FROM vault_items ORDER BY updated_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                let nonce_blob: Vec<u8> = row.try_get("nonce")?;
                if nonce_blob.len() != 12 {
                    return Err(ChacrabError::Storage);
                }
                let mut nonce = [0u8; 12];
                nonce.copy_from_slice(&nonce_blob);

                let id_text: String = row.try_get("id")?;
                let item_type_text: String = row.try_get("item_type")?;
                let created_at_text: String = row.try_get("created_at")?;
                let updated_at_text: String = row.try_get("updated_at")?;

                let created_at = DateTime::parse_from_rfc3339(&created_at_text)
                    .map_err(|_| ChacrabError::Storage)?
                    .with_timezone(&Utc);
                let updated_at = DateTime::parse_from_rfc3339(&updated_at_text)
                    .map_err(|_| ChacrabError::Storage)?
                    .with_timezone(&Utc);

                Ok(VaultItem {
                    id: Uuid::parse_str(&id_text).map_err(|_| ChacrabError::Storage)?,
                    r#type: Self::parse_item_type(&item_type_text)?,
                    title: row.try_get("title")?,
                    username: row.try_get("username")?,
                    url: row.try_get("url")?,
                    encrypted_data: row.try_get("encrypted_data")?,
                    nonce,
                    created_at,
                    updated_at,
                })
            })
            .collect::<Result<Vec<_>, ChacrabError>>()
    }

    async fn get_item(&self, id: Uuid) -> ChacrabResult<VaultItem> {
        let row = sqlx::query(
            "SELECT id, item_type, title, username, url, encrypted_data, nonce, created_at, updated_at FROM vault_items WHERE id = ?1",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?
        .ok_or(ChacrabError::NotFound)?;

        let nonce_blob: Vec<u8> = row.try_get("nonce")?;
        if nonce_blob.len() != 12 {
            return Err(ChacrabError::Storage);
        }
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&nonce_blob);

        let item_type_text: String = row.try_get("item_type")?;
        let created_at_text: String = row.try_get("created_at")?;
        let updated_at_text: String = row.try_get("updated_at")?;

        let created_at = DateTime::parse_from_rfc3339(&created_at_text)
            .map_err(|_| ChacrabError::Storage)?
            .with_timezone(&Utc);
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_text)
            .map_err(|_| ChacrabError::Storage)?
            .with_timezone(&Utc);

        Ok(VaultItem {
            id,
            r#type: Self::parse_item_type(&item_type_text)?,
            title: row.try_get("title")?,
            username: row.try_get("username")?,
            url: row.try_get("url")?,
            encrypted_data: row.try_get("encrypted_data")?,
            nonce,
            created_at,
            updated_at,
        })
    }

    async fn delete_item(&self, id: Uuid) -> ChacrabResult<()> {
        let result = sqlx::query("DELETE FROM vault_items WHERE id = ?1")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(ChacrabError::NotFound);
        }
        Ok(())
    }

    async fn get_auth_record(&self) -> ChacrabResult<Option<AuthRecord>> {
        let row = sqlx::query(
            "SELECT salt, verifier, argon2_m_cost, argon2_t_cost, argon2_p_cost FROM auth WHERE id = 1",
        )
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| {
            Ok(AuthRecord {
                salt: r.try_get("salt")?,
                verifier: r.try_get("verifier")?,
                argon2_m_cost: r.try_get("argon2_m_cost")?,
                argon2_t_cost: r.try_get("argon2_t_cost")?,
                argon2_p_cost: r.try_get("argon2_p_cost")?,
            })
        })
        .transpose()
    }

    async fn set_auth_record(&self, auth: &AuthRecord) -> ChacrabResult<()> {
        sqlx::query(
            "INSERT INTO auth (id, salt, verifier, argon2_m_cost, argon2_t_cost, argon2_p_cost)
             VALUES (1, ?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
               salt=excluded.salt,
               verifier=excluded.verifier,
               argon2_m_cost=excluded.argon2_m_cost,
               argon2_t_cost=excluded.argon2_t_cost,
               argon2_p_cost=excluded.argon2_p_cost",
        )
        .bind(&auth.salt)
        .bind(&auth.verifier)
        .bind(auth.argon2_m_cost)
        .bind(auth.argon2_t_cost)
        .bind(auth.argon2_p_cost)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
