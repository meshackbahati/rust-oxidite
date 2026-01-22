# oxidite-middleware

HTTP middleware for Oxidite (CORS, logging, compression, rate limiting).

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/oxidite-middleware.svg)](https://crates.io/crates/oxidite-middleware)
[![Docs.rs](https://docs.rs/oxidite-middleware/badge.svg)](https://docs.rs/oxidite-middleware)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

</div>

## Overview

`oxidite-middleware` provides a collection of commonly needed HTTP middleware for the Oxidite web framework. Built on top of the `tower` ecosystem, it offers composable, reusable components that can be easily integrated into your application's request/response pipeline.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
oxidite-middleware = "0.1"
```

## Features

- **CORS handling** - Configurable Cross-Origin Resource Sharing policies
- **Request logging** - Detailed request/response logging with customizable formats
- **Gzip compression** - Automatic response compression for bandwidth savings
- **Rate limiting** - Sliding window rate limiting to prevent abuse
- **CSRF protection** - Cross-Site Request Forgery prevention
- **Security headers** - Automatic addition of security headers
- **Timeout handling** - Request timeout protection
- **Request ID tracking** - Unique request IDs for tracing
- **Easy composition** - Seamless integration with the Tower service stack

## Usage

### Basic Setup

Use the `ServiceBuilder` to compose multiple middleware layers:

```rust
use oxidite::prelude::*;
use oxidite_middleware::{ServiceBuilder, LoggerLayer, CorsLayer, CompressionLayer};

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    router.get("/", handler);
    
    // Compose multiple middleware layers
    let app = ServiceBuilder::new()
        .layer(LoggerLayer::new())
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .service(router);
    
    Server::new(app)
        .listen("127.0.0.1:3000".parse().unwrap())
        .await
}
```

### CORS Middleware

Configure Cross-Origin Resource Sharing policies:

```rust
use oxidite_middleware::{CorsLayer, Origin, Method, Header};
use std::time::Duration;

let cors = CorsLayer::new()
    .allow_origin(Origin::exact("https://example.com"))
    .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers(vec![Header::CONTENT_TYPE, Header::AUTHORIZATION])
    .max_age(Duration::from_secs(86400)); // Cache preflight for 1 day

let app = ServiceBuilder::new()
    .layer(cors)
    .service(router);
```

### Logging Middleware

Add detailed request/response logging:

```rust
use oxidite_middleware::LoggerLayer;

let app = ServiceBuilder::new()
    .layer(LoggerLayer::new())
    .service(router);

// Or customize the log format
let custom_logger = LoggerLayer::custom(|request, response| {
    println!(
        "{} {} -> {} ({})",
        request.method(),
        request.uri().path(),
        response.status(),
        response.headers().get("content-length").unwrap_or(&"0".parse().unwrap())
    );
});
```

### Compression Middleware

Enable automatic response compression:

```rust
use oxidite_middleware::CompressionLayer;

let app = ServiceBuilder::new()
    .layer(CompressionLayer::new())
    .service(router);

// The middleware automatically compresses responses based on Accept-Encoding header
```

### Rate Limiting Middleware

Protect your application from abuse with rate limiting:

```rust
use oxidite_middleware::RateLimitLayer;
use std::time::Duration;

// Allow 100 requests per minute per IP address
let rate_limiter = RateLimitLayer::new(100, Duration::from_secs(60));

let app = ServiceBuilder::new()
    .layer(rate_limiter)
    .service(router);

// For more granular control
let custom_limiter = RateLimitLayer::builder()
    .limit(50)                    // Max requests
    .duration(Duration::from_secs(60))  // Time window
    .build();
```

### Security Headers

Add important security headers to responses:

```rust
use oxidite_middleware::SecurityHeadersLayer;

let security = SecurityHeadersLayer::new()
    .hsts(true)
    .x_frame_options("DENY")
    .x_content_type_options("nosniff")
    .x_xss_protection("1; mode=block");

let app = ServiceBuilder::new()
    .layer(security)
    .service(router);
```

### CSRF Protection

Protect against Cross-Site Request Forgery attacks:

```rust
use oxidite_middleware::CsrfLayer;

let csrf = CsrfLayer::new()
    .cookie_name("csrf-token")
    .header_name("x-csrf-token");

let app = ServiceBuilder::new()
    .layer(csrf)
    .service(router);
```

### Timeout Middleware

Prevent hanging requests with timeouts:

```rust
use oxidite_middleware::TimeoutLayer;
use std::time::Duration;

let timeout = TimeoutLayer::new(Duration::from_secs(30));

let app = ServiceBuilder::new()
    .layer(timeout)
    .service(router);
```

### Request ID Middleware

Track requests across your application with unique IDs:

```rust
use oxidite_middleware::RequestIdLayer;

let request_id = RequestIdLayer::new();

let app = ServiceBuilder::new()
    .layer(request_id)
    .service(router);

// Request IDs can be accessed in handlers via request extensions
```

### Complete Example

Combine multiple middleware for a production-ready application:

```rust
use oxidite::prelude::*;
use oxidite_middleware::{
    ServiceBuilder, LoggerLayer, CorsLayer, CompressionLayer, 
    RateLimitLayer, SecurityHeadersLayer, TimeoutLayer
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    router.get("/", home_handler);
    router.post("/api/users", create_user_handler);
    
    // Build a comprehensive middleware stack
    let app = ServiceBuilder::new()
        // Security
        .layer(SecurityHeadersLayer::new())
        .layer(CorsLayer::permissive())
        
        // Performance
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        
        // Protection
        .layer(RateLimitLayer::new(100, Duration::from_secs(60)))
        
        // Observability
        .layer(LoggerLayer::new())
        
        .service(router);
    
    Server::new(app)
        .listen("127.0.0.1:3000".parse().unwrap())
        .await
}
```

## Custom Middleware

Create your own middleware by implementing the Tower `Layer` and `Service` traits:

```rust
use tower::{Layer, Service};
use std::task::{Context, Poll};
use http::{Request, Response};

#[derive(Clone)]
pub struct CustomMiddleware<S> {
    inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for CustomMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
        // Pre-processing
        println!("Processing request: {} {}", request.method(), request.uri().path());
        
        // Call the inner service
        self.inner.call(request)
    }
}

#[derive(Clone)]
pub struct CustomMiddlewareLayer;

impl<S> Layer<S> for CustomMiddlewareLayer {
    type Service = CustomMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CustomMiddleware { inner }
    }
}
```

## Integration with Oxidite

All middleware is designed to work seamlessly with Oxidite's architecture:

```rust
use oxidite::prelude::*;
use oxidite_middleware::{ServiceBuilder, LoggerLayer};

async fn handler(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    Ok(response::text("Hello, World!"))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    router.get("/", handler);
    
    // Wrap the router with middleware
    let service = ServiceBuilder::new()
        .layer(LoggerLayer::new())
        .service(router);
    
    Server::new(service)
        .listen("127.0.0.1:3000".parse().unwrap())
        .await
}
```

## License

MIT
