use secrecy::SecretString;

use chacrab::{
    auth::login,
    core::errors::{ChacrabError, ChacrabResult},
    storage::{app::AppRepository, r#trait::VaultRepository},
};

#[tokio::test]
async fn app_repository_rejects_unknown_backend() {
    let result = AppRepository::connect("unknown", "ignored").await;
    assert!(matches!(result, Err(ChacrabError::UnsupportedBackend(_))));
}

#[tokio::test]
async fn sqlite_backend_selection_and_auth_roundtrip() -> ChacrabResult<()> {
    let repo = AppRepository::connect("sqlite", "sqlite::memory:").await?;
    repo.init().await?;

    let master = SecretString::new("MasterPass12!".to_owned().into_boxed_str());
    login::register(&repo, master).await?;

    let auth = repo.get_auth_record().await?;
    assert!(auth.is_some());
    Ok(())
}

#[tokio::test]
async fn postgres_backend_selection_if_env_configured() -> ChacrabResult<()> {
    let Ok(url) = std::env::var("CHACRAB_TEST_POSTGRES_URL") else {
        return Ok(());
    };

    let repo = AppRepository::connect("postgres", &url).await?;
    repo.init().await?;
    Ok(())
}

#[tokio::test]
async fn mongo_backend_selection_if_env_configured() -> ChacrabResult<()> {
    let Ok(url) = std::env::var("CHACRAB_TEST_MONGO_URL") else {
        return Ok(());
    };

    let repo = AppRepository::connect("mongo", &url).await?;
    repo.init().await?;
    Ok(())
}
