use async_trait::async_trait;
use redis::{Client, AsyncCommands};
use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::{validate_cache_key, validate_ttl, Cache, Result};

/// Redis cache backend
pub struct RedisCache {
    client: Client,
    default_ttl: Option<Duration>,
}

impl RedisCache {
    pub fn new(url: &str) -> Result<Self> {
        let client = Client::open(url)?;
        
        Ok(Self {
            client,
            default_ttl: Some(Duration::from_secs(3600)),
        })
    }

    pub fn with_default_ttl(url: &str, ttl: Duration) -> Result<Self> {
        validate_ttl(Some(ttl))?;
        let client = Client::open(url)?;
        
        Ok(Self {
            client,
            default_ttl: Some(ttl),
        })
    }
}

#[async_trait]
impl Cache for RedisCache {
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        validate_cache_key(key)?;
        let mut conn = self.client.get_multiplexed_async_connection().await?;
            
        let result: Option<String> = conn.get(key).await?;
            
        if let Some(data) = result {
            let value: T = serde_json::from_str(&data)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<()>
    where
        T: Serialize + Send + Sync,
    {
        validate_cache_key(key)?;
        validate_ttl(ttl)?;
        let mut conn = self.client.get_multiplexed_async_connection().await?;
            
        let data = serde_json::to_string(value)?;
            
        let ttl = ttl.or(self.default_ttl);
        
        if let Some(duration) = ttl {
            let seconds = duration.as_secs().max(1);
            let _: () = conn.set_ex(key, data, seconds).await?;
        } else {
            let _: () = conn.set(key, data).await?;
        }
        
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        validate_cache_key(key)?;
        let mut conn = self.client.get_multiplexed_async_connection().await?;
            
        let _: () = conn.del(key).await?;
            
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        validate_cache_key(key)?;
        let mut conn = self.client.get_multiplexed_async_connection().await?;
            
        let exists: bool = conn.exists(key).await?;
            
        Ok(exists)
    }

    async fn flush(&self) -> Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
            
        let _: () = redis::cmd("FLUSHDB")
            .query_async(&mut conn)
            .await?;
            
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::RedisCache;
    use std::time::Duration;

    #[test]
    fn rejects_zero_default_ttl() {
        let result = RedisCache::with_default_ttl("redis://127.0.0.1/", Duration::from_secs(0));
        assert!(result.is_err());
    }
}
