use std::path::PathBuf;

use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use chacrab::{
    core::{errors::ChacrabError, errors::ChacrabResult},
    storage::{sqlite::SqliteRepository, r#trait::VaultRepository},
};

fn temp_db_url() -> (String, PathBuf) {
    let mut path = std::env::temp_dir();
    path.push(format!("chacrab-nonce-{}.db", Uuid::new_v4()));
    (format!("sqlite://{}?mode=rwc", path.display()), path)
}

#[tokio::test]
async fn list_items_rejects_malformed_nonce_length() -> ChacrabResult<()> {
    let (url, path) = temp_db_url();
    let repo = SqliteRepository::connect(&url).await?;
    repo.init().await?;

    let pool = SqlitePool::connect(&url).await?;
    sqlx::query(
        "INSERT INTO vault_items (id, item_type, title, username, url, encrypted_data, nonce, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
    )
    .bind(Uuid::new_v4().to_string())
    .bind("password")
    .bind("Bad Nonce")
    .bind(Option::<String>::None)
    .bind(Option::<String>::None)
    .bind(vec![1u8, 2, 3])
    .bind(vec![7u8, 8, 9])
    .bind(Utc::now().to_rfc3339())
    .bind(Utc::now().to_rfc3339())
    .execute(&pool)
    .await?;

    let result = repo.list_items().await;
    assert!(matches!(result, Err(ChacrabError::Storage)));

    let _ = std::fs::remove_file(path);
    Ok(())
}

#[tokio::test]
async fn get_item_rejects_malformed_nonce_length() -> ChacrabResult<()> {
    let (url, path) = temp_db_url();
    let repo = SqliteRepository::connect(&url).await?;
    repo.init().await?;

    let bad_id = Uuid::new_v4();
    let pool = SqlitePool::connect(&url).await?;
    sqlx::query(
        "INSERT INTO vault_items (id, item_type, title, username, url, encrypted_data, nonce, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
    )
    .bind(bad_id.to_string())
    .bind("note")
    .bind("Bad Nonce")
    .bind(Option::<String>::None)
    .bind(Option::<String>::None)
    .bind(vec![1u8, 2, 3])
    .bind(vec![7u8, 8, 9, 10, 11])
    .bind(Utc::now().to_rfc3339())
    .bind(Utc::now().to_rfc3339())
    .execute(&pool)
    .await?;

    let result = repo.get_item(bad_id).await;
    assert!(matches!(result, Err(ChacrabError::Storage)));

    let _ = std::fs::remove_file(path);
    Ok(())
}
