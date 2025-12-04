use crate::{Storage, StoredFile, FileMetadata, Result, StorageError};
use async_trait::async_trait;
use bytes::Bytes;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Local filesystem storage
pub struct LocalStorage {
    root: PathBuf,
}

impl LocalStorage {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        
        // Create root directory if it doesn't exist
        std::fs::create_dir_all(&root)?;
        
        Ok(Self { root })
    }

    fn resolve_path(&self, path: &str) -> Result<PathBuf> {
        let full_path = self.root.join(path);
        
        // Prevent directory traversal
        if !full_path.starts_with(&self.root) {
            return Err(StorageError::InvalidPath(path.to_string()));
        }
        
        Ok(full_path)
    }
}

#[async_trait]
impl Storage for LocalStorage {
    async fn put(&self, path: &str, data: Bytes) -> Result<StoredFile> {
        let full_path = self.resolve_path(path)?;
        
        // Create parent directories
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        // Write file
        let mut file = fs::File::create(&full_path).await?;
        file.write_all(&data).await?;
        
        let size = data.len() as u64;
        let mime_type = mime_guess::from_path(&full_path)
            .first_or_octet_stream()
            .to_string();
        
        Ok(StoredFile {
            path: path.to_string(),
            size,
            mime_type,
            url: None,
        })
    }

    async fn get(&self, path: &str) -> Result<Bytes> {
        let full_path = self.resolve_path(path)?;
        
        if !full_path.exists() {
            return Err(StorageError::NotFound(path.to_string()));
        }
        
        let mut file = fs::File::open(&full_path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;
        
        Ok(Bytes::from(buffer))
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let full_path = self.resolve_path(path)?;
        
        if !full_path.exists() {
            return Err(StorageError::NotFound(path.to_string()));
        }
        
        fs::remove_file(&full_path).await?;
        Ok(())
    }

    async fn exists(&self, path: &str) -> Result<bool> {
        let full_path = self.resolve_path(path)?;
        Ok(full_path.exists())
    }

    async fn metadata(&self, path: &str) -> Result<FileMetadata> {
        let full_path = self.resolve_path(path)?;
        
        if !full_path.exists() {
            return Err(StorageError::NotFound(path.to_string()));
        }
        
        let metadata = fs::metadata(&full_path).await?;
        let mime_type = mime_guess::from_path(&full_path)
            .first_or_octet_stream()
            .to_string();
        
        Ok(FileMetadata {
            size: metadata.len(),
            mime_type,
            created_at: metadata.created().ok().map(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            }),
            modified_at: metadata.modified().ok().map(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            }),
        })
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let dir_path = self.resolve_path(prefix)?;
        
        if !dir_path.exists() {
            return Ok(Vec::new());
        }
        
        let mut entries = Vec::new();
        let mut read_dir = fs::read_dir(&dir_path).await?;
        
        while let Some(entry) = read_dir.next_entry().await? {
            if let Some(name) = entry.file_name().to_str() {
                entries.push(format!("{}/{}", prefix, name));
            }
        }
        
        Ok(entries)
    }
}
