use async_trait::async_trait;
use redis::{Client, AsyncCommands};
use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::{Cache, Result};

/// Redis cache backend
pub struct RedisCache {
    client: Client,
    default_ttl: Option<Duration>,
}

impl RedisCache {
    pub fn new(url: &str) -> Result<Self> {
        let client = Client::open(url)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        
        Ok(Self {
            client,
            default_ttl: Some(Duration::from_secs(3600)),
        })
    }

    pub fn with_default_ttl(url: &str, ttl: Duration) -> Result<Self> {
        let client = Client::open(url)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        
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
        let mut conn = self.client.get_multiplexed_async_connection()
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            
        let result: Option<String> = conn.get(key)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            
        if let Some(data) = result {
            let value: T = serde_json::from_str(&data)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<()>
    where
        T: Serialize + Send + Sync,
    {
        let mut conn = self.client.get_multiplexed_async_connection()
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            
        let data = serde_json::to_string(value)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            
        let ttl = ttl.or(self.default_ttl);
        
        if let Some(duration) = ttl {
            let _: () = conn.set_ex(key, data, duration.as_secs() as u64)
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        } else {
            let _: () = conn.set(key, data)
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        }
        
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection()
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            
        let _: () = conn.del(key)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let mut conn = self.client.get_multiplexed_async_connection()
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            
        let exists: bool = conn.exists(key)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            
        Ok(exists)
    }

    async fn flush(&self) -> Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection()
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            
        let _: () = redis::cmd("FLUSHDB")
            .query_async(&mut conn)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            
        Ok(())
    }
}
