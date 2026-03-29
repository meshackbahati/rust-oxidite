//! Secure random generation

use rand::Rng;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use crate::{Result, SecurityError};

/// Generate random bytes
#[must_use]
pub fn random_bytes(length: usize) -> Vec<u8> {
    let mut rng = rand::rng();
    let mut bytes = vec![0u8; length];
    rng.fill(&mut bytes[..]);
    bytes
}

/// Generate a random hex string
#[must_use]
pub fn random_hex(length: usize) -> String {
    hex::encode(random_bytes(length))
}

/// Generate a secure token (URL-safe base64)
#[must_use]
pub fn secure_token(bytes: usize) -> String {
    URL_SAFE_NO_PAD.encode(random_bytes(bytes))
}

/// Generate a random alphanumeric string
#[must_use]
pub fn random_alphanumeric(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    
    (0..length)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Generate a random number in range
#[must_use]
pub fn random_range(min: u64, max: u64) -> u64 {
    try_random_range(min, max).unwrap_or(min)
}

/// Generate a random number in half-open range (`min..max`).
pub fn try_random_range(min: u64, max: u64) -> Result<u64> {
    if min > max {
        return Err(SecurityError::InvalidRange { min, max });
    }
    if min == max {
        return Ok(min);
    }
    let mut rng = rand::rng();
    Ok(rng.random_range(min..max))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_bytes() {
        let bytes = random_bytes(32);
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn test_random_hex() {
        let hex_str = random_hex(16);
        assert_eq!(hex_str.len(), 32); // 16 bytes = 32 hex chars
    }

    #[test]
    fn test_secure_token() {
        let token = secure_token(32);
        // Base64 encoded 32 bytes is about 43 chars
        assert!(token.len() > 40);
    }

    #[test]
    fn test_random_range() {
        for _ in 0..100 {
            let n = random_range(10, 20);
            assert!(n >= 10 && n < 20);
        }
    }

    #[test]
    fn test_try_random_range_validation() {
        assert!(try_random_range(5, 5).is_ok());
        assert!(try_random_range(6, 5).is_err());
    }
}
