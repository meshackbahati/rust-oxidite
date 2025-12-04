//! Cryptographic utilities

use crate::{Result, SecurityError};
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use aes_gcm::aead::rand_core::RngCore;
use base64::{Engine as _, engine::general_purpose::STANDARD};

/// AES-256-GCM encryption key
pub struct AesKey {
    cipher: Aes256Gcm,
}

impl AesKey {
    /// Create a new key from bytes (must be 32 bytes)
    pub fn from_bytes(key: &[u8]) -> Result<Self> {
        if key.len() != 32 {
            return Err(SecurityError::InvalidKeyLength);
        }
        
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| SecurityError::EncryptionError(e.to_string()))?;
        
        Ok(Self { cipher })
    }

    /// Generate a new random key
    pub fn generate() -> Self {
        let cipher = Aes256Gcm::new(&Aes256Gcm::generate_key(&mut OsRng));
        Self { cipher }
    }

    /// Encrypt data
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self.cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| SecurityError::EncryptionError(e.to_string()))?;

        // Prepend nonce to ciphertext
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend(ciphertext);
        
        Ok(result)
    }

    /// Decrypt data
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        if ciphertext.len() < 12 {
            return Err(SecurityError::InvalidFormat);
        }

        let (nonce_bytes, encrypted) = ciphertext.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        self.cipher
            .decrypt(nonce, encrypted)
            .map_err(|e| SecurityError::DecryptionError(e.to_string()))
    }
}

/// Encrypt data with a key (returns base64-encoded result)
pub fn encrypt(key: &[u8], plaintext: &[u8]) -> Result<String> {
    let aes_key = AesKey::from_bytes(key)?;
    let encrypted = aes_key.encrypt(plaintext)?;
    Ok(STANDARD.encode(&encrypted))
}

/// Decrypt base64-encoded data with a key
pub fn decrypt(key: &[u8], ciphertext: &str) -> Result<Vec<u8>> {
    let aes_key = AesKey::from_bytes(key)?;
    let encrypted = STANDARD.decode(ciphertext)
        .map_err(|_| SecurityError::InvalidFormat)?;
    aes_key.decrypt(&encrypted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = AesKey::generate();
        let plaintext = b"Hello, World!";
        
        let encrypted = key.encrypt(plaintext).unwrap();
        let decrypted = key.decrypt(&encrypted).unwrap();
        
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_convenience_functions() {
        let key = [0u8; 32]; // 32-byte key
        let plaintext = b"Secret message";
        
        let encrypted = encrypt(&key, plaintext).unwrap();
        let decrypted = decrypt(&key, &encrypted).unwrap();
        
        assert_eq!(decrypted, plaintext);
    }
}
