use crate::{
    core::{
        errors::{ChacrabError, ChacrabResult},
        models::VaultItem,
    },
    storage::r#trait::VaultRepository,
};

pub struct SyncEngine;

#[derive(Debug, Clone, Copy, Default)]
pub struct SyncReport {
    pub uploaded: u64,
    pub downloaded: u64,
}

impl SyncEngine {
    pub async fn sync_bidirectional<R: VaultRepository>(
        local: &R,
        remote: &R,
    ) -> ChacrabResult<SyncReport> {
        let local_items = local.list_items().await?;
        let remote_items = remote.list_items().await?;
        let mut report = SyncReport::default();

        if local_items
            .iter()
            .any(|item| !Self::validate_encrypted_blob_only(item))
            || remote_items
                .iter()
                .any(|item| !Self::validate_encrypted_blob_only(item))
        {
            return Err(ChacrabError::Config(
                "sync rejected invalid encrypted payload".to_owned(),
            ));
        }

        let mut index = std::collections::HashMap::new();
        for item in remote_items {
            index.insert(item.id, item);
        }

        for local_item in local_items {
            if let Some(remote_item) = index.get(&local_item.id) {
                if local_item.updated_at > remote_item.updated_at {
                    remote.upsert_item(&local_item).await?;
                    report.uploaded += 1;
                } else if remote_item.updated_at > local_item.updated_at {
                    local.upsert_item(remote_item).await?;
                    report.downloaded += 1;
                }
            } else {
                remote.upsert_item(&local_item).await?;
                report.uploaded += 1;
            }
        }

        let local_ids: std::collections::HashSet<_> = local
            .list_items()
            .await?
            .into_iter()
            .map(|item| item.id)
            .collect();

        for remote_item in remote.list_items().await? {
            if !local_ids.contains(&remote_item.id) {
                local.upsert_item(&remote_item).await?;
                report.downloaded += 1;
            }
        }

        Ok(report)
    }

    pub fn validate_encrypted_blob_only(item: &VaultItem) -> bool {
        !item.encrypted_data.is_empty() && item.nonce.len() == 12
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use async_trait::async_trait;
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    use crate::{
        core::{
            errors::{ChacrabError, ChacrabResult},
            models::{AuthRecord, VaultItem, VaultItemType},
        },
        storage::r#trait::VaultRepository,
    };

    use super::SyncEngine;

    #[derive(Clone, Default)]
    struct MemoryRepo {
        items: Arc<Mutex<HashMap<Uuid, VaultItem>>>,
    }

    #[async_trait]
    impl VaultRepository for MemoryRepo {
        async fn init(&self) -> ChacrabResult<()> {
            Ok(())
        }

        async fn upsert_item(&self, item: &VaultItem) -> ChacrabResult<()> {
            self.items
                .lock()
                .expect("poisoned")
                .insert(item.id, item.clone());
            Ok(())
        }

        async fn list_items(&self) -> ChacrabResult<Vec<VaultItem>> {
            Ok(self
                .items
                .lock()
                .expect("poisoned")
                .values()
                .cloned()
                .collect())
        }

        async fn get_item(&self, id: Uuid) -> ChacrabResult<VaultItem> {
            self.items
                .lock()
                .expect("poisoned")
                .get(&id)
                .cloned()
                .ok_or(ChacrabError::NotFound)
        }

        async fn delete_item(&self, id: Uuid) -> ChacrabResult<()> {
            self.items.lock().expect("poisoned").remove(&id);
            Ok(())
        }

        async fn get_auth_record(&self) -> ChacrabResult<Option<AuthRecord>> {
            Ok(None)
        }

        async fn set_auth_record(&self, _: &AuthRecord) -> ChacrabResult<()> {
            Ok(())
        }
    }

    fn build_item(id: Uuid, title: &str, updated_at: chrono::DateTime<Utc>) -> VaultItem {
        VaultItem {
            id,
            r#type: VaultItemType::Password,
            title: title.to_owned(),
            username: None,
            url: None,
            encrypted_data: vec![1, 2, 3],
            nonce: [7u8; 12],
            created_at: updated_at,
            updated_at,
        }
    }

    #[tokio::test]
    async fn sync_reports_uploads_and_downloads() {
        let local = MemoryRepo::default();
        let remote = MemoryRepo::default();
        let now = Utc::now();
        let same_id = Uuid::new_v4();

        let local_newer = build_item(same_id, "local newer", now + Duration::seconds(60));
        let remote_older = build_item(same_id, "remote older", now);
        let local_only = build_item(Uuid::new_v4(), "local only", now);
        let remote_only = build_item(Uuid::new_v4(), "remote only", now);

        local.upsert_item(&local_newer).await.expect("local upsert");
        local.upsert_item(&local_only).await.expect("local upsert");
        remote
            .upsert_item(&remote_older)
            .await
            .expect("remote upsert");
        remote
            .upsert_item(&remote_only)
            .await
            .expect("remote upsert");

        let report = SyncEngine::sync_bidirectional(&local, &remote)
            .await
            .expect("sync should succeed");

        assert_eq!(report.uploaded, 2);
        assert_eq!(report.downloaded, 1);
        assert_eq!(local.list_items().await.expect("local list").len(), 3);
        assert_eq!(remote.list_items().await.expect("remote list").len(), 3);
    }
}
