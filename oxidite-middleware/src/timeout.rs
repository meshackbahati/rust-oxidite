//! Timeout middleware

use std::time::Duration;
use std::future::Future;
use tokio::time::timeout;

/// Timeout middleware
pub struct TimeoutMiddleware {
    duration: Duration,
}

impl TimeoutMiddleware {
    /// Create a new `TimeoutMiddleware` with the given duration
    pub fn new(duration: Duration) -> Self {
        Self { duration }
    }
    
    /// Create a new `TimeoutMiddleware` with a duration in seconds
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

/// Errors that can occur during timeout operations
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
    /// Create a new `RequestIdMiddleware` with the default header name
    pub fn new() -> Self {
        Self {
            header_name: "X-Request-ID".to_string(),
        }
    }
    
    /// Set a custom header name for the request ID
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
