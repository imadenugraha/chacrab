use base64::{Engine, engine::general_purpose::STANDARD};
use zeroize::Zeroize;

use crate::core::{crypto, errors::ChacrabResult};

const KEYRING_SERVICE: &str = "chacrab";
const KEYRING_USER: &str = "session-master-key";

pub fn store_session_key(key: &[u8; crypto::KEY_SIZE]) -> ChacrabResult<()> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)?;
    let mut encoded = STANDARD.encode(key);
    entry.set_password(&encoded)?;
    encoded.zeroize();
    Ok(())
}

pub fn load_session_key() -> ChacrabResult<[u8; crypto::KEY_SIZE]> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)?;
    let mut encoded = entry.get_password()?;
    let mut decoded = STANDARD
        .decode(encoded.as_bytes())
        .map_err(|_| crate::core::errors::ChacrabError::NoActiveSession)?;
    encoded.zeroize();
    if decoded.len() != crypto::KEY_SIZE {
        decoded.zeroize();
        return Err(crate::core::errors::ChacrabError::NoActiveSession);
    }
    let mut key = [0u8; crypto::KEY_SIZE];
    key.copy_from_slice(&decoded);
    decoded.zeroize();
    Ok(key)
}

pub fn clear_session_key() -> ChacrabResult<()> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)?;
    let _ = entry.delete_password();
    Ok(())
}
