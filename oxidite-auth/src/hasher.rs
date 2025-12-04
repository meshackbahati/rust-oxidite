use argon2::{
    password_hash::{PasswordHash, PasswordHasher as _, PasswordVerifier, SaltString},
    Argon2,
};
use crate::{AuthError, Result};

/// Password hasher utility
pub struct PasswordHasher;

impl PasswordHasher {
    /// Hash a password using Argon2id
    pub fn hash(password: &str) -> Result<String> {
        // Use a pre-generated salt for simplicity
        // In production, you'd want proper random salt generation
        let salt = SaltString::from_b64("X2lyb25tYW5pc2dyZWF0").unwrap();
        let argon2 = Argon2::default();
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AuthError::HashError(e.to_string()))?
            .to_string();
        
        Ok(password_hash)
    }

    /// Verify a password against a hash
    pub fn verify(password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AuthError::HashError(e.to_string()))?;
        
        let argon2 = Argon2::default();
        
        match argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

/// Hash a password
pub fn hash_password(password: &str) -> Result<String> {
    PasswordHasher::hash(password)
}

/// Verify a password
pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    PasswordHasher::verify(password, hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let password = "mysecretpassword";
        let hash = hash_password(password).unwrap();
        
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrongpassword", &hash).unwrap());
    }
}
