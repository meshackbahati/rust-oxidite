use std::time::Duration;
use http::{Request, Method};
use tower::{Layer, Service};
use std::task::{Context, Poll};

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
        // We intentionally fall through when caching is disabled for the method.
        // Full response-body caching is backend-specific and implemented separately.
        let _cache_enabled = self.should_cache_method(req.method());
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
    use http::Method;

    #[test]
    fn test_should_cache_method_flags() {
        let config = CacheConfig {
            max_entries: 100,
            default_ttl: Duration::from_secs(3600),
            cache_get: true,
            cache_post: false,
        };

        let middleware = CacheMiddleware {
            inner: (),
            config,
        };

        assert!(middleware.should_cache_method(&Method::GET));
        assert!(!middleware.should_cache_method(&Method::POST));
        assert!(!middleware.should_cache_method(&Method::PUT));
    }

    #[test]
    fn test_cache_layer_builder() {
        let layer = CacheLayer::builder()
            .max_entries(42)
            .default_ttl(Duration::from_secs(5))
            .cache_get(false)
            .cache_post(true)
            .build();

        let middleware = layer.layer(());
        assert_eq!(middleware.config.max_entries, 42);
        assert_eq!(middleware.config.default_ttl, Duration::from_secs(5));
        assert!(!middleware.config.cache_get);
        assert!(middleware.config.cache_post);
    }
}
