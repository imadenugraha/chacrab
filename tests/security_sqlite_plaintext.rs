use secrecy::SecretString;

use chacrab::{
    auth::login,
    core::{crypto, errors::ChacrabResult, vault::VaultService},
    storage::{r#trait::VaultRepository, sqlite::SqliteRepository},
};

async fn session_key(repo: &SqliteRepository, master_password: &SecretString) -> ChacrabResult<[u8; 32]> {
    let auth = repo
        .get_auth_record()
        .await?
        .ok_or(chacrab::core::errors::ChacrabError::Storage)?;
    crypto::verify_password(master_password, &auth.salt, &auth.verifier)
}

#[tokio::test]
async fn sqlite_ciphertext_never_contains_password_or_note_plaintext() -> ChacrabResult<()> {
    let repo = SqliteRepository::connect("sqlite::memory:").await?;
    repo.init().await?;

    let master_password = SecretString::new("MasterPass12!".to_owned().into_boxed_str());
    login::register(&repo, master_password.clone()).await?;
    let key = session_key(&repo, &master_password).await?;

    let vault = VaultService::new(repo.clone());
    vault
        .add_password(
            "Email".to_owned(),
            Some("alice@example.com".to_owned()),
            Some("https://mail.example.com".to_owned()),
            SecretString::new("SuperSecret#123".to_owned().into_boxed_str()),
            Some("Recovery code: 123456".to_owned()),
            &key,
        )
        .await?;

    vault
        .add_note(
            "Private Note".to_owned(),
            SecretString::new("this should never be plaintext at rest".to_owned().into_boxed_str()),
            &key,
        )
        .await?;

    let rows = repo.list_items().await?;
    assert_eq!(rows.len(), 2);

    for row in rows {
        let blob_view = String::from_utf8_lossy(&row.encrypted_data);
        assert!(!blob_view.contains("SuperSecret#123"));
        assert!(!blob_view.contains("Recovery code: 123456"));
        assert!(!blob_view.contains("this should never be plaintext at rest"));
        assert!(!blob_view.contains("alice@example.com"));
    }

    Ok(())
}