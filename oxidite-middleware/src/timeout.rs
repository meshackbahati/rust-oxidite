//! Timeout middleware

use std::time::Duration;
use std::future::Future;
use tokio::time::timeout;

/// Timeout middleware
pub struct TimeoutMiddleware {
    duration: Duration,
}

impl TimeoutMiddleware {
    pub fn new(duration: Duration) -> Self {
        Self { duration }
    }
    
    pub fn seconds(seconds: u64) -> Self {
        Self::new(Duration::from_secs(seconds))
    }
    
    /// Wrap a future with timeout
    pub async fn wrap<F, T>(&self, future: F) -> Result<T, TimeoutError>
    where
        F: Future<Output = T>,
    {
        timeout(self.duration, future)
            .await
            .map_err(|_| TimeoutError::Elapsed)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TimeoutError {
    #[error("Request timeout elapsed")]
    Elapsed,
}

/// Request ID middleware for tracing
pub struct RequestIdMiddleware {
    header_name: String,
}

impl RequestIdMiddleware {
    pub fn new() -> Self {
        Self {
            header_name: "X-Request-ID".to_string(),
        }
    }
    
    pub fn with_header(mut self, header: String) -> Self {
        self.header_name = header;
        self
    }
    
    /// Generate a unique request ID
    pub fn generate_id() -> String {
        use uuid::Uuid;
        Uuid::new_v4().to_string()
    }
}

impl Default for RequestIdMiddleware {
    fn default() -> Self {
        Self::new()
    }
}
