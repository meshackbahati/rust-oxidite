use crate::{validate_storage_path, Storage, StoredFile, FileMetadata, Result, StorageError};
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
        validate_storage_path(path)?;
        let full_path = self.root.join(path);

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
        
        if !fs::try_exists(&full_path).await? {
            return Err(StorageError::NotFound(path.to_string()));
        }
        
        let mut file = fs::File::open(&full_path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;
        
        Ok(Bytes::from(buffer))
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let full_path = self.resolve_path(path)?;
        
        if !fs::try_exists(&full_path).await? {
            return Err(StorageError::NotFound(path.to_string()));
        }
        
        fs::remove_file(&full_path).await?;
        Ok(())
    }

    async fn exists(&self, path: &str) -> Result<bool> {
        let full_path = self.resolve_path(path)?;
        Ok(fs::try_exists(full_path).await?)
    }

    async fn metadata(&self, path: &str) -> Result<FileMetadata> {
        let full_path = self.resolve_path(path)?;
        
        if !fs::try_exists(&full_path).await? {
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
        let dir_path = if prefix.is_empty() {
            self.root.clone()
        } else {
            self.resolve_path(prefix)?
        };
        
        if !fs::try_exists(&dir_path).await? {
            return Ok(Vec::new());
        }
        
        let mut entries = Vec::new();
        let mut read_dir = fs::read_dir(&dir_path).await?;
        
        while let Some(entry) = read_dir.next_entry().await? {
            let entry_path = entry.path();
            if let Ok(relative) = entry_path.strip_prefix(&self.root) {
                if let Some(name) = relative.to_str() {
                    entries.push(name.replace('\\', "/"));
                }
            }
        }
        
        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::{LocalStorage, Storage};
    use bytes::Bytes;

    #[tokio::test]
    async fn local_storage_rejects_parent_dir_paths() {
        let root = std::env::temp_dir().join(format!("oxidite-storage-{}", uuid::Uuid::new_v4()));
        let storage = LocalStorage::new(&root).expect("storage init");
        let err = storage.put("../escape.txt", Bytes::from_static(b"x")).await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn local_storage_list_returns_root_relative_paths() {
        let root = std::env::temp_dir().join(format!("oxidite-storage-{}", uuid::Uuid::new_v4()));
        let storage = LocalStorage::new(&root).expect("storage init");

        storage
            .put("images/logo.txt", Bytes::from_static(b"logo"))
            .await
            .expect("put");
        let files = storage.list("images").await.expect("list");
        assert_eq!(files, vec!["images/logo.txt".to_string()]);
    }
}
