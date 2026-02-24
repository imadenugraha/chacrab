use crate::{
    core::{
        errors::{ChacrabError, ChacrabResult},
        models::{SyncTombstone, VaultItem},
    },
    storage::r#trait::VaultRepository,
};

pub struct SyncEngine;

#[derive(Debug, Clone, Default)]
pub struct SyncReport {
    pub uploaded: u64,
    pub downloaded: u64,
    pub conflicts: u64,
    pub replay_blocked: u64,
    pub conflict_ids: Vec<uuid::Uuid>,
}

#[derive(Debug, Clone)]
enum SyncState {
    Item(VaultItem),
    Tombstone(SyncTombstone),
}

impl SyncEngine {
    pub async fn sync_bidirectional<R: VaultRepository>(
        local: &R,
        remote: &R,
    ) -> ChacrabResult<SyncReport> {
        let local_items = local.list_items().await?;
        let remote_items = remote.list_items().await?;
        let local_tombstones = local.list_tombstones().await?;
        let remote_tombstones = remote.list_tombstones().await?;
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

        let mut local_index = std::collections::HashMap::new();
        let mut remote_index = std::collections::HashMap::new();
        let mut all_ids = std::collections::HashSet::new();

        for item in local_items {
            all_ids.insert(item.id);
            local_index.insert(item.id, SyncState::Item(item));
        }

        for tombstone in local_tombstones {
            all_ids.insert(tombstone.id);
            local_index.insert(tombstone.id, SyncState::Tombstone(tombstone));
        }

        for item in remote_items {
            all_ids.insert(item.id);
            remote_index.insert(item.id, SyncState::Item(item));
        }

        for tombstone in remote_tombstones {
            all_ids.insert(tombstone.id);
            remote_index.insert(tombstone.id, SyncState::Tombstone(tombstone));
        }

        for id in all_ids {
            let local_state = local_index.get(&id);
            let remote_state = remote_index.get(&id);

            let Some(resolution) = Self::resolve_state(id, local_state, remote_state) else {
                continue;
            };

            if resolution.conflict {
                report.conflicts += 1;
                report.conflict_ids.push(id);
            }
            if resolution.replay_blocked {
                report.replay_blocked += 1;
            }

            match &resolution.winner {
                SyncState::Item(item) => {
                    if !Self::state_matches_winner(local_state, &resolution.winner) {
                        local.upsert_item(item).await?;
                        local.delete_tombstone(id).await?;
                        report.downloaded += 1;
                    }
                    if !Self::state_matches_winner(remote_state, &resolution.winner) {
                        remote.upsert_item(item).await?;
                        remote.delete_tombstone(id).await?;
                        report.uploaded += 1;
                    }
                }
                SyncState::Tombstone(tombstone) => {
                    if !Self::state_matches_winner(local_state, &resolution.winner) {
                        local.delete_item(id).await?;
                        local.upsert_tombstone(tombstone).await?;
                        report.downloaded += 1;
                    }
                    if !Self::state_matches_winner(remote_state, &resolution.winner) {
                        remote.delete_item(id).await?;
                        remote.upsert_tombstone(tombstone).await?;
                        report.uploaded += 1;
                    }
                }
            }
        }

        Ok(report)
    }

    pub fn validate_encrypted_blob_only(item: &VaultItem) -> bool {
        !item.encrypted_data.is_empty() && item.nonce.len() == 12
    }

    fn state_matches_winner(current: Option<&SyncState>, winner: &SyncState) -> bool {
        match (current, winner) {
            (Some(SyncState::Item(left)), SyncState::Item(right)) => Self::same_item(left, right),
            (Some(SyncState::Tombstone(left)), SyncState::Tombstone(right)) => left == right,
            _ => false,
        }
    }

    fn same_item(left: &VaultItem, right: &VaultItem) -> bool {
        left.id == right.id
            && left.r#type == right.r#type
            && left.title == right.title
            && left.username == right.username
            && left.url == right.url
            && left.encrypted_data == right.encrypted_data
            && left.nonce == right.nonce
            && left.sync_version == right.sync_version
            && left.created_at == right.created_at
            && left.updated_at == right.updated_at
    }

    fn resolve_state(
        _id: uuid::Uuid,
        local: Option<&SyncState>,
        remote: Option<&SyncState>,
    ) -> Option<Resolution> {
        match (local.cloned(), remote.cloned()) {
            (None, None) => None,
            (Some(state), None) | (None, Some(state)) => Some(Resolution {
                winner: state,
                conflict: false,
                replay_blocked: false,
            }),
            (Some(local_state), Some(remote_state)) => {
                if Self::state_equivalent(&local_state, &remote_state) {
                    return Some(Resolution {
                        winner: local_state,
                        conflict: false,
                        replay_blocked: false,
                    });
                }

                let local_version = Self::state_version(&local_state);
                let remote_version = Self::state_version(&remote_state);

                if remote_version < local_version {
                    return Some(Resolution {
                        winner: local_state,
                        conflict: true,
                        replay_blocked: true,
                    });
                }

                if local_version < remote_version {
                    return Some(Resolution {
                        winner: remote_state,
                        conflict: true,
                        replay_blocked: false,
                    });
                }

                let local_time = Self::state_timestamp(&local_state);
                let remote_time = Self::state_timestamp(&remote_state);

                if local_time > remote_time {
                    return Some(Resolution {
                        winner: local_state,
                        conflict: true,
                        replay_blocked: false,
                    });
                }

                if remote_time > local_time {
                    return Some(Resolution {
                        winner: remote_state,
                        conflict: true,
                        replay_blocked: false,
                    });
                }

                let winner = match (&local_state, &remote_state) {
                    (SyncState::Tombstone(_), SyncState::Item(_)) => local_state,
                    (SyncState::Item(_), SyncState::Tombstone(_)) => remote_state,
                    (SyncState::Item(local_item), SyncState::Item(remote_item)) => {
                        if local_item.encrypted_data >= remote_item.encrypted_data {
                            local_state
                        } else {
                            remote_state
                        }
                    }
                    (SyncState::Tombstone(_), SyncState::Tombstone(_)) => local_state,
                };

                Some(Resolution {
                    winner,
                    conflict: true,
                    replay_blocked: false,
                })
            }
        }
    }

    fn state_timestamp(state: &SyncState) -> chrono::DateTime<chrono::Utc> {
        match state {
            SyncState::Item(item) => item.updated_at,
            SyncState::Tombstone(tombstone) => tombstone.deleted_at,
        }
    }

    fn state_version(state: &SyncState) -> u64 {
        match state {
            SyncState::Item(item) => item.sync_version,
            SyncState::Tombstone(tombstone) => tombstone.sync_version,
        }
    }

    fn state_equivalent(left: &SyncState, right: &SyncState) -> bool {
        match (left, right) {
            (SyncState::Item(local), SyncState::Item(remote)) => Self::same_item(local, remote),
            (SyncState::Tombstone(local), SyncState::Tombstone(remote)) => local == remote,
            _ => false,
        }
    }
}

struct Resolution {
    winner: SyncState,
    conflict: bool,
    replay_blocked: bool,
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
            models::{AuthRecord, SyncTombstone, VaultItem, VaultItemType},
        },
        storage::r#trait::VaultRepository,
    };

    use super::SyncEngine;

    #[derive(Clone, Default)]
    struct MemoryRepo {
        items: Arc<Mutex<HashMap<Uuid, VaultItem>>>,
        tombstones: Arc<Mutex<HashMap<Uuid, SyncTombstone>>>,
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

        async fn upsert_tombstone(&self, tombstone: &SyncTombstone) -> ChacrabResult<()> {
            self.tombstones
                .lock()
                .expect("poisoned")
                .insert(tombstone.id, tombstone.clone());
            Ok(())
        }

        async fn list_tombstones(&self) -> ChacrabResult<Vec<SyncTombstone>> {
            Ok(self
                .tombstones
                .lock()
                .expect("poisoned")
                .values()
                .cloned()
                .collect())
        }

        async fn delete_tombstone(&self, id: Uuid) -> ChacrabResult<()> {
            self.tombstones.lock().expect("poisoned").remove(&id);
            Ok(())
        }

        async fn get_auth_record(&self) -> ChacrabResult<Option<AuthRecord>> {
            Ok(None)
        }

        async fn set_auth_record(&self, _: &AuthRecord) -> ChacrabResult<()> {
            Ok(())
        }
    }

    fn build_item(
        id: Uuid,
        title: &str,
        updated_at: chrono::DateTime<Utc>,
        sync_version: u64,
    ) -> VaultItem {
        VaultItem {
            id,
            r#type: VaultItemType::Password,
            title: title.to_owned(),
            username: None,
            url: None,
            encrypted_data: vec![1, 2, 3],
            nonce: [7u8; 12],
            sync_version,
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

        let local_newer = build_item(same_id, "local newer", now + Duration::seconds(60), 2);
        let remote_older = build_item(same_id, "remote older", now, 1);
        let local_only = build_item(Uuid::new_v4(), "local only", now, 1);
        let remote_only = build_item(Uuid::new_v4(), "remote only", now, 1);

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
        assert_eq!(report.conflicts, 1);
        assert_eq!(report.replay_blocked, 1);
        assert_eq!(local.list_items().await.expect("local list").len(), 3);
        assert_eq!(remote.list_items().await.expect("remote list").len(), 3);
    }

    #[tokio::test]
    async fn tombstone_wins_tie_and_deletes_item() {
        let local = MemoryRepo::default();
        let remote = MemoryRepo::default();
        let now = Utc::now();
        let same_id = Uuid::new_v4();

        let remote_item = build_item(same_id, "remote live", now, 3);
        let local_tombstone = SyncTombstone {
            id: same_id,
            deleted_at: now,
            sync_version: 3,
        };

        remote
            .upsert_item(&remote_item)
            .await
            .expect("remote upsert");
        local
            .upsert_tombstone(&local_tombstone)
            .await
            .expect("local tombstone upsert");

        let report = SyncEngine::sync_bidirectional(&local, &remote)
            .await
            .expect("sync should succeed");

        assert_eq!(report.conflicts, 1);
        assert_eq!(report.uploaded, 1);
        assert_eq!(remote.list_items().await.expect("remote list").len(), 0);
        assert_eq!(local.list_items().await.expect("local list").len(), 0);
        assert_eq!(
            remote
                .list_tombstones()
                .await
                .expect("remote tombstones")
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn newer_remote_version_downloads_without_replay_block() {
        let local = MemoryRepo::default();
        let remote = MemoryRepo::default();
        let now = Utc::now();
        let same_id = Uuid::new_v4();

        let local_item = build_item(same_id, "local", now + Duration::seconds(120), 1);
        let remote_item = build_item(same_id, "remote", now, 2);

        local.upsert_item(&local_item).await.expect("local upsert");
        remote
            .upsert_item(&remote_item)
            .await
            .expect("remote upsert");

        let report = SyncEngine::sync_bidirectional(&local, &remote)
            .await
            .expect("sync should succeed");

        assert_eq!(report.downloaded, 1);
        assert_eq!(report.uploaded, 0);
        assert_eq!(report.replay_blocked, 0);
        let final_local = local.get_item(same_id).await.expect("final local item");
        assert_eq!(final_local.sync_version, 2);
        assert_eq!(final_local.title, "remote");
    }
}
