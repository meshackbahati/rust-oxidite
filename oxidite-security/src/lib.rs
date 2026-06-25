//! # Oxidite Security
//!
//! Security utilities for the Oxidite framework including encryption,
//! hashing, sanitization, and secure random generation.

/// Cryptographic encryption/decryption utilities
pub mod crypto;
/// Hashing utilities
pub mod hash;
/// Secure random generation utilities
pub mod random;
/// HTML sanitization utilities
pub mod sanitize;

/// Re-export of [`encrypt`], [`decrypt`], [`AesKey`]
pub use crypto::{encrypt, decrypt, AesKey};
/// Re-export of [`sha256`], [`sha512`], [`hmac_sha256`], [`verify_hmac_sha256`]
pub use hash::{sha256, sha512, hmac_sha256, verify_hmac_sha256};
/// Re-export of [`random_bytes`], [`random_hex`], [`secure_token`], [`random_alphanumeric`], [`random_range`], [`try_random_range`]
pub use random::{random_bytes, random_hex, secure_token, random_alphanumeric, random_range, try_random_range};
/// Re-export of [`sanitize_html`], [`escape_html`], [`strip_tags`]
pub use sanitize::{sanitize_html, escape_html, strip_tags};

use thiserror::Error;

/// Security errors
#[derive(Error, Debug)]
pub enum SecurityError {
    /// Encryption failed
    #[error("Encryption failed: {0}")]
    EncryptionError(String),
    
    /// Decryption failed
    #[error("Decryption failed: {0}")]
    DecryptionError(String),
    
    /// Invalid key length
    #[error("Invalid key length")]
    InvalidKeyLength,
    
    /// Invalid data format
    #[error("Invalid data format")]
    InvalidFormat,

    /// Invalid random range
    #[error("Invalid random range: min ({min}) must be <= max ({max})")]
    InvalidRange { min: u64, max: u64 },
}

/// Security result type alias
pub type Result<T> = std::result::Result<T, SecurityError>;
