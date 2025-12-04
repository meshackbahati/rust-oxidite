//! # Oxidite Security
//!
//! Security utilities for the Oxidite framework including encryption,
//! hashing, sanitization, and secure random generation.

pub mod crypto;
pub mod hash;
pub mod random;
pub mod sanitize;

pub use crypto::{encrypt, decrypt, AesKey};
pub use hash::{sha256, sha512, hmac_sha256};
pub use random::{random_bytes, random_hex, secure_token};
pub use sanitize::{sanitize_html, escape_html};

use thiserror::Error;

/// Security errors
#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Encryption failed: {0}")]
    EncryptionError(String),
    
    #[error("Decryption failed: {0}")]
    DecryptionError(String),
    
    #[error("Invalid key length")]
    InvalidKeyLength,
    
    #[error("Invalid data format")]
    InvalidFormat,
}

pub type Result<T> = std::result::Result<T, SecurityError>;
