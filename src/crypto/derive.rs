use anyhow::{Context, Result};
use argon2::{
    password_hash::SaltString, Argon2, Params, PasswordHasher, Version,
};

/// Derives a 32-byte encryption key from a master password and salt using Argon2id.
///
/// Configuration:
/// - Memory: 64 MB
/// - Iterations: 3
/// - Parallelism: 4
/// - Output: 32 bytes (256 bits)
pub fn derive_key(master_password: &str, salt_str: &str) -> Result<[u8; 32]> {
    // Parse the salt string
    let salt = SaltString::from_b64(salt_str)
        .map_err(|e| anyhow::anyhow!("Failed to parse salt string: {:?}", e))?;

    // Configure Argon2id with specified parameters
    // Memory cost: 64MB = 65536 KB
    let params = Params::new(65536, 3, 4, Some(32))
        .map_err(|e| anyhow::anyhow!("Failed to create Argon2 parameters: {:?}", e))?;

    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        Version::V0x13,
        params,
    );

    // Hash the password
    let password_hash = argon2
        .hash_password(master_password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Failed to derive key from password: {:?}", e))?;

    // Extract the raw hash bytes
    let hash_bytes = password_hash
        .hash
        .context("Password hash missing hash output")?;

    // Convert to fixed-size array
    let mut key = [0u8; 32];
    key.copy_from_slice(hash_bytes.as_bytes());

    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use argon2::password_hash::SaltString;
    use rand::rngs::OsRng;

    #[test]
    fn test_derive_key_deterministic() {
        let password = "test_password_123";
        let salt = SaltString::generate(&mut OsRng);
        let salt_str = salt.as_str();

        let key1 = derive_key(password, salt_str).unwrap();
        let key2 = derive_key(password, salt_str).unwrap();

        assert_eq!(key1, key2, "Same password+salt should produce same key");
    }

    #[test]
    fn test_derive_key_different_passwords() {
        let salt = SaltString::generate(&mut OsRng);
        let salt_str = salt.as_str();

        let key1 = derive_key("password1", salt_str).unwrap();
        let key2 = derive_key("password2", salt_str).unwrap();

        assert_ne!(key1, key2, "Different passwords should produce different keys");
    }

    #[test]
    fn test_derive_key_different_salts() {
        let password = "same_password";
        let salt1 = SaltString::generate(&mut OsRng);
        let salt2 = SaltString::generate(&mut OsRng);

        let key1 = derive_key(password, salt1.as_str()).unwrap();
        let key2 = derive_key(password, salt2.as_str()).unwrap();

        assert_ne!(key1, key2, "Different salts should produce different keys");
    }

    #[test]
    fn test_derive_key_length() {
        let password = "test";
        let salt = SaltString::generate(&mut OsRng);
        let key = derive_key(password, salt.as_str()).unwrap();

        assert_eq!(key.len(), 32, "Derived key should be 32 bytes");
    }
}
