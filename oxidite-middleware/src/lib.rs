/// Re-export commonly used tower-http middleware
pub use tower_http::compression::CompressionLayer;
pub use tower_http::cors::{CorsLayer, Any};

/// Request/response logging middleware
pub mod logger;
/// Request ID generation and propagation middleware
pub mod request_id;
/// Security headers injection middleware
pub mod security_headers;
/// CSRF protection middleware
pub mod csrf;
/// Rate limiting middleware
pub mod rate_limit;
/// Request timeout middleware
pub mod timeout;
/// Server identification header middleware
pub mod server_header;
/// Response caching middleware
pub mod cache;
/// Metrics collection middleware
pub mod metrics;

pub use logger::LoggerLayer;
pub use request_id::{RequestIdLayer, RequestIdMiddleware};
pub use security_headers::{SecurityHeadersLayer, SecurityHeadersConfig, FrameOptions};
pub use csrf::{CsrfLayer, CsrfConfig};
pub use rate_limit::{RateLimiter, RateLimitConfig};
pub use timeout::{TimeoutMiddleware, TimeoutError};
pub use server_header::add_server_header;
pub use cache::{CacheLayer, CacheMiddleware, CacheConfig, CacheLayerBuilder};
pub use metrics::{Metrics, MetricsLayer};

/// Re-export [`ServiceBuilder`] for convenient middleware stacking
pub use tower::ServiceBuilder;
/// Re-export the `tower` crate for service and layer traits
pub use tower;
/// Re-export the `tower-http` crate for HTTP-specific middleware
pub use tower_http;
