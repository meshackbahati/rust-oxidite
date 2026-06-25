use async_trait::async_trait;
use bytes::Bytes;
use std::path::{Component, Path};

/// Local filesystem storage backend
pub mod local;
/// File validation utilities
pub mod validation;

#[cfg(feature = "s3")]
/// S3-compatible storage backend
pub mod s3;

/// Re-export of [`LocalStorage`]
pub use local::LocalStorage;
/// Re-export of [`FileValidator`] and [`ValidationRules`]
pub use validation::{FileValidator, ValidationRules};

#[cfg(feature = "s3")]
/// Re-export of [`S3Storage`]
pub use s3::S3Storage;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;

/// Global storage manager instance
pub static STORAGE: Lazy<StorageManager> = Lazy::new(|| StorageManager::new());

/// The Storage Facade for static access
pub struct StorageFacade;

impl StorageFacade {
    /// Get a specific disk by name
    pub async fn disk(name: &str) -> Option<Arc<dyn Storage>> {
        STORAGE.disk(name).await
    }
    
    /// Get the default disk
    pub async fn default() -> Result<Arc<dyn Storage>> {
        STORAGE.default_disk().await.ok_or_else(|| StorageError::Other("Default disk not found".to_string()))
    }

    /// Store a file using the default disk
    pub async fn put(path: &str, data: Bytes) -> Result<StoredFile> {
        Self::default().await?.put(path, data).await
    }
    
    /// Retrieve a file using the default disk
    pub async fn get(path: &str) -> Result<Bytes> {
        Self::default().await?.get(path).await
    }
    
    /// Delete a file using the default disk
    pub async fn delete(path: &str) -> Result<()> {
        Self::default().await?.delete(path).await
    }
    
    /// Check if file exists on default disk
    pub async fn exists(path: &str) -> Result<bool> {
        Self::default().await?.exists(path).await
    }
}

/// Manager for configuring different storage backends
pub struct StorageManager {
    backends: RwLock<HashMap<String, Arc<dyn Storage>>>,
    default_disk: RwLock<String>,
}

impl StorageManager {
    /// Create a new empty storage manager with a default "local" disk
    pub fn new() -> Self {
        Self {
            backends: RwLock::new(HashMap::new()),
            default_disk: RwLock::new("local".to_string()),
        }
    }
    
    /// Register a named storage backend
    pub async fn add_disk(&self, name: &str, backend: Arc<dyn Storage>) {
        self.backends.write().await.insert(name.to_string(), backend);
    }
    
    /// Set the default disk by name
    pub async fn set_default(&self, name: &str) {
        *self.default_disk.write().await = name.to_string();
    }
    
    /// Get a storage backend by name
    pub async fn disk(&self, name: &str) -> Option<Arc<dyn Storage>> {
        self.backends.read().await.get(name).cloned()
    }
    
    /// Get the currently configured default storage backend
    pub async fn default_disk(&self) -> Option<Arc<dyn Storage>> {
        let name = self.default_disk.read().await.clone();
        self.disk(&name).await
    }
}

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
    /// Relative path of the stored file
    pub path: String,
    /// File size in bytes
    pub size: u64,
    /// MIME type of the file
    pub mime_type: String,
    /// Optional public URL for accessing the file
    pub url: Option<String>,
}

/// File metadata
#[derive(Debug, Clone)]
pub struct FileMetadata {
    /// File size in bytes
    pub size: u64,
    /// MIME type of the file
    pub mime_type: String,
    /// Unix timestamp of file creation, if available
    pub created_at: Option<u64>,
    /// Unix timestamp of last modification, if available
    pub modified_at: Option<u64>,
}

/// Storage errors
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    /// File not found
    #[error("File not found: {0}")]
    NotFound(String),
    
    /// I/O error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Invalid path
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    
    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// S3 error
    #[error("S3 error: {0}")]
    S3(String),
    
    /// Other storage error
    #[error("Storage error: {0}")]
    Other(String),
}

/// Storage result type alias
pub type Result<T> = std::result::Result<T, StorageError>;

pub(crate) fn validate_storage_path(path: &str) -> Result<()> {
    if path.is_empty() {
        return Err(StorageError::InvalidPath("path cannot be empty".to_string()));
    }
    if path.contains('\0') {
        return Err(StorageError::InvalidPath(
            "path cannot contain null bytes".to_string(),
        ));
    }
    if path.starts_with('/') || path.starts_with('\\') {
        return Err(StorageError::InvalidPath(
            "path must be relative to storage root".to_string(),
        ));
    }

    for component in Path::new(path).components() {
        match component {
            Component::Normal(_) => {}
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(StorageError::InvalidPath(path.to_string()));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_storage_path;

    #[test]
    fn validate_storage_path_rejects_parent_dir() {
        assert!(validate_storage_path("../secret").is_err());
        assert!(validate_storage_path("a/../b").is_err());
    }

    #[test]
    fn validate_storage_path_accepts_relative_path() {
        assert!(validate_storage_path("images/logo.png").is_ok());
    }
}
