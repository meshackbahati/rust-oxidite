use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
pub mod redis;
pub use crate::redis::RedisCache;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Cache trait
#[async_trait]
pub trait Cache: Send + Sync {
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send;

    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<()>
    where
        T: Serialize + Send + Sync;

    async fn delete(&self, key: &str) -> Result<()>;
    
    async fn exists(&self, key: &str) -> Result<bool>;
    
    async fn flush(&self) -> Result<()>;
}

/// Cache entry with expiration
#[derive(Clone)]
struct CacheEntry {
    data: Vec<u8>,
    expires_at: Option<Instant>,
}

impl CacheEntry {
    fn new(data: Vec<u8>, ttl: Option<Duration>) -> Self {
        let expires_at = ttl.map(|d| Instant::now() + d);
        Self { data, expires_at }
    }

    fn is_expired(&self) -> bool {
        self.expires_at.map(|t| Instant::now() > t).unwrap_or(false)
    }
}

/// In-memory cache implementation
pub struct MemoryCache {
    store: Arc<RwLock<HashMap<String, CacheEntry>>>,
    default_ttl: Option<Duration>,
}

impl MemoryCache {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            default_ttl: Some(Duration::from_secs(3600)),
        }
    }

    pub fn with_default_ttl(ttl: Duration) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            default_ttl: Some(ttl),
        }
    }

    /// Remember a value, executing the closure if not cached
    pub async fn remember<T, F, Fut>(&self, key: &str, ttl: Duration, f: F) -> Result<T>
    where
        T: Serialize + for<'de> Deserialize<'de> + Send + Sync,
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T>> + Send,
    {
        // Try to get from cache
        if let Some(value) = self.get::<T>(key).await? {
            return Ok(value);
        }

        // Execute closure and cache result
        let value = f().await?;
        self.set(key, &value, Some(ttl)).await?;
        Ok(value)
    }

    /// Clean expired entries
    async fn cleanup(&self) {
        let mut store = self.store.write().await;
        store.retain(|_, entry| !entry.is_expired());
    }
}

impl Default for MemoryCache {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Cache for MemoryCache {
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        let store = self.store.read().await;
        
        if let Some(entry) = store.get(key) {
            if entry.is_expired() {
                return Ok(None);
            }
            
            let value: T = serde_json::from_slice(&entry.data)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<()>
    where
        T: Serialize + Send + Sync,
    {
        let data = serde_json::to_vec(value)?;
        let ttl = ttl.or(self.default_ttl);
        let entry = CacheEntry::new(data, ttl);

        let mut store = self.store.write().await;
        store.insert(key.to_string(), entry);
        
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let mut store = self.store.write().await;
        store.remove(key);
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let store = self.store.read().await;
        Ok(store.get(key).map(|e| !e.is_expired()).unwrap_or(false))
    }

    async fn flush(&self) -> Result<()> {
        let mut store = self.store.write().await;
        store.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_set_and_get() {
        let cache = MemoryCache::new();
        
        cache.set("key1", &"value1", None).await.unwrap();
        let value: Option<String> = cache.get("key1").await.unwrap();
        
        assert_eq!(value, Some("value1".to_string()));
    }

    #[tokio::test]
    async fn test_expiration() {
        let cache = MemoryCache::new();
        
        cache.set("key1", &"value1", Some(Duration::from_millis(100))).await.unwrap();
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        let value: Option<String> = cache.get("key1").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_remember() {
        let cache = MemoryCache::new();
        let mut call_count = 0;

        let value = cache.remember("key1", Duration::from_secs(60), || async {
            call_count += 1;
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>("computed".to_string())
        }).await.unwrap();

        assert_eq!(value, "computed");
        assert_eq!(call_count, 1);

        // Second call should use cache
        let value2 = cache.remember("key1", Duration::from_secs(60), || async {
            call_count += 1;
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>("computed".to_string())
        }).await.unwrap();

        assert_eq!(value2, "computed");
        assert_eq!(call_count, 1); // Should not increment
    }
}
