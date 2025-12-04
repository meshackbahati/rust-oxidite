use oxidite_core::{OxiditeRequest, OxiditeResponse, Error as CoreError};
use tower::{Service, Layer};
use std::task::{Context, Poll};
use std::future::Future;
use std::pin::Pin;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// Rate limit configuration
#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    pub requests_per_window: usize,
    pub window_secs: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_window: 100,
            window_secs: 60,
        }
    }
}

struct RateLimitEntry {
    count: usize,
    window_start: u64,
}

/// Rate limiting middleware
#[derive(Clone)]
pub struct RateLimitMiddleware<S> {
    inner: S,
    config: RateLimitConfig,
    store: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
}

impl<S> RateLimitMiddleware<S> {
    pub fn new(inner: S, config: RateLimitConfig) -> Self {
        Self {
            inner,
            config,
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn get_client_ip(req: &OxiditeRequest) -> String {
        // Try to get real IP from X-Forwarded-For or X-Real-IP
        req.headers()
            .get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
            .or_else(|| {
                req.headers()
                    .get("x-real-ip")
                    .and_then(|h| h.to_str().ok())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "unknown".to_string())
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

impl<S>Service<OxiditeRequest> for RateLimitMiddleware<S>
where
    S: Service<OxiditeRequest, Response = OxiditeResponse, Error = CoreError> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: OxiditeRequest) -> Self::Future {
        let client_ip = Self::get_client_ip(&req);
        let config = self.config.clone();
        let store = self.store.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let now = Self::current_timestamp();
            let mut store = store.write().await;

            let entry = store.entry(client_ip.clone()).or_insert(RateLimitEntry {
                count: 0,
                window_start: now,
            });

            // Reset window if expired
            if now - entry.window_start >= config.window_secs {
                entry.count = 0;
                entry.window_start = now;
            }

            // Check rate limit
            if entry.count >= config.requests_per_window {
                drop(store); // Release lock
                return Err(CoreError::BadRequest("Rate limit exceeded".to_string()));
            }

            entry.count += 1;
            drop(store); // Release lock before calling inner service

            inner.call(req).await
        })
    }
}

/// Layer for rate limiting middleware
#[derive(Clone)]
pub struct RateLimitLayer {
    config: RateLimitConfig,
}

impl RateLimitLayer {
    pub fn new(config: RateLimitConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self {
            config: RateLimitConfig::default(),
        }
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitMiddleware::new(inner, self.config.clone())
    }
}
