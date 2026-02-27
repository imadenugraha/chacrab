use secrecy::SecretString;
use uuid::Uuid;

use chacrab::{
    core::{crypto, errors::ChacrabResult, vault::VaultService},
    storage::{sqlite::SqliteRepository, r#trait::VaultRepository},
};

async fn build_service()
-> ChacrabResult<(SqliteRepository, VaultService<SqliteRepository>, [u8; 32])> {
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

#[tokio::test]
async fn update_password_updates_secret_and_audit_trail() -> ChacrabResult<()> {
    let (_repo, service, key) = build_service().await?;
    let item = service
        .add_password(
            "Email".to_owned(),
            Some("me@example.com".to_owned()),
            None,
            SecretString::new("old-secret".to_owned().into_boxed_str()),
            Some("initial".to_owned()),
            &key,
        )
        .await?;

    let updated = service
        .update_password(
            item.id,
            Some("Email Primary".to_owned()),
            None,
            None,
            Some(SecretString::new("new-secret".to_owned().into_boxed_str())),
            Some(Some("rotated".to_owned())),
            &key,
        )
        .await?;

    assert_eq!(updated.sync_version, item.sync_version + 1);
    let (_stored, payload) = service.show_decrypted(item.id, &key).await?;
    assert_eq!(payload["password"].as_str(), Some("new-secret"));
    assert_eq!(payload["notes"].as_str(), Some("rotated"));
    assert_eq!(payload["custom_fields"]["_audit"][0]["action"], "update_password");

    Ok(())
}

#[tokio::test]
async fn update_note_updates_content_and_audit_trail() -> ChacrabResult<()> {
    let (_repo, service, key) = build_service().await?;
    let item = service
        .add_note(
            "Recovery".to_owned(),
            SecretString::new("backup-codes".to_owned().into_boxed_str()),
            &key,
        )
        .await?;

    let updated = service
        .update_note(
            item.id,
            Some("Recovery Codes".to_owned()),
            Some(SecretString::new("new-codes".to_owned().into_boxed_str())),
            &key,
        )
        .await?;

    assert_eq!(updated.sync_version, item.sync_version + 1);
    let (_stored, payload) = service.show_decrypted(item.id, &key).await?;
    assert_eq!(payload["notes"].as_str(), Some("new-codes"));
    assert_eq!(payload["custom_fields"]["_audit"][0]["action"], "update_note");

    Ok(())
}
