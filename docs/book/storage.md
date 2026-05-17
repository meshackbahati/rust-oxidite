# Unified File Storage API

Oxidite provides a unified `Storage` facade to abstract away file storage mechanisms. Whether you're using the local filesystem, AWS S3, DigitalOcean Spaces, Cloudinary, or ImageKit, the API remains the same.

## Setup & Configuration

Oxidite comes with the `StorageFacade` configured globally. By default, it uses the local disk.

```rust
use oxidite::storage::{StorageFacade as Storage, LocalStorage};

// Add disks to the storage manager on application boot
Storage::add_disk("local", Arc::new(LocalStorage::new("uploads/")?)).await;

// Set the default disk
Storage::set_default("local").await;
```

## Basic Usage

With the facade configured, you can perform storage operations asynchronously from anywhere in your application without needing to pass database or state references explicitly.

```rust
use oxidite::storage::StorageFacade as Storage;
use bytes::Bytes;

async fn upload_avatar(user_id: i64, data: Bytes) -> Result<String, oxidite::storage::StorageError> {
    let path = format!("avatars/{}.png", user_id);
    
    // Store file
    let file = Storage::put(&path, data).await?;
    
    // Get URL
    Ok(file.url.unwrap_or(path))
}

async fn download_file(path: &str) -> Result<Bytes, oxidite::storage::StorageError> {
    // Read file from default disk
    Storage::get(path).await
}

async fn remove_avatar(user_id: i64) -> Result<(), oxidite::storage::StorageError> {
    let path = format!("avatars/{}.png", user_id);
    
    if Storage::exists(&path).await? {
        Storage::delete(&path).await?;
    }
    
    Ok(())
}
```

## Cloud Storage Integrations

Oxidite's `Storage` trait makes it incredibly easy to plug in popular third-party services. Because you only interact with `StorageFacade`, your application logic never changes when migrating from Local to Cloud.

### AWS S3 / DigitalOcean Spaces

Oxidite includes built-in support for S3-compatible APIs.

```toml
# Cargo.toml
oxidite = { version = "2.2", features = ["storage-s3"] }
```

```rust
use oxidite::storage::{StorageFacade as Storage, S3Storage, S3Config};

let s3_config = S3Config {
    bucket: "my-bucket".to_string(),
    region: "us-east-1".to_string(),
    access_key: "ACCESS_KEY".to_string(),
    secret_key: "SECRET_KEY".to_string(),
    endpoint: None, // Set this for DigitalOcean, MinIO, or Cloudflare R2
};

Storage::add_disk("s3", Arc::new(S3Storage::new(s3_config))).await;

// Switch default disk to S3
Storage::set_default("s3").await;

// This will now upload directly to your S3 bucket!
Storage::put("documents/invoice.pdf", pdf_bytes).await?;
```

### Writing Custom Integrations (Cloudinary, ImageKit)

It's extremely simple to create a custom driver. Just implement the `Storage` trait, and your entire application can instantly start uploading to ImageKit, Cloudinary, or any other provider.

```rust
use async_trait::async_trait;
use oxidite::storage::{Storage, StoredFile, FileMetadata, Result};
use bytes::Bytes;

pub struct CloudinaryStorage {
    api_key: String,
    api_secret: String,
    cloud_name: String,
}

#[async_trait]
impl Storage for CloudinaryStorage {
    async fn put(&self, path: &str, data: Bytes) -> Result<StoredFile> {
        // Use reqwest to POST data to the Cloudinary API
        let upload_url = format!("https://api.cloudinary.com/v1_1/{}/upload", self.cloud_name);
        
        // ... (HTTP request logic) ...

        Ok(StoredFile {
            path: path.to_string(),
            size: data.len() as u64,
            mime_type: "image/jpeg".to_string(),
            url: Some("https://res.cloudinary.com/...".to_string()),
        })
    }
    
    async fn get(&self, path: &str) -> Result<Bytes> { /* ... */ }
    async fn delete(&self, path: &str) -> Result<()> { /* ... */ }
    async fn exists(&self, path: &str) -> Result<bool> { /* ... */ }
    async fn metadata(&self, path: &str) -> Result<FileMetadata> { /* ... */ }
    async fn list(&self, prefix: &str) -> Result<Vec<String>> { /* ... */ }
}
```

Once implemented, you register it with the facade:

```rust
Storage::add_disk("cloudinary", Arc::new(CloudinaryStorage { ... })).await;

// Upload explicitly using the Cloudinary disk
Storage::disk("cloudinary").await.unwrap().put("images/hero.jpg", img_bytes).await?;
```
