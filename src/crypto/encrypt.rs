use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};
use rand::RngCore;

/// Encrypts plaintext data using ChaCha20-Poly1305 with the provided key.
///
/// Returns a tuple of (ciphertext_base64, nonce_base64).
pub fn encrypt_data(key: &[u8; 32], plaintext: &str) -> Result<(String, String)> {
    // Create cipher instance
    let cipher = ChaCha20Poly1305::new(key.into());

    // Generate random 12-byte nonce
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt the plaintext
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {:?}", e))?;

    // Encode both as base64
    let ciphertext_base64 = BASE64.encode(&ciphertext);
    let nonce_base64 = BASE64.encode(nonce_bytes);

    Ok((ciphertext_base64, nonce_base64))
}

/// Decrypts ciphertext using ChaCha20-Poly1305 with the provided key and nonce.
///
/// Both ciphertext and nonce should be base64-encoded strings.
pub fn decrypt_data(key: &[u8; 32], ciphertext_base64: &str, nonce_base64: &str) -> Result<String> {
    // Decode from base64
    let ciphertext = BASE64
        .decode(ciphertext_base64)
        .context("Failed to decode ciphertext from base64")?;

    let nonce_bytes = BASE64
        .decode(nonce_base64)
        .context("Failed to decode nonce from base64")?;

    // Validate nonce length
    if nonce_bytes.len() != 12 {
        anyhow::bail!("Invalid nonce length: expected 12 bytes, got {}", nonce_bytes.len());
    }

    let nonce = Nonce::from_slice(&nonce_bytes);

    // Create cipher instance
    let cipher = ChaCha20Poly1305::new(key.into());

    // Decrypt
    let plaintext_bytes = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| anyhow::anyhow!("Decryption failed - wrong key or corrupted data: {:?}", e))?;

    // Convert to string
    let plaintext = String::from_utf8(plaintext_bytes)
        .context("Decrypted data is not valid UTF-8")?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_key() -> [u8; 32] {
        [42u8; 32] // Simple test key
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = get_test_key();
        let plaintext = "Hello, ChaCrab! 🦀";

        let (ciphertext, nonce) = encrypt_data(&key, plaintext).unwrap();
        let decrypted = decrypt_data(&key, &ciphertext, &nonce).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_nonce_uniqueness() {
        let key = get_test_key();
        let plaintext = "test data";

        let mut nonces = std::collections::HashSet::new();

        // Generate 100 encryptions and verify all nonces are unique
        for _ in 0..100 {
            let (_, nonce) = encrypt_data(&key, plaintext).unwrap();
            assert!(
                nonces.insert(nonce.clone()),
                "Duplicate nonce detected!"
            );
        }

        assert_eq!(nonces.len(), 100);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = get_test_key();
        let mut key2 = get_test_key();
        key2[0] = 99; // Different key

        let plaintext = "secret message";
        let (ciphertext, nonce) = encrypt_data(&key1, plaintext).unwrap();

        // Attempting to decrypt with wrong key should fail
        let result = decrypt_data(&key2, &ciphertext, &nonce);
        assert!(result.is_err());
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let key = get_test_key();
        let plaintext = "important data";

        let (mut ciphertext, nonce) = encrypt_data(&key, plaintext).unwrap();

        // Tamper with the ciphertext
        let mut bytes = BASE64.decode(&ciphertext).unwrap();
        bytes[0] ^= 0xFF; // Flip bits
        ciphertext = BASE64.encode(&bytes);

        // Decryption should fail due to authentication tag mismatch
        let result = decrypt_data(&key, &ciphertext, &nonce);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_nonce_length() {
        let key = get_test_key();
        let ciphertext = BASE64.encode(b"fake ciphertext");
        let bad_nonce = BASE64.encode(b"short"); // Wrong length

        let result = decrypt_data(&key, &ciphertext, &bad_nonce);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid nonce length"));
    }

    #[test]
    fn test_empty_plaintext() {
        let key = get_test_key();
        let plaintext = "";

        let (ciphertext, nonce) = encrypt_data(&key, plaintext).unwrap();
        let decrypted = decrypt_data(&key, &ciphertext, &nonce).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_unicode_plaintext() {
        let key = get_test_key();
        let plaintext = "Hello 世界! 🔐 مرحبا";

        let (ciphertext, nonce) = encrypt_data(&key, plaintext).unwrap();
        let decrypted = decrypt_data(&key, &ciphertext, &nonce).unwrap();

        assert_eq!(plaintext, decrypted);
    }
}
