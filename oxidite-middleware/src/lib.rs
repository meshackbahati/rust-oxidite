// Re-export commonly used tower-http middleware
pub use tower_http::compression::CompressionLayer;
pub use tower_http::cors::{CorsLayer, Any};

// Custom middleware
pub mod logger;
pub mod request_id;
pub mod security_headers;
pub mod csrf;
pub mod rate_limit;

pub use logger::LoggerLayer;
pub use request_id::{RequestIdLayer, RequestIdMiddleware};
pub use security_headers::{SecurityHeadersLayer, SecurityHeadersConfig, FrameOptions};
pub use csrf::{CsrfLayer, CsrfConfig};
pub use rate_limit::{RateLimitLayer, RateLimitConfig};

// Re-export ServiceBuilder for convenience
pub use tower::ServiceBuilder;
