//! Optional query result caching with LRU eviction.
//!
//! The cache is opt-in and not used by default. Models can use it to cache
//! frequently executed queries, reducing database load for read-heavy workloads.
//!
//! # Example
//!
//! ```ignore
//! use oxidite_db::QueryCache;
//!
//! static CACHE: std::sync::LazyLock<QueryCache> = std::sync::LazyLock::new(|| {
//!     QueryCache::with_capacity(1000)
//! });
//!
//! // Cache a query result
//! let users = CACHE
//!     .get_or_insert("active_users", Duration::from_secs(60), || async {
//!         User::query().filter_eq("status", "active").fetch_all(&db).await
//!     })
//!     .await?;
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// A cached query result.
struct CacheEntry {
    /// When the entry was created
    created_at: Instant,
    /// How long the entry is valid for
    ttl: Duration,
    /// The cached data (serialized as JSON for simplicity)
    data: Vec<u8>,
}

impl CacheEntry {
    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

/// An LRU query cache for ORM results.
///
/// The cache stores serialized query results with TTL-based expiration.
/// It uses a simple LRU eviction strategy when the capacity is reached.
pub struct QueryCache {
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
    /// Maximum number of entries before eviction begins
    capacity: usize,
    /// Access order for LRU eviction (most recently used at the end)
    access_order: Arc<RwLock<Vec<String>>>,
}

impl QueryCache {
    /// Create a new cache with the default capacity of 1000 entries.
    pub fn new() -> Self {
        Self::with_capacity(1000)
    }

    /// Create a new cache with the specified maximum capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::with_capacity(capacity))),
            capacity,
            access_order: Arc::new(RwLock::new(Vec::with_capacity(capacity))),
        }
    }

    /// Get a cached result for the given key.
    /// Returns None if the key is not found or the entry has expired.
    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        let (exists, expired) = {
            let entries = self.entries.read().await;
            if let Some(entry) = entries.get(key) {
                (true, entry.is_expired())
            } else {
                (false, false)
            }
        };

        if !exists {
            return None;
        }
        if expired {
            return None;
        }

        // Update access order for LRU
        self.touch_access_order(key).await;

        // Re-read to get the data
        let entries = self.entries.read().await;
        entries.get(key).map(|e| e.data.clone())
    }

    /// Insert a result into the cache with the given TTL.
    pub async fn insert(&self, key: String, ttl: Duration, data: Vec<u8>) {
        let mut entries = self.entries.write().await;

        // Evict if at capacity and key is new
        if !entries.contains_key(&key) && entries.len() >= self.capacity {
            self.evict_lru(&mut entries).await;
        }

        entries.insert(
            key.clone(),
            CacheEntry {
                created_at: Instant::now(),
                ttl,
                data,
            },
        );

        drop(entries);
        self.touch_access_order(&key).await;
    }

    /// Remove a cached result.
    pub async fn remove(&self, key: &str) {
        let mut entries = self.entries.write().await;
        entries.remove(key);
        drop(entries);

        let mut order = self.access_order.write().await;
        order.retain(|k| k != key);
    }

    /// Clear all cached entries.
    pub async fn clear(&self) {
        self.entries.write().await.clear();
        self.access_order.write().await.clear();
    }

    /// Get the current number of cached entries.
    pub async fn len(&self) -> usize {
        self.entries.read().await.len()
    }

    /// Check if the cache is empty.
    pub async fn is_empty(&self) -> bool {
        self.entries.read().await.is_empty()
    }

    /// Invalidate all entries matching a prefix.
    /// Useful for invalidating cache entries for a specific model.
    pub async fn invalidate_prefix(&self, prefix: &str) {
        let mut entries = self.entries.write().await;
        let keys_to_remove: Vec<String> = entries
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();

        for key in keys_to_remove {
            entries.remove(&key);
        }

        drop(entries);
        let mut order = self.access_order.write().await;
        order.retain(|k| !k.starts_with(prefix));
    }

    /// Move key to the end of access order (most recently used).
    async fn touch_access_order(&self, key: &str) {
        let mut order = self.access_order.write().await;
        if let Some(pos) = order.iter().position(|k| k == key) {
            order.remove(pos);
        }
        order.push(key.to_string());
    }

    /// Evict the least recently used entry.
    async fn evict_lru(&self, entries: &mut HashMap<String, CacheEntry>) {
        let order = self.access_order.read().await;
        if let Some(oldest_key) = order.first().cloned() {
            drop(order);
            entries.remove(&oldest_key);
            let mut order = self.access_order.write().await;
            if !order.is_empty() {
                order.remove(0);
            }
        }
    }
}

impl Default for QueryCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cache_stores_and_retrieves() {
        let cache = QueryCache::new();
        cache
            .insert("test_key".to_string(), Duration::from_secs(60), vec![1, 2, 3])
            .await;
        let result = cache.get("test_key").await;
        assert_eq!(result, Some(vec![1, 2, 3]));
    }

    #[tokio::test]
    async fn cache_expires() {
        let cache = QueryCache::new();
        cache
            .insert("expire_key".to_string(), Duration::from_millis(10), vec![4, 5, 6])
            .await;

        // Should be present immediately
        assert!(cache.get("expire_key").await.is_some());

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert!(cache.get("expire_key").await.is_none());
    }

    #[tokio::test]
    async fn cache_remove_works() {
        let cache = QueryCache::new();
        cache
            .insert("remove_key".to_string(), Duration::from_secs(60), vec![7, 8, 9])
            .await;
        cache.remove("remove_key").await;
        assert!(cache.get("remove_key").await.is_none());
    }

    #[tokio::test]
    async fn cache_clear_works() {
        let cache = QueryCache::new();
        cache
            .insert("key1".to_string(), Duration::from_secs(60), vec![1])
            .await;
        cache
            .insert("key2".to_string(), Duration::from_secs(60), vec![2])
            .await;
        cache.clear().await;
        assert!(cache.is_empty().await);
    }

    #[tokio::test]
    async fn cache_invalidate_prefix() {
        let cache = QueryCache::new();
        cache
            .insert("user:1".to_string(), Duration::from_secs(60), vec![1])
            .await;
        cache
            .insert("user:2".to_string(), Duration::from_secs(60), vec![2])
            .await;
        cache
            .insert("post:1".to_string(), Duration::from_secs(60), vec![3])
            .await;

        cache.invalidate_prefix("user:").await;
        assert!(cache.get("user:1").await.is_none());
        assert!(cache.get("user:2").await.is_none());
        assert!(cache.get("post:1").await.is_some());
    }
}
