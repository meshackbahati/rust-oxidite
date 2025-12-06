use oxidite_db::Database;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;

/// Rate limit configuration
#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub requests_per_hour: Option<u32>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            requests_per_hour: Some(1000),
        }
    }
}

/// In-memory rate limiter with sliding window
pub struct RateLimiter {
    db: Option<Arc<dyn Database>>,
    config: RateLimitConfig,
    // In-memory cache: identifier -> (timestamp, count)
    cache: Arc<Mutex<HashMap<String, Vec<i64>>>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            db: None,
            config,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn with_db(config: RateLimitConfig, db: Arc<dyn Database>) -> Self {
        Self {
            db: Some(db),
            config,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Check if request is allowed (returns true if allowed)
    pub async fn check(&self, identifier: &str, endpoint: &str) -> bool {
        let now = chrono::Utc::now().timestamp();
        let minute_ago = now - 60;
        let hour_ago = now - 3600;
        
        // Use in-memory cache for performance
        let mut cache = self.cache.lock().await;
        let key = format!("{}:{}", identifier, endpoint);
        
        // Get timestamps for this identifier+endpoint
        let timestamps = cache.entry(key.clone()).or_insert_with(Vec::new);
        
        // Remove timestamps older than 1 hour
        timestamps.retain(|&ts| ts > hour_ago);
        
        // Count requests in last minute
        let minute_count = timestamps.iter().filter(|&&ts| ts > minute_ago).count() as u32;
        
        // Check minute limit
        if minute_count >= self.config.requests_per_minute {
            return false;
        }
        
        // Check hour limit if configured
        if let Some(hour_limit) = self.config.requests_per_hour {
            let hour_count = timestamps.len() as u32;
            if hour_count >= hour_limit {
                return false;
            }
        }
        
        // Request allowed - add timestamp
        timestamps.push(now);
        
        // Persist to database if configured (async, don't wait)
        if let Some(db) = &self.db {
            let db_clone = db.clone();
            let ident = identifier.to_string();
            let ep = endpoint.to_string();
            tokio::spawn(async move {
                let _ = Self::record_request(&*db_clone, &ident, &ep).await;
            });
        }
        
        true
    }
    
    /// Record request in database
    async fn record_request(db: &dyn Database, identifier: &str, endpoint: &str) -> oxidite_db::Result<()> {
        let now = chrono::Utc::now().timestamp();
        let window_start = (now / 60) * 60; // Round to minute
        
        // Try to increment existing record
        let update_query = format!(
            "UPDATE rate_limits 
             SET request_count = request_count + 1, updated_at = {}
             WHERE identifier = '{}' AND endpoint = '{}' AND window_start = {}",
            now, identifier, endpoint, window_start
        );
        
        let rows = db.execute(&update_query).await?;
        
        // If no existing record, insert new one
        if rows == 0 {
            let insert_query = format!(
                "INSERT INTO rate_limits (identifier, endpoint, request_count, window_start, created_at, updated_at)
                 VALUES ('{}', '{}', 1, {}, {}, {})",
                identifier, endpoint, window_start, now, now
            );
            db.execute(&insert_query).await?;
        }
        
        Ok(())
    }
    
    /// Get remaining requests for identifier
    pub async fn get_remaining(&self, identifier: &str, endpoint: &str) -> u32 {
        let now = chrono::Utc::now().timestamp();
        let minute_ago = now - 60;
        
        let cache = self.cache.lock().await;
        let key = format!("{}:{}", identifier, endpoint);
        
        if let Some(timestamps) = cache.get(&key) {
            let minute_count = timestamps.iter().filter(|&&ts| ts > minute_ago).count() as u32;
            self.config.requests_per_minute.saturating_sub(minute_count)
        } else {
            self.config.requests_per_minute
        }
    }
    
    /// Clean up old entries from cache (call periodically)
    pub async fn cleanup(&self) {
        let now = chrono::Utc::now().timestamp();
        let hour_ago = now - 3600;
        
        let mut cache = self.cache.lock().await;
        cache.retain(|_, timestamps| {
            timestamps.retain(|&ts| ts > hour_ago);
            !timestamps.is_empty()
        });
    }
}
