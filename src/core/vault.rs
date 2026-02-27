use chrono::Utc;
use secrecy::{ExposeSecret, SecretString};
use serde_json::{Value, json};
use uuid::Uuid;
use zeroize::Zeroize;

use crate::{
    core::{
        crypto,
        errors::{ChacrabError, ChacrabResult},
        models::{EncryptedPayload, NewVaultItem, SyncTombstone, VaultItem, VaultItemType},
    },
    storage::r#trait::VaultRepository,
};

pub struct VaultService<R: VaultRepository> {
    repository: R,
}

impl<R: VaultRepository> VaultService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn add_password(
        &self,
        title: String,
        username: Option<String>,
        url: Option<String>,
        password: SecretString,
        notes: Option<String>,
        key: &[u8; crypto::KEY_SIZE],
    ) -> ChacrabResult<VaultItem> {
        let payload = EncryptedPayload::for_password(password, notes);
        self.add_item(
            NewVaultItem {
                r#type: VaultItemType::Password,
                title,
                username,
                url,
                payload,
            },
            key,
        )
        .await
    }

    pub async fn add_note(
        &self,
        title: String,
        notes: SecretString,
        key: &[u8; crypto::KEY_SIZE],
    ) -> ChacrabResult<VaultItem> {
        let payload = EncryptedPayload::for_note(notes);
        self.add_item(
            NewVaultItem {
                r#type: VaultItemType::Note,
                title,
                username: None,
                url: None,
                payload,
            },
            key,
        )
        .await
    }

    pub async fn update_password(
        &self,
        id: Uuid,
        title: Option<String>,
        username: Option<String>,
        url: Option<String>,
        password: Option<SecretString>,
        notes: Option<Option<String>>,
        key: &[u8; crypto::KEY_SIZE],
    ) -> ChacrabResult<VaultItem> {
        let mut item = self.repository.get_item(id).await?;
        if item.r#type != VaultItemType::Password {
            return Err(ChacrabError::Config(
                "item type mismatch for update".to_owned(),
            ));
        }

        let mut payload = self.decrypt_payload(&item, key)?;

        if let Some(next_title) = title {
            item.title = next_title;
        }
        if let Some(next_username) = username {
            item.username = Some(next_username);
        }
        if let Some(next_url) = url {
            item.url = Some(next_url);
        }
        if let Some(next_password) = password {
            payload.password = Some(next_password.expose_secret().to_owned());
        }
        if let Some(next_notes) = notes {
            payload.notes = next_notes;
        }

        Self::append_audit_event(&mut payload, "update_password");
        self.persist_item_update(item, payload, key).await
    }

    pub async fn update_note(
        &self,
        id: Uuid,
        title: Option<String>,
        notes: Option<SecretString>,
        key: &[u8; crypto::KEY_SIZE],
    ) -> ChacrabResult<VaultItem> {
        let mut item = self.repository.get_item(id).await?;
        if item.r#type != VaultItemType::Note {
            return Err(ChacrabError::Config(
                "item type mismatch for update".to_owned(),
            ));
        }

        let mut payload = self.decrypt_payload(&item, key)?;

        if let Some(next_title) = title {
            item.title = next_title;
        }
        if let Some(next_notes) = notes {
            payload.notes = Some(next_notes.expose_secret().to_owned());
        }

        Self::append_audit_event(&mut payload, "update_note");
        self.persist_item_update(item, payload, key).await
    }

    async fn add_item(
        &self,
        new_item: NewVaultItem,
        key: &[u8; crypto::KEY_SIZE],
    ) -> ChacrabResult<VaultItem> {
        let mut serialized = serde_json::to_vec(&new_item.payload)?;
        let encrypted = crypto::encrypt(key, &serialized)?;
        crypto::zeroize_vec(&mut serialized);

        let now = Utc::now();
        let item = VaultItem {
            id: Uuid::new_v4(),
            r#type: new_item.r#type,
            title: new_item.title,
            username: new_item.username,
            url: new_item.url,
            encrypted_data: encrypted.ciphertext,
            nonce: encrypted.nonce,
            sync_version: 1,
            created_at: now,
            updated_at: now,
        };
        self.repository.upsert_item(&item).await?;
        Ok(item)
    }

    fn decrypt_payload(
        &self,
        item: &VaultItem,
        key: &[u8; crypto::KEY_SIZE],
    ) -> ChacrabResult<EncryptedPayload> {
        let mut plaintext = crypto::decrypt(key, &item.nonce, &item.encrypted_data)?;
        let payload: EncryptedPayload = serde_json::from_slice(&plaintext)?;
        plaintext.zeroize();
        Ok(payload)
    }

    async fn persist_item_update(
        &self,
        mut item: VaultItem,
        payload: EncryptedPayload,
        key: &[u8; crypto::KEY_SIZE],
    ) -> ChacrabResult<VaultItem> {
        let mut serialized = serde_json::to_vec(&payload)?;
        let encrypted = crypto::encrypt(key, &serialized)?;
        crypto::zeroize_vec(&mut serialized);

        item.encrypted_data = encrypted.ciphertext;
        item.nonce = encrypted.nonce;
        item.sync_version = item.sync_version.saturating_add(1);
        item.updated_at = Utc::now();

        self.repository.upsert_item(&item).await?;
        Ok(item)
    }

    fn append_audit_event(payload: &mut EncryptedPayload, action: &str) {
        let now = Utc::now().to_rfc3339();
        let event = json!({
            "action": action,
            "at": now,
        });

        let entry = payload
            .custom_fields
            .entry("_audit".to_owned())
            .or_insert_with(|| Value::Array(Vec::new()));

        if let Value::Array(events) = entry {
            events.push(event);
            if events.len() > 20 {
                let overflow = events.len() - 20;
                events.drain(0..overflow);
            }
            return;
        }

        payload
            .custom_fields
            .insert("_audit".to_owned(), Value::Array(vec![event]));
    }

    pub async fn list(&self) -> ChacrabResult<Vec<VaultItem>> {
        self.repository.list_items().await
    }

    pub async fn show_decrypted(
        &self,
        id: Uuid,
        key: &[u8; crypto::KEY_SIZE],
    ) -> ChacrabResult<(VaultItem, Value)> {
        let item = self.repository.get_item(id).await?;
        let mut plaintext = crypto::decrypt(key, &item.nonce, &item.encrypted_data)?;
        let payload: Value = serde_json::from_slice(&plaintext)?;
        plaintext.zeroize();
        Ok((item, payload))
    }

    pub async fn delete(&self, id: Uuid) -> ChacrabResult<()> {
        let next_sync_version = match self.repository.get_item(id).await {
            Ok(item) => item.sync_version.saturating_add(1),
            Err(_) => self
                .repository
                .list_tombstones()
                .await?
                .into_iter()
                .find(|entry| entry.id == id)
                .map(|entry| entry.sync_version.saturating_add(1))
                .unwrap_or(1),
        };

        self.repository.delete_item(id).await?;
        self.repository
            .upsert_tombstone(&SyncTombstone {
                id,
                deleted_at: Utc::now(),
                sync_version: next_sync_version,
            })
            .await
    }

    pub fn repository(&self) -> &R {
        &self.repository
    }
}
