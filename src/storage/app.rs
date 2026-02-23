use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    core::{
        errors::{ChacrabError, ChacrabResult},
        models::{AuthRecord, VaultItem},
    },
    storage::{
        mongo::MongoRepository,
        postgres::PostgresRepository,
        r#trait::VaultRepository,
        sqlite::SqliteRepository,
    },
};

#[derive(Clone)]
pub enum AppRepository {
    Sqlite(SqliteRepository),
    Postgres(PostgresRepository),
    Mongo(MongoRepository),
}

impl AppRepository {
    pub async fn connect(backend: &str, database_url: &str) -> ChacrabResult<Self> {
        match backend {
            "sqlite" => Ok(Self::Sqlite(SqliteRepository::connect(database_url).await?)),
            "postgres" => Ok(Self::Postgres(PostgresRepository::connect(database_url).await?)),
            "mongo" => Ok(Self::Mongo(MongoRepository::connect(database_url).await?)),
            other => Err(ChacrabError::UnsupportedBackend(other.to_owned())),
        }
    }
}

#[async_trait]
impl VaultRepository for AppRepository {
    async fn init(&self) -> ChacrabResult<()> {
        match self {
            AppRepository::Sqlite(repo) => repo.init().await,
            AppRepository::Postgres(repo) => repo.init().await,
            AppRepository::Mongo(repo) => repo.init().await,
        }
    }

    async fn upsert_item(&self, item: &VaultItem) -> ChacrabResult<()> {
        match self {
            AppRepository::Sqlite(repo) => repo.upsert_item(item).await,
            AppRepository::Postgres(repo) => repo.upsert_item(item).await,
            AppRepository::Mongo(repo) => repo.upsert_item(item).await,
        }
    }

    async fn list_items(&self) -> ChacrabResult<Vec<VaultItem>> {
        match self {
            AppRepository::Sqlite(repo) => repo.list_items().await,
            AppRepository::Postgres(repo) => repo.list_items().await,
            AppRepository::Mongo(repo) => repo.list_items().await,
        }
    }

    async fn get_item(&self, id: Uuid) -> ChacrabResult<VaultItem> {
        match self {
            AppRepository::Sqlite(repo) => repo.get_item(id).await,
            AppRepository::Postgres(repo) => repo.get_item(id).await,
            AppRepository::Mongo(repo) => repo.get_item(id).await,
        }
    }

    async fn delete_item(&self, id: Uuid) -> ChacrabResult<()> {
        match self {
            AppRepository::Sqlite(repo) => repo.delete_item(id).await,
            AppRepository::Postgres(repo) => repo.delete_item(id).await,
            AppRepository::Mongo(repo) => repo.delete_item(id).await,
        }
    }

    async fn get_auth_record(&self) -> ChacrabResult<Option<AuthRecord>> {
        match self {
            AppRepository::Sqlite(repo) => repo.get_auth_record().await,
            AppRepository::Postgres(repo) => repo.get_auth_record().await,
            AppRepository::Mongo(repo) => repo.get_auth_record().await,
        }
    }

    async fn set_auth_record(&self, auth: &AuthRecord) -> ChacrabResult<()> {
        match self {
            AppRepository::Sqlite(repo) => repo.set_auth_record(auth).await,
            AppRepository::Postgres(repo) => repo.set_auth_record(auth).await,
            AppRepository::Mongo(repo) => repo.set_auth_record(auth).await,
        }
    }
}