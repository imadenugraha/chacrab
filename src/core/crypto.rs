use argon2::{password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString}, Algorithm, Argon2, Params, Version};
use chacha20poly1305::{aead::{Aead, KeyInit}, ChaCha20Poly1305, Key, Nonce};
use rand::RngCore;
use secrecy::{ExposeSecret, SecretString};
use zeroize::Zeroize;

use crate::core::errors::{ChacrabError, ChacrabResult};

pub const KEY_SIZE: usize = 32;
pub const NONCE_SIZE: usize = 12;
pub const SALT_LEN: usize = 16;
pub const ARGON2_M_COST: u32 = 65_536;
pub const ARGON2_T_COST: u32 = 3;
pub const ARGON2_P_COST: u32 = 1;

#[derive(Debug, Clone)]
pub struct CipherBlob {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
}

#[derive(Debug, Clone)]
pub struct RegistrationMaterial {
    pub salt_b64: String,
    pub verifier: String,
}

fn argon2_instance() -> ChacrabResult<Argon2<'static>> {
    let params = Params::new(ARGON2_M_COST, ARGON2_T_COST, ARGON2_P_COST, Some(KEY_SIZE))
        .map_err(|_| ChacrabError::Config("invalid argon2 parameters".to_owned()))?;
    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

pub fn generate_salt() -> String {
    SaltString::generate(&mut OsRng).to_string()
}

pub fn derive_key(master_password: &SecretString, salt_b64: &str) -> ChacrabResult<[u8; KEY_SIZE]> {
    let _ = SaltString::from_b64(salt_b64).map_err(|_| ChacrabError::InvalidCredentials)?;
    let argon2 = argon2_instance()?;
    let mut out = [0u8; KEY_SIZE];
    argon2
        .hash_password_into(master_password.expose_secret().as_bytes(), salt_b64.as_bytes(), &mut out)
        .map_err(|_| ChacrabError::InvalidCredentials)?;
    Ok(out)
}

pub fn create_registration_material(master_password: &SecretString) -> ChacrabResult<(RegistrationMaterial, [u8; KEY_SIZE])> {
    let salt = generate_salt();
    let derived = derive_key(master_password, &salt)?;
    let argon2 = argon2_instance()?;
    let salt_string = SaltString::from_b64(&salt).map_err(|_| ChacrabError::Crypto)?;
    let verifier = argon2
        .hash_password(&derived, &salt_string)
        .map_err(|_| ChacrabError::Crypto)?
        .to_string();

    Ok((
        RegistrationMaterial {
            salt_b64: salt,
            verifier,
        },
        derived,
    ))
}

pub fn verify_password(master_password: &SecretString, salt_b64: &str, verifier: &str) -> ChacrabResult<[u8; KEY_SIZE]> {
    let derived = derive_key(master_password, salt_b64)?;
    let parsed = PasswordHash::new(verifier).map_err(|_| ChacrabError::InvalidCredentials)?;
    let argon2 = argon2_instance()?;
    argon2.verify_password(&derived, &parsed).map_err(|_| ChacrabError::InvalidCredentials)?;
    Ok(derived)
}

pub fn encrypt(key_bytes: &[u8; KEY_SIZE], plaintext: &[u8]) -> ChacrabResult<CipherBlob> {
    let mut nonce = [0u8; NONCE_SIZE];
    let mut rng = rand::rng();
    rng.fill_bytes(&mut nonce);

    let key = Key::from_slice(key_bytes);
    let cipher = ChaCha20Poly1305::new(key);
    let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce), plaintext)?;

    Ok(CipherBlob { ciphertext, nonce })
}

pub fn decrypt(key_bytes: &[u8; KEY_SIZE], nonce: &[u8; NONCE_SIZE], ciphertext: &[u8]) -> ChacrabResult<Vec<u8>> {
    let key = Key::from_slice(key_bytes);
    let cipher = ChaCha20Poly1305::new(key);
    let plaintext = cipher.decrypt(Nonce::from_slice(nonce), ciphertext)?;
    Ok(plaintext)
}

pub fn zeroize_vec(buffer: &mut Vec<u8>) {
    buffer.zeroize();
}

#[cfg(test)]
mod tests {
    use secrecy::SecretString;

    use super::{
        create_registration_material, decrypt, derive_key, encrypt, verify_password, KEY_SIZE,
    };

    #[test]
    fn derive_and_verify_password_roundtrip() {
        let master_password = SecretString::new("MasterPass12!".to_owned().into_boxed_str());
        let (registration, derived) =
            create_registration_material(&master_password).expect("registration material");

        let verified = verify_password(
            &master_password,
            &registration.salt_b64,
            &registration.verifier,
        )
        .expect("password verification should pass");

        assert_eq!(derived.len(), KEY_SIZE);
        assert_eq!(verified.len(), KEY_SIZE);
        assert_eq!(verified, derived);
    }

    #[test]
    fn verify_rejects_wrong_password() {
        let master_password = SecretString::new("MasterPass12!".to_owned().into_boxed_str());
        let wrong_password = SecretString::new("WrongPass12!".to_owned().into_boxed_str());
        let (registration, _) =
            create_registration_material(&master_password).expect("registration material");

        let result = verify_password(
            &wrong_password,
            &registration.salt_b64,
            &registration.verifier,
        );
        assert!(result.is_err());
    }

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let master_password = SecretString::new("MasterPass12!".to_owned().into_boxed_str());
        let salt = super::generate_salt();
        let key = derive_key(&master_password, &salt).expect("key derivation");
        let plaintext = b"top secret payload";

        let blob = encrypt(&key, plaintext).expect("encryption");
        let decrypted = decrypt(&key, &blob.nonce, &blob.ciphertext).expect("decryption");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encryption_uses_random_nonce() {
        let master_password = SecretString::new("MasterPass12!".to_owned().into_boxed_str());
        let salt = super::generate_salt();
        let key = derive_key(&master_password, &salt).expect("key derivation");
        let plaintext = b"same plaintext";

        let first = encrypt(&key, plaintext).expect("first encryption");
        let second = encrypt(&key, plaintext).expect("second encryption");

        assert_ne!(first.nonce, second.nonce, "nonces should be randomly generated");
    }
}
