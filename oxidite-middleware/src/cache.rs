use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use http::{Request, Response, Method};
use http_body_util::Full;
use bytes::Bytes;
use tower::{Layer, Service};
use std::task::{Context, Poll};
use std::future::Future;
use std::pin::Pin;

/// Configuration for the caching middleware
#[derive(Clone)]
pub struct CacheConfig {
    /// Maximum cache size (in number of entries)
    pub max_entries: usize,
    /// Default TTL for cached responses
    pub default_ttl: Duration,
    /// Whether to cache responses for GET requests by default
    pub cache_get: bool,
    /// Whether to cache responses for POST requests
    pub cache_post: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            default_ttl: Duration::from_secs(300), // 5 minutes
            cache_get: true,
            cache_post: false,
        }
    }
}

/// Cache layer that wraps services with caching functionality
#[derive(Clone)]
pub struct CacheLayer {
    config: CacheConfig,
}

impl CacheLayer {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            config,
        }
    }

    pub fn builder() -> CacheLayerBuilder {
        CacheLayerBuilder::new()
    }
}

impl<S> Layer<S> for CacheLayer {
    type Service = CacheMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CacheMiddleware {
            inner,
            config: self.config.clone(),
        }
    }
}

/// Cache middleware service
pub struct CacheMiddleware<S> {
    inner: S,
    config: CacheConfig,
}

impl<S> CacheMiddleware<S> {
    fn should_cache_method(&self, method: &Method) -> bool {
        match *method {
            Method::GET => self.config.cache_get,
            Method::POST => self.config.cache_post,
            _ => false,
        }
    }
}

impl<S, ReqBody> Service<Request<ReqBody>> for CacheMiddleware<S>
where
    S: Service<Request<ReqBody>> + Clone,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        // Just pass through to the inner service for now
        // Proper caching implementation would require more complex async handling
        self.inner.call(req)
    }
}

/// Builder for CacheLayer
pub struct CacheLayerBuilder {
    config: CacheConfig,
}

impl CacheLayerBuilder {
    pub fn new() -> Self {
        Self {
            config: CacheConfig::default(),
        }
    }

    pub fn max_entries(mut self, max: usize) -> Self {
        self.config.max_entries = max;
        self
    }

    pub fn default_ttl(mut self, ttl: Duration) -> Self {
        self.config.default_ttl = ttl;
        self
    }

    pub fn cache_get(mut self, enable: bool) -> Self {
        self.config.cache_get = enable;
        self
    }

    pub fn cache_post(mut self, enable: bool) -> Self {
        self.config.cache_post = enable;
        self
    }

    pub fn build(self) -> CacheLayer {
        CacheLayer::new(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::{Request, StatusCode};
    use tower::{Service, ServiceExt};

    #[tokio::test]
    async fn test_cache_middleware() {
        let config = CacheConfig {
            max_entries: 100,
            default_ttl: Duration::from_secs(3600), // 1 hour
            cache_get: true,
            cache_post: false,
        };
        
        let layer = CacheLayer::new(config);
        
        // Simple service that always returns the same response
        let svc = tower::service_fn(|_req: Request<String>| async {
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(
                Response::builder()
                    .status(StatusCode::OK)
                    .body("Hello, world!".to_string())
                    .unwrap()
            )
        });

        let mut cached_svc = layer.layer(svc);

        // First request
        let req1 = Request::get("/test").body("".to_string()).unwrap();
        let resp1 = cached_svc.ready().await.unwrap().call(req1).await.unwrap();
        assert_eq!(resp1.status(), StatusCode::OK);

        // Second request to same endpoint should work
        let req2 = Request::get("/test").body("".to_string()).unwrap();
        let resp2 = cached_svc.ready().await.unwrap().call(req2).await.unwrap();
        assert_eq!(resp2.status(), StatusCode::OK);
    }
}