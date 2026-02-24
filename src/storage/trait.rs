use async_trait::async_trait;
use uuid::Uuid;

use crate::core::{
    errors::ChacrabResult,
    models::{AuthRecord, SyncTombstone, VaultItem},
};

#[async_trait]
pub trait VaultRepository: Send + Sync {
    async fn init(&self) -> ChacrabResult<()>;
    async fn upsert_item(&self, item: &VaultItem) -> ChacrabResult<()>;
    async fn list_items(&self) -> ChacrabResult<Vec<VaultItem>>;
    async fn get_item(&self, id: Uuid) -> ChacrabResult<VaultItem>;
    async fn delete_item(&self, id: Uuid) -> ChacrabResult<()>;

    async fn upsert_tombstone(&self, tombstone: &SyncTombstone) -> ChacrabResult<()>;
    async fn list_tombstones(&self) -> ChacrabResult<Vec<SyncTombstone>>;
    async fn delete_tombstone(&self, id: Uuid) -> ChacrabResult<()>;

    async fn get_auth_record(&self) -> ChacrabResult<Option<AuthRecord>>;
    async fn set_auth_record(&self, auth: &AuthRecord) -> ChacrabResult<()>;
}
