use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use futures_util::TryStreamExt;
use mongodb::{
    Client, Collection, IndexModel,
    bson::{self, Binary, Bson, DateTime as BsonDateTime, Document, doc},
    options::{ClientOptions, IndexOptions},
};
use uuid::Uuid;

use crate::core::{
    errors::{ChacrabError, ChacrabResult},
    models::{AuthRecord, VaultItem, VaultItemType},
};
use crate::storage::r#trait::VaultRepository;

const SCHEMA_VERSION: i64 = 1;

#[derive(Clone)]
pub struct MongoRepository {
    vault_items: Collection<Document>,
    auth: Collection<Document>,
    metadata: Collection<Document>,
}

impl MongoRepository {
    pub async fn connect(database_url: &str) -> ChacrabResult<Self> {
        let mut options = ClientOptions::parse(database_url).await?;
        if options.default_database.is_none() {
            options.default_database = Some("chacrab".to_owned());
        }

        let client = Client::with_options(options)?;
        let database = client
            .default_database()
            .ok_or_else(|| ChacrabError::Config("missing mongo database name".to_owned()))?;

        Ok(Self {
            vault_items: database.collection("vault_items"),
            auth: database.collection("auth"),
            metadata: database.collection("metadata"),
        })
    }

    fn parse_item_type(value: &str) -> ChacrabResult<VaultItemType> {
        match value {
            "password" => Ok(VaultItemType::Password),
            "note" => Ok(VaultItemType::Note),
            _ => Err(ChacrabError::Storage),
        }
    }

    fn item_type_to_str(item_type: &VaultItemType) -> &'static str {
        match item_type {
            VaultItemType::Password => "password",
            VaultItemType::Note => "note",
        }
    }

    fn to_document(item: &VaultItem) -> Document {
        doc! {
            "id": item.id.to_string(),
            "item_type": Self::item_type_to_str(&item.r#type),
            "title": &item.title,
            "username": item.username.clone(),
            "url": item.url.clone(),
            "encrypted_data": Bson::Binary(Binary { subtype: bson::spec::BinarySubtype::Generic, bytes: item.encrypted_data.clone() }),
            "nonce": Bson::Binary(Binary { subtype: bson::spec::BinarySubtype::Generic, bytes: item.nonce.to_vec() }),
            "created_at": Bson::DateTime(BsonDateTime::from_millis(item.created_at.timestamp_millis())),
            "updated_at": Bson::DateTime(BsonDateTime::from_millis(item.updated_at.timestamp_millis())),
        }
    }

    fn from_document(document: Document) -> ChacrabResult<VaultItem> {
        let id_text = document.get_str("id").map_err(|_| ChacrabError::Storage)?;
        let item_type_text = document
            .get_str("item_type")
            .map_err(|_| ChacrabError::Storage)?;
        let encrypted_data = document
            .get_binary_generic("encrypted_data")
            .map_err(|_| ChacrabError::Storage)?
            .to_vec();
        let nonce_blob = document
            .get_binary_generic("nonce")
            .map_err(|_| ChacrabError::Storage)?
            .to_vec();

        if nonce_blob.len() != 12 {
            return Err(ChacrabError::Storage);
        }
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&nonce_blob);

        let created_at = document
            .get_datetime("created_at")
            .map_err(|_| ChacrabError::Storage)?
            .timestamp_millis();
        let updated_at = document
            .get_datetime("updated_at")
            .map_err(|_| ChacrabError::Storage)?
            .timestamp_millis();

        Ok(VaultItem {
            id: Uuid::parse_str(id_text).map_err(|_| ChacrabError::Storage)?,
            r#type: Self::parse_item_type(item_type_text)?,
            title: document
                .get_str("title")
                .map_err(|_| ChacrabError::Storage)?
                .to_owned(),
            username: document.get_str("username").ok().map(str::to_owned),
            url: document.get_str("url").ok().map(str::to_owned),
            encrypted_data,
            nonce,
            created_at: Utc
                .timestamp_millis_opt(created_at)
                .single()
                .ok_or(ChacrabError::Storage)?,
            updated_at: Utc
                .timestamp_millis_opt(updated_at)
                .single()
                .ok_or(ChacrabError::Storage)?,
        })
    }
}

#[async_trait]
impl VaultRepository for MongoRepository {
    async fn init(&self) -> ChacrabResult<()> {
        let unique_index = IndexModel::builder()
            .keys(doc! { "id": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();
        self.vault_items.create_index(unique_index).await?;

        self.metadata
            .update_one(
                doc! { "_id": "schema" },
                doc! { "$set": { "version": SCHEMA_VERSION } },
            )
            .upsert(true)
            .await?;

        Ok(())
    }

    async fn upsert_item(&self, item: &VaultItem) -> ChacrabResult<()> {
        self.vault_items
            .replace_one(doc! { "id": item.id.to_string() }, Self::to_document(item))
            .upsert(true)
            .await?;
        Ok(())
    }

    async fn list_items(&self) -> ChacrabResult<Vec<VaultItem>> {
        let mut cursor = self
            .vault_items
            .find(doc! {})
            .sort(doc! { "updated_at": -1 })
            .await?;

        let mut out = Vec::new();
        while let Some(document) = cursor.try_next().await? {
            out.push(Self::from_document(document)?);
        }
        Ok(out)
    }

    async fn get_item(&self, id: Uuid) -> ChacrabResult<VaultItem> {
        let document = self
            .vault_items
            .find_one(doc! { "id": id.to_string() })
            .await?
            .ok_or(ChacrabError::NotFound)?;
        Self::from_document(document)
    }

    async fn delete_item(&self, id: Uuid) -> ChacrabResult<()> {
        let result = self
            .vault_items
            .delete_one(doc! { "id": id.to_string() })
            .await?;

        if result.deleted_count == 0 {
            return Err(ChacrabError::NotFound);
        }
        Ok(())
    }

    async fn get_auth_record(&self) -> ChacrabResult<Option<AuthRecord>> {
        let document = self.auth.find_one(doc! { "id": 1 }).await?;
        document
            .map(|doc| {
                Ok(AuthRecord {
                    salt: doc
                        .get_str("salt")
                        .map_err(|_| ChacrabError::Storage)?
                        .to_owned(),
                    verifier: doc
                        .get_str("verifier")
                        .map_err(|_| ChacrabError::Storage)?
                        .to_owned(),
                    argon2_m_cost: doc
                        .get_i32("argon2_m_cost")
                        .map_err(|_| ChacrabError::Storage)?
                        as u32,
                    argon2_t_cost: doc
                        .get_i32("argon2_t_cost")
                        .map_err(|_| ChacrabError::Storage)?
                        as u32,
                    argon2_p_cost: doc
                        .get_i32("argon2_p_cost")
                        .map_err(|_| ChacrabError::Storage)?
                        as u32,
                })
            })
            .transpose()
    }

    async fn set_auth_record(&self, auth: &AuthRecord) -> ChacrabResult<()> {
        self.auth
            .update_one(
                doc! { "id": 1 },
                doc! {
                    "$set": {
                        "id": 1,
                        "salt": &auth.salt,
                        "verifier": &auth.verifier,
                        "argon2_m_cost": auth.argon2_m_cost as i32,
                        "argon2_t_cost": auth.argon2_t_cost as i32,
                        "argon2_p_cost": auth.argon2_p_cost as i32,
                    }
                },
            )
            .upsert(true)
            .await?;
        Ok(())
    }
}
