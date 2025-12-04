use async_trait::async_trait;
use bytes::Bytes;
use std::path::PathBuf;

pub mod local;
pub mod validation;

#[cfg(feature = "s3")]
pub mod s3;

pub use local::LocalStorage;
pub use validation::{FileValidator, ValidationRules};

#[cfg(feature = "s3")]
pub use s3::S3Storage;

/// Storage trait for file operations
#[async_trait]
pub trait Storage: Send + Sync {
    /// Store a file
    async fn put(&self, path: &str, data: Bytes) -> Result<StoredFile>;
    
    /// Retrieve a file
    async fn get(&self, path: &str) -> Result<Bytes>;
    
    /// Delete a file
    async fn delete(&self, path: &str) -> Result<()>;
    
    /// Check if file exists
    async fn exists(&self, path: &str) -> Result<bool>;
    
    /// Get file metadata
    async fn metadata(&self, path: &str) -> Result<FileMetadata>;
    
    /// List files in directory
    async fn list(&self, prefix: &str) -> Result<Vec<String>>;
}

/// Stored file information
#[derive(Debug, Clone)]
pub struct StoredFile {
    pub path: String,
    pub size: u64,
    pub mime_type: String,
    pub url: Option<String>,
}

/// File metadata
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub size: u64,
    pub mime_type: String,
    pub created_at: Option<u64>,
    pub modified_at: Option<u64>,
}

/// Storage errors
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("File not found: {0}")]
    NotFound(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Storage error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;
