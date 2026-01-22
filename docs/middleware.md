# Middleware

Oxidite provides a powerful middleware system built on top of the `tower` ecosystem, allowing you to intercept and modify requests and responses.

## Overview

Middleware in Oxidite allows you to:

- Log requests and responses
- Handle cross-origin resource sharing (CORS)
- Implement security headers
- Add request IDs for tracing
- Implement rate limiting
- Handle authentication
- Add compression
- Implement timeouts

## Installation

Middleware components are included with the core framework:

```toml
[dependencies]
oxidite = "1.0"
```

## Using Middleware

### ServiceBuilder Pattern

Oxidite uses the `ServiceBuilder` from the `tower` ecosystem to compose middleware:

```rust
use oxidite::prelude::*;
use oxidite_middleware::{ServiceBuilder, LoggerLayer, CorsLayer};

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    router.get("/", handler);
    
    // Compose middleware using ServiceBuilder
    let service = ServiceBuilder::new()
        .layer(LoggerLayer)           // Log requests and responses
        .layer(CorsLayer::permissive()) // Allow cross-origin requests
        .service(router);
    
    Server::new(service)
        .listen("127.0.0.1:3000".parse().unwrap())
        .await
}
```

## Built-in Middleware

### LoggerLayer

Logs incoming requests and outgoing responses:

```rust
use oxidite_middleware::LoggerLayer;

let service = ServiceBuilder::new()
    .layer(LoggerLayer)
    .service(router);
```

The logger includes:
- Request method and path
- Response status code
- Response time
- Request size
- Response size

### CorsLayer

Handles Cross-Origin Resource Sharing:

```rust
use oxidite_middleware::{CorsLayer, Any};

// Permissive CORS (allows all origins, methods, and headers)
let cors_layer = CorsLayer::permissive();

// Custom CORS configuration
use tower_http::cors::{AllowOrigin, AllowMethods, AllowHeaders};

let cors_layer = CorsLayer::new()
    .allow_origin(AllowOrigin::predicate(|origin, _request_parts| {
        origin.as_bytes().starts_with(b"http://localhost:")
    }))
    .allow_methods(AllowMethods::any())
    .allow_headers(AllowHeaders::any());

let service = ServiceBuilder::new()
    .layer(cors_layer)
    .service(router);
```

### CompressionLayer

Adds response compression:

```rust
use oxidite_middleware::CompressionLayer;

let service = ServiceBuilder::new()
    .layer(CompressionLayer::new())
    .service(router);
```

### SecurityHeadersLayer

Adds security headers to responses:

```rust
use oxidite_middleware::{SecurityHeadersLayer, SecurityHeadersConfig, FrameOptions};

let security_config = SecurityHeadersConfig::new()
    .with_frame_options(FrameOptions::Deny)
    .with_content_security_policy("default-src 'self'")
    .with_xss_protection("1; mode=block");

let service = ServiceBuilder::new()
    .layer(SecurityHeadersLayer::new(security_config))
    .service(router);
```

### RequestIdLayer

Adds unique request IDs for tracing:

```rust
use oxidite_middleware::RequestIdLayer;

let service = ServiceBuilder::new()
    .layer(RequestIdLayer)
    .service(router);
```

### RateLimiter

Implements request rate limiting:

```rust
use oxidite_middleware::{RateLimiter, RateLimitConfig};

let rate_limit_config = RateLimitConfig::new()
    .with_requests_per_minute(100)
    .with_burst_size(10);

let service = ServiceBuilder::new()
    .layer(RateLimiter::new(rate_limit_config))
    .service(router);
```

### TimeoutMiddleware

Adds request timeout:

```rust
use oxidite_middleware::{TimeoutMiddleware, TimeoutError};
use std::time::Duration;

let timeout_service = TimeoutMiddleware::new(Duration::from_secs(30));

let service = ServiceBuilder::new()
    .layer(timeout_service)
    .service(router);
```

## Ordering of Middleware

The order of middleware matters as it affects the request/response flow:

```rust
let service = ServiceBuilder::new()
    .layer(RequestIdLayer)      // 1. Add request ID first
    .layer(LoggerLayer)         // 2. Log after request ID is set
    .layer(CorsLayer::permissive()) // 3. Handle CORS
    .layer(TimeoutMiddleware::new(Duration::from_secs(30))) // 4. Add timeout
    .service(router);           // 5. Finally reach the router
```

In the request flow, middleware executes in the order listed. In the response flow, middleware executes in reverse order.

## Custom Middleware

You can create custom middleware by implementing the `Layer` and `Service` traits:

```rust
use tower::{Layer, Service};
use tower::util::BoxService;
use tower::BoxError;
use futures::future::BoxFuture;
use std::task::{Context, Poll};
use http::{Request, Response};

// Define your middleware service
#[derive(Clone)]
pub struct CustomMiddleware<S> {
    inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for CustomMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<BoxError> + Send,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = S::Response;
    type Error = BoxError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        // Pre-processing: modify request if needed
        req.headers_mut().insert("x-custom-header", "custom-value".parse().unwrap());

        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            // Call the next service in the chain
            let response = inner.call(req).await.map_err(Into::into)?;

            // Post-processing: modify response if needed
            let mut response_clone = response.map(|body| body);
            response_clone.headers_mut().insert("x-powered-by", "Oxidite".parse().unwrap());

            Ok(response_clone)
        })
    }
}

// Define the layer that wraps services with your middleware
#[derive(Clone, Default)]
pub struct CustomMiddlewareLayer;

impl<S> Layer<S> for CustomMiddlewareLayer {
    type Service = CustomMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CustomMiddleware { inner }
    }
}

// Usage
let service = ServiceBuilder::new()
    .layer(CustomMiddlewareLayer)
    .service(router);
```

### Function-based Middleware

For simpler cases, you can use function-based middleware:

```rust
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use std::time::Instant;

// Using tower's trace layer for custom logging
let service = ServiceBuilder::new()
    .layer(TraceLayer::new_for_http())
    .service(router);
```

## Authentication Middleware

While not built-in, you can implement authentication middleware:

```rust
use tower::{Layer, Service};
use futures::future::BoxFuture;
use http::{Request, Response, StatusCode};
use std::task::{Context, Poll};

pub struct AuthMiddleware<S> {
    inner: S,
    secret: String,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for AuthMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Send,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        // Check for authentication
        if let Some(auth_header) = req.headers().get("authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = &auth_str[7..];
                    
                    // Validate token (simplified example)
                    if validate_token(token, &self.secret) {
                        // Token is valid, continue with request
                        let clone = self.inner.clone();
                        let mut inner = std::mem::replace(&mut self.inner, clone);
                        
                        return Box::pin(inner.call(req));
                    }
                }
            }
        }
        
        // Authentication failed, return 401
        Box::pin(async {
            let response = Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(oxidite_core::types::ResponseBody::empty())
                .unwrap();
            Ok(response)
        })
    }
}

fn validate_token(token: &str, secret: &str) -> bool {
    // In a real implementation, you'd verify the JWT token
    // This is just a placeholder
    token == secret
}
```

## Complete Middleware Example

Here's a complete example showing how to use multiple middleware components:

```rust
use oxidite::prelude::*;
use oxidite_middleware::{ServiceBuilder, LoggerLayer, CorsLayer, CompressionLayer};
use std::time::Duration;

async fn handler(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    Ok(response::json(serde_json::json!({
        "message": "Hello from Oxidite with middleware!",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

async fn protected_handler(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    Ok(response::json(serde_json::json!({
        "message": "This is a protected endpoint",
        "status": "authorized"
    })))
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting Oxidite server with middleware...");

    let mut router = Router::new();
    router.get("/", handler);
    router.get("/protected", protected_handler);
    
    // Apply middleware in the correct order
    let service = ServiceBuilder::new()
        .layer(LoggerLayer)                           // Log all requests
        .layer(CorsLayer::permissive())              // Allow cross-origin requests
        .layer(CompressionLayer::new())              // Compress responses
        .service(router);
    
    println!("📡 Server listening on http://127.0.0.1:3000");
    println!("📋 Middleware applied: Logger, CORS, Compression");
    println!("🔗 Try: curl http://localhost:3000/");
    println!("🔒 Try: curl http://localhost:3000/protected -H \"Authorization: Bearer token\"");
    
    Server::new(service)
        .listen("127.0.0.1:3000".parse().unwrap())
        .await
}
```

## Error Handling in Middleware

Middleware can catch and handle errors from downstream services:

```rust
use tower::{Layer, Service};
use futures::future::BoxFuture;
use http::{Request, Response, StatusCode};
use std::task::{Context, Poll};

pub struct ErrorHandlingMiddleware<S> {
    inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for ErrorHandlingMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>, Error = oxidite_core::Error> 
        + Clone 
        + Send 
        + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = Response<ResBody>;
    type Error = oxidite_core::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        
        Box::pin(async move {
            match inner.call(req).await {
                Ok(response) => Ok(response),
                Err(error) => {
                    // Convert framework errors to appropriate HTTP responses
                    let status_code = match &error {
                        oxidite_core::Error::NotFound => StatusCode::NOT_FOUND,
                        oxidite_core::Error::BadRequest(_) => StatusCode::BAD_REQUEST,
                        oxidite_core::Error::Unauthorized(_) => StatusCode::UNAUTHORIZED,
                        _ => StatusCode::INTERNAL_SERVER_ERROR,
                    };
                    
                    let response = Response::builder()
                        .status(status_code)
                        .header("content-type", "application/json")
                        .body(oxidite_core::types::ResponseBody::from(
                            serde_json::json!({
                                "error": status_code.as_str(),
                                "message": error.to_string()
                            }).to_string()
                        ))?;
                    
                    Ok(response)
                }
            }
        })
    }
}
```

## Best Practices

1. **Order Matters**: Place authentication/authorization middleware early in the chain
2. **Performance**: Be mindful of the overhead each middleware adds
3. **Error Handling**: Implement proper error handling in your middleware
4. **Logging**: Use structured logging for better observability
5. **Security**: Apply security headers and validations consistently
6. **Resource Cleanup**: Ensure middleware properly cleans up resources
7. **Testing**: Test middleware in isolation and integration