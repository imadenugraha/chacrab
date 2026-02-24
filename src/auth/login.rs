use secrecy::SecretString;
use zeroize::Zeroize;

use crate::{
    auth::keyring,
    core::{
        crypto,
        errors::{ChacrabError, ChacrabResult},
        models::AuthRecord,
    },
    storage::r#trait::VaultRepository,
};

pub trait SessionKeyStore {
    fn store(&self, key: &[u8; crypto::KEY_SIZE]) -> ChacrabResult<()>;
    fn load(&self) -> ChacrabResult<[u8; crypto::KEY_SIZE]>;
    fn clear(&self) -> ChacrabResult<()>;
}

struct OsSessionKeyStore;

impl SessionKeyStore for OsSessionKeyStore {
    fn store(&self, key: &[u8; crypto::KEY_SIZE]) -> ChacrabResult<()> {
        keyring::store_session_key(key)
    }

    fn load(&self) -> ChacrabResult<[u8; crypto::KEY_SIZE]> {
        keyring::load_session_key()
    }

    fn clear(&self) -> ChacrabResult<()> {
        keyring::clear_session_key()
    }
}

pub async fn register<R: VaultRepository>(
    repo: &R,
    master_password: SecretString,
) -> ChacrabResult<()> {
    let (material, mut derived) = crypto::create_registration_material(&master_password)?;

    let auth = AuthRecord {
        salt: material.salt_b64,
        verifier: material.verifier,
        argon2_m_cost: crypto::ARGON2_M_COST,
        argon2_t_cost: crypto::ARGON2_T_COST,
        argon2_p_cost: crypto::ARGON2_P_COST,
    };
    repo.set_auth_record(&auth).await?;

    derived.zeroize();
    Ok(())
}

pub async fn login<R: VaultRepository>(
    repo: &R,
    master_password: SecretString,
) -> ChacrabResult<()> {
    let key_store = OsSessionKeyStore;
    login_with_store(repo, master_password, &key_store).await
}

pub(crate) async fn login_with_store<R: VaultRepository, S: SessionKeyStore>(
    repo: &R,
    master_password: SecretString,
    key_store: &S,
) -> ChacrabResult<()> {
    let auth = repo
        .get_auth_record()
        .await?
        .ok_or_else(|| ChacrabError::Config("vault not initialized; run init".to_owned()))?;

    let mut derived = crypto::verify_password_with_params(
        &master_password,
        &auth.salt,
        &auth.verifier,
        auth.argon2_m_cost,
        auth.argon2_t_cost,
        auth.argon2_p_cost,
    )?;
    key_store.store(&derived)?;
    derived.zeroize();
    Ok(())
}

pub fn logout() -> ChacrabResult<()> {
    let key_store = OsSessionKeyStore;
    logout_with_store(&key_store)?;
    Ok(())
}

pub(crate) fn logout_with_store<S: SessionKeyStore>(key_store: &S) -> ChacrabResult<()> {
    key_store.clear()?;
    Ok(())
}

pub fn current_session_key() -> ChacrabResult<[u8; crypto::KEY_SIZE]> {
    let key_store = OsSessionKeyStore;
    current_session_key_with_store(&key_store)
}

pub(crate) fn current_session_key_with_store<S: SessionKeyStore>(
    key_store: &S,
) -> ChacrabResult<[u8; crypto::KEY_SIZE]> {
    key_store.load()
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use argon2::{Algorithm, Argon2, Params, PasswordHasher, Version, password_hash::SaltString};
    use async_trait::async_trait;

    use crate::{
        core::{
            errors::{ChacrabError, ChacrabResult},
            models::{AuthRecord, VaultItem},
        },
        storage::r#trait::VaultRepository,
    };

    use super::{
        SessionKeyStore, current_session_key_with_store, login_with_store, logout_with_store,
        register,
    };
    use secrecy::SecretString;
    use uuid::Uuid;

    #[derive(Clone, Default)]
    struct MemoryRepo {
        auth: Arc<Mutex<Option<AuthRecord>>>,
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
            Ok(self.auth.lock().expect("poisoned").clone())
        }

        async fn set_auth_record(&self, auth: &AuthRecord) -> ChacrabResult<()> {
            *self.auth.lock().expect("poisoned") = Some(auth.clone());
            Ok(())
        }
    }

    #[derive(Default)]
    struct MemorySessionStore {
        key: Mutex<Option<[u8; crate::core::crypto::KEY_SIZE]>>,
    }

    impl SessionKeyStore for MemorySessionStore {
        fn store(&self, key: &[u8; crate::core::crypto::KEY_SIZE]) -> ChacrabResult<()> {
            *self.key.lock().expect("poisoned") = Some(*key);
            Ok(())
        }

        fn load(&self) -> ChacrabResult<[u8; crate::core::crypto::KEY_SIZE]> {
            self.key
                .lock()
                .expect("poisoned")
                .as_ref()
                .copied()
                .ok_or(ChacrabError::NoActiveSession)
        }

        fn clear(&self) -> ChacrabResult<()> {
            *self.key.lock().expect("poisoned") = None;
            Ok(())
        }
    }

    #[tokio::test]
    async fn auth_lifecycle_register_login_logout() {
        let repo = MemoryRepo::default();
        let store = MemorySessionStore::default();
        let master_password = SecretString::new("MasterPass12!".to_owned().into_boxed_str());

        register(&repo, master_password.clone())
            .await
            .expect("register should succeed");

        login_with_store(&repo, master_password.clone(), &store)
            .await
            .expect("login should succeed");

        let loaded = current_session_key_with_store(&store).expect("session key should load");
        assert_eq!(loaded.len(), crate::core::crypto::KEY_SIZE);

        logout_with_store(&store).expect("logout should succeed");
        assert!(current_session_key_with_store(&store).is_err());
    }

    #[tokio::test]
    async fn login_honors_stored_argon2_parameters() {
        let repo = MemoryRepo::default();
        let store = MemorySessionStore::default();
        let master_password = SecretString::new("MasterPass12!".to_owned().into_boxed_str());
        let salt = crate::core::crypto::generate_salt();
        let custom_m = 32_768;
        let custom_t = 4;
        let custom_p = 1;

        let derived = crate::core::crypto::derive_key_with_params(
            &master_password,
            &salt,
            custom_m,
            custom_t,
            custom_p,
        )
        .expect("derive with custom argon2 params");

        let params = Params::new(
            custom_m,
            custom_t,
            custom_p,
            Some(crate::core::crypto::KEY_SIZE),
        )
        .expect("argon2 params");
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        let salt_string = SaltString::from_b64(&salt).expect("salt decode");
        let verifier = argon2
            .hash_password(&derived, &salt_string)
            .expect("verifier")
            .to_string();

        repo.set_auth_record(&AuthRecord {
            salt,
            verifier,
            argon2_m_cost: custom_m,
            argon2_t_cost: custom_t,
            argon2_p_cost: custom_p,
        })
        .await
        .expect("set auth");

        login_with_store(&repo, master_password, &store)
            .await
            .expect("login should use stored argon2 params");
        assert!(current_session_key_with_store(&store).is_ok());
    }
}
