use crate::{
    core::{errors::ChacrabResult, models::VaultItem},
    storage::r#trait::VaultRepository,
};

pub struct SyncEngine;

impl SyncEngine {
    pub async fn sync_bidirectional<R: VaultRepository>(local: &R, remote: &R) -> ChacrabResult<()> {
        let local_items = local.list_items().await?;
        let remote_items = remote.list_items().await?;

        let mut index = std::collections::HashMap::new();
        for item in remote_items {
            index.insert(item.id, item);
        }

        for local_item in local_items {
            if let Some(remote_item) = index.get(&local_item.id) {
                if local_item.updated_at > remote_item.updated_at {
                    remote.upsert_item(&local_item).await?;
                } else if remote_item.updated_at > local_item.updated_at {
                    local.upsert_item(remote_item).await?;
                }
            } else {
                remote.upsert_item(&local_item).await?;
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
            }
        }

        Ok(())
    }

    pub fn validate_encrypted_blob_only(item: &VaultItem) -> bool {
        !item.encrypted_data.is_empty() && item.nonce.len() == 12
    }
}
