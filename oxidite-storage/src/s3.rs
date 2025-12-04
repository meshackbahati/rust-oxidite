//! S3-compatible storage backend
//!
//! This module provides S3 storage support for the oxidite-storage crate.
//! Requires the `s3` feature to be enabled.

use crate::{Storage, StoredFile, FileMetadata, Result, StorageError};
use async_trait::async_trait;
use bytes::Bytes;
use aws_sdk_s3::Client;
use aws_sdk_s3::primitives::ByteStream;

/// S3 storage configuration
#[derive(Debug, Clone)]
pub struct S3Config {
    /// S3 bucket name
    pub bucket: String,
    /// AWS region
    pub region: String,
    /// Optional endpoint URL for S3-compatible services
    pub endpoint: Option<String>,
    /// Base URL for public access
    pub public_url: Option<String>,
}

impl S3Config {
    /// Create a new S3 config with the given bucket and region
    pub fn new(bucket: impl Into<String>, region: impl Into<String>) -> Self {
        Self {
            bucket: bucket.into(),
            region: region.into(),
            endpoint: None,
            public_url: None,
        }
    }

    /// Set a custom endpoint (for S3-compatible services like MinIO)
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    /// Set a public URL base for generating public URLs
    pub fn with_public_url(mut self, url: impl Into<String>) -> Self {
        self.public_url = Some(url.into());
        self
    }
}

/// S3 storage backend
pub struct S3Storage {
    client: Client,
    config: S3Config,
}

impl S3Storage {
    /// Create a new S3 storage with the given configuration
    pub async fn new(config: S3Config) -> Result<Self> {
        let mut aws_config_builder = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new(config.region.clone()));

        if let Some(endpoint) = &config.endpoint {
            aws_config_builder = aws_config_builder.endpoint_url(endpoint);
        }

        let aws_config = aws_config_builder.load().await;
        let client = Client::new(&aws_config);

        Ok(Self { client, config })
    }

    /// Generate a public URL for the file
    fn public_url(&self, path: &str) -> Option<String> {
        self.config.public_url.as_ref().map(|base| {
            format!("{}/{}/{}", base, self.config.bucket, path)
        })
    }
}

#[async_trait]
impl Storage for S3Storage {
    async fn put(&self, path: &str, data: Bytes) -> Result<StoredFile> {
        let size = data.len() as u64;
        
        // Guess content type from path
        let content_type = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();

        self.client
            .put_object()
            .bucket(&self.config.bucket)
            .key(path)
            .body(ByteStream::from(data))
            .content_type(&content_type)
            .send()
            .await
            .map_err(|e| StorageError::Other(format!("S3 put failed: {}", e)))?;

        Ok(StoredFile {
            path: path.to_string(),
            size,
            mime_type: content_type,
            url: self.public_url(path),
        })
    }

    async fn get(&self, path: &str) -> Result<Bytes> {
        let response = self.client
            .get_object()
            .bucket(&self.config.bucket)
            .key(path)
            .send()
            .await
            .map_err(|e| {
                if e.to_string().contains("NoSuchKey") {
                    StorageError::NotFound(path.to_string())
                } else {
                    StorageError::Other(format!("S3 get failed: {}", e))
                }
            })?;

        let data = response
            .body
            .collect()
            .await
            .map_err(|e| StorageError::Other(format!("Failed to read S3 response: {}", e)))?
            .into_bytes();

        Ok(data)
    }

    async fn delete(&self, path: &str) -> Result<()> {
        self.client
            .delete_object()
            .bucket(&self.config.bucket)
            .key(path)
            .send()
            .await
            .map_err(|e| StorageError::Other(format!("S3 delete failed: {}", e)))?;

        Ok(())
    }

    async fn exists(&self, path: &str) -> Result<bool> {
        let result = self.client
            .head_object()
            .bucket(&self.config.bucket)
            .key(path)
            .send()
            .await;

        match result {
            Ok(_) => Ok(true),
            Err(e) if e.to_string().contains("NotFound") => Ok(false),
            Err(e) => Err(StorageError::Other(format!("S3 exists check failed: {}", e))),
        }
    }

    async fn metadata(&self, path: &str) -> Result<FileMetadata> {
        let response = self.client
            .head_object()
            .bucket(&self.config.bucket)
            .key(path)
            .send()
            .await
            .map_err(|e| {
                if e.to_string().contains("NotFound") {
                    StorageError::NotFound(path.to_string())
                } else {
                    StorageError::Other(format!("S3 metadata failed: {}", e))
                }
            })?;

        let mime_type = response
            .content_type()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        Ok(FileMetadata {
            size: response.content_length().unwrap_or(0) as u64,
            mime_type,
            created_at: None, // S3 doesn't provide creation time
            modified_at: response.last_modified().map(|t| t.secs() as u64),
        })
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let response = self.client
            .list_objects_v2()
            .bucket(&self.config.bucket)
            .prefix(prefix)
            .send()
            .await
            .map_err(|e| StorageError::Other(format!("S3 list failed: {}", e)))?;

        let files = response
            .contents()
            .iter()
            .filter_map(|obj| obj.key().map(|k| k.to_string()))
            .collect();

        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_s3_config() {
        let config = S3Config::new("my-bucket", "us-east-1")
            .with_endpoint("http://localhost:9000")
            .with_public_url("https://cdn.example.com");

        assert_eq!(config.bucket, "my-bucket");
        assert_eq!(config.region, "us-east-1");
        assert_eq!(config.endpoint, Some("http://localhost:9000".to_string()));
        assert_eq!(config.public_url, Some("https://cdn.example.com".to_string()));
    }
}
