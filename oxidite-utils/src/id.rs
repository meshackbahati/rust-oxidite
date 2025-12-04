//! ID generation utilities

use uuid::Uuid;
use rand::Rng;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

/// Generate a UUID v4
pub fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
}

/// Generate a shorter unique ID (22 chars, URL-safe base64 encoded UUID)
pub fn generate_id() -> String {
    let uuid = Uuid::new_v4();
    URL_SAFE_NO_PAD.encode(uuid.as_bytes())
}

/// Generate a short ID with custom length (alphanumeric)
pub fn generate_short_id(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    
    (0..length)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Generate a numeric ID with custom length
pub fn generate_numeric_id(length: usize) -> String {
    const CHARSET: &[u8] = b"0123456789";
    let mut rng = rand::rng();
    
    (0..length)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_uuid() {
        let id = generate_uuid();
        assert_eq!(id.len(), 36); // UUID format with dashes
        assert!(Uuid::parse_str(&id).is_ok());
    }

    #[test]
    fn test_generate_id() {
        let id = generate_id();
        assert_eq!(id.len(), 22); // Base64 encoded 16 bytes
    }

    #[test]
    fn test_generate_short_id() {
        let id = generate_short_id(8);
        assert_eq!(id.len(), 8);
        assert!(id.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_generate_numeric_id() {
        let id = generate_numeric_id(6);
        assert_eq!(id.len(), 6);
        assert!(id.chars().all(|c| c.is_numeric()));
    }
}
