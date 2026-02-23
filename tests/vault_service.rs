use secrecy::SecretString;
use uuid::Uuid;

use chacrab::{
    core::{crypto, errors::ChacrabResult, vault::VaultService},
    storage::{r#trait::VaultRepository, sqlite::SqliteRepository},
};

async fn build_service() -> ChacrabResult<(SqliteRepository, VaultService<SqliteRepository>, [u8; 32])> {
    let repo = SqliteRepository::connect("sqlite::memory:").await?;
    repo.init().await?;
    let master = SecretString::new("MasterPass12!".to_owned().into_boxed_str());
    let salt = crypto::generate_salt();
    let key = crypto::derive_key(&master, &salt)?;
    let service = VaultService::new(repo.clone());
    Ok((repo, service, key))
}

#[tokio::test]
async fn add_and_show_password_item() -> ChacrabResult<()> {
    let (_repo, service, key) = build_service().await?;

    let item = service
        .add_password(
            "GitHub".to_owned(),
            Some("moonliez".to_owned()),
            Some("https://github.com".to_owned()),
            SecretString::new("Secret#123".to_owned().into_boxed_str()),
            Some("2FA enabled".to_owned()),
            &key,
        )
        .await?;

    let (_stored, payload) = service.show_decrypted(item.id, &key).await?;
    assert_eq!(payload["password"].as_str(), Some("Secret#123"));
    assert_eq!(payload["notes"].as_str(), Some("2FA enabled"));

    Ok(())
}

#[tokio::test]
async fn delete_removes_item() -> ChacrabResult<()> {
    let (_repo, service, key) = build_service().await?;
    let item = service
        .add_note(
            "Recovery".to_owned(),
            SecretString::new("backup-codes".to_owned().into_boxed_str()),
            &key,
        )
        .await?;

    service.delete(item.id).await?;
    let result = service.show_decrypted(item.id, &key).await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn show_missing_item_fails() -> ChacrabResult<()> {
    let (_repo, service, key) = build_service().await?;
    let result = service.show_decrypted(Uuid::new_v4(), &key).await;
    assert!(result.is_err());
    Ok(())
}