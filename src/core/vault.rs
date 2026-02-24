use chrono::Utc;
use secrecy::SecretString;
use serde_json::Value;
use uuid::Uuid;
use zeroize::Zeroize;

use crate::{
    core::{
        crypto,
        errors::ChacrabResult,
        models::{EncryptedPayload, NewVaultItem, VaultItem, VaultItemType},
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
            created_at: now,
            updated_at: now,
        };
        self.repository.upsert_item(&item).await?;
        Ok(item)
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
        self.repository.delete_item(id).await
    }

    pub fn repository(&self) -> &R {
        &self.repository
    }
}
