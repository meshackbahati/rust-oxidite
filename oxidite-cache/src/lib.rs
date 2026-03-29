use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::RwLock;
pub mod redis;
pub use crate::redis::RedisCache;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("invalid cache key: {0}")]
    InvalidKey(String),
    #[error("invalid TTL: duration must be greater than zero")]
    InvalidTtl,
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("redis error: {0}")]
    Redis(#[from] ::redis::RedisError),
}

pub type Result<T> = std::result::Result<T, CacheError>;

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
    hits: Arc<AtomicU64>,
    misses: Arc<AtomicU64>,
    sets: Arc<AtomicU64>,
    deletes: Arc<AtomicU64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub sets: u64,
    pub deletes: u64,
}

/// Namespaced cache wrapper to prevent key collisions across domains.
pub struct NamespacedCache<C> {
    namespace: String,
    inner: C,
}

impl<C> NamespacedCache<C> {
    pub fn new(namespace: impl Into<String>, inner: C) -> Self {
        Self {
            namespace: namespace.into(),
            inner,
        }
    }

    fn key(&self, key: &str) -> String {
        format!("{}:{}", self.namespace, key)
    }
}

impl MemoryCache {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            default_ttl: Some(Duration::from_secs(3600)),
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
            sets: Arc::new(AtomicU64::new(0)),
            deletes: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn with_default_ttl(ttl: Duration) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            default_ttl: Some(ttl),
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
            sets: Arc::new(AtomicU64::new(0)),
            deletes: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Remember a value, executing the closure if not cached
    pub async fn remember<T, F, Fut>(&self, key: &str, ttl: Duration, f: F) -> Result<T>
    where
        T: Serialize + for<'de> Deserialize<'de> + Send + Sync,
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T>> + Send,
    {
        validate_cache_key(key)?;
        validate_ttl(Some(ttl))?;

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

    /// Snapshot in-memory cache operation counters.
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            sets: self.sets.load(Ordering::Relaxed),
            deletes: self.deletes.load(Ordering::Relaxed),
        }
    }

    /// Reset cache operation counters.
    pub fn reset_stats(&self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
        self.sets.store(0, Ordering::Relaxed);
        self.deletes.store(0, Ordering::Relaxed);
    }
}

impl Default for MemoryCache {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<C: Cache> Cache for NamespacedCache<C> {
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        self.inner.get(&self.key(key)).await
    }

    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<()>
    where
        T: Serialize + Send + Sync,
    {
        self.inner.set(&self.key(key), value, ttl).await
    }

    async fn delete(&self, key: &str) -> Result<()> {
        self.inner.delete(&self.key(key)).await
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        self.inner.exists(&self.key(key)).await
    }

    async fn flush(&self) -> Result<()> {
        self.inner.flush().await
    }
}

#[async_trait]
impl Cache for MemoryCache {
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        validate_cache_key(key)?;
        self.cleanup().await;
        let store = self.store.read().await;
        
        if let Some(entry) = store.get(key) {
            if entry.is_expired() {
                self.misses.fetch_add(1, Ordering::Relaxed);
                return Ok(None);
            }
            
            let value: T = serde_json::from_slice(&entry.data)?;
            self.hits.fetch_add(1, Ordering::Relaxed);
            Ok(Some(value))
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            Ok(None)
        }
    }

    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<()>
    where
        T: Serialize + Send + Sync,
    {
        validate_cache_key(key)?;
        validate_ttl(ttl)?;
        self.cleanup().await;
        let data = serde_json::to_vec(value)?;
        let ttl = ttl.or(self.default_ttl);
        let entry = CacheEntry::new(data, ttl);

        let mut store = self.store.write().await;
        store.insert(key.to_string(), entry);
        self.sets.fetch_add(1, Ordering::Relaxed);
        
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        validate_cache_key(key)?;
        let mut store = self.store.write().await;
        store.remove(key);
        self.deletes.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        validate_cache_key(key)?;
        self.cleanup().await;
        let store = self.store.read().await;
        Ok(store.get(key).map(|e| !e.is_expired()).unwrap_or(false))
    }

    async fn flush(&self) -> Result<()> {
        let mut store = self.store.write().await;
        store.clear();
        Ok(())
    }
}

pub(crate) fn validate_cache_key(key: &str) -> Result<()> {
    if key.trim().is_empty() {
        return Err(CacheError::InvalidKey(
            "key cannot be empty".to_string(),
        ));
    }
    if key.chars().any(char::is_control) {
        return Err(CacheError::InvalidKey(
            "key cannot contain control characters".to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn validate_ttl(ttl: Option<Duration>) -> Result<()> {
    if matches!(ttl, Some(d) if d.is_zero()) {
        return Err(CacheError::InvalidTtl);
    }
    Ok(())
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
            Ok::<_, CacheError>("computed".to_string())
        }).await.unwrap();

        assert_eq!(value, "computed");
        assert_eq!(call_count, 1);

        // Second call should use cache
        let value2 = cache.remember("key1", Duration::from_secs(60), || async {
            call_count += 1;
            Ok::<_, CacheError>("computed".to_string())
        }).await.unwrap();

        assert_eq!(value2, "computed");
        assert_eq!(call_count, 1); // Should not increment
    }

    #[tokio::test]
    async fn test_reject_empty_key() {
        let cache = MemoryCache::new();
        let result = cache.set("", &"value", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_reject_zero_ttl() {
        let cache = MemoryCache::new();
        let result = cache
            .set("k", &"value", Some(Duration::from_secs(0)))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_namespaced_cache_prefixes_keys() {
        let base = MemoryCache::new();
        let scoped = NamespacedCache::new("users", base);

        scoped.set("1", &"Alice", None).await.expect("set");
        let value: Option<String> = scoped.get("1").await.expect("get");
        assert_eq!(value.as_deref(), Some("Alice"));
    }

    #[tokio::test]
    async fn test_memory_cache_stats() {
        let cache = MemoryCache::new();
        cache.set("k", &"v", None).await.expect("set");
        let _v: Option<String> = cache.get("k").await.expect("get");
        let _missing: Option<String> = cache.get("missing").await.expect("get");
        cache.delete("k").await.expect("delete");

        let stats = cache.stats();
        assert_eq!(stats.sets, 1);
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.deletes, 1);
    }
}
