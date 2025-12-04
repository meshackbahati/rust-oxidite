//! Hashing utilities

use hmac::{Hmac, Mac};
use sha2::{Sha256, Sha512, Digest};

/// Compute SHA-256 hash of data
pub fn sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Compute SHA-512 hash of data
pub fn sha512(data: &[u8]) -> String {
    let mut hasher = Sha512::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Compute HMAC-SHA256
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> String {
    type HmacSha256 = Hmac<Sha256>;
    
    let mut mac = HmacSha256::new_from_slice(key)
        .expect("HMAC can take key of any size");
    mac.update(data);
    
    hex::encode(mac.finalize().into_bytes())
}

/// Verify HMAC-SHA256
pub fn verify_hmac_sha256(key: &[u8], data: &[u8], signature: &str) -> bool {
    let expected = hmac_sha256(key, data);
    constant_time_eq(expected.as_bytes(), signature.as_bytes())
}

/// Constant-time string comparison (prevents timing attacks)
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256() {
        let hash = sha256(b"hello");
        assert_eq!(hash.len(), 64); // 256 bits = 32 bytes = 64 hex chars
    }

    #[test]
    fn test_sha512() {
        let hash = sha512(b"hello");
        assert_eq!(hash.len(), 128); // 512 bits = 64 bytes = 128 hex chars
    }

    #[test]
    fn test_hmac() {
        let key = b"secret-key";
        let data = b"message";
        
        let sig = hmac_sha256(key, data);
        assert!(verify_hmac_sha256(key, data, &sig));
        assert!(!verify_hmac_sha256(key, b"other", &sig));
    }
}
