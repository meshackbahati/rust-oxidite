# Middleware

Middleware in Oxidite provides a way to modify requests and responses globally or for specific routes. This chapter covers how to create, use, and compose middleware.

## Overview

Middleware in Oxidite is a function that sits between the server and your route handlers. It can:
- Modify incoming requests
- Modify outgoing responses
- Perform authentication/validation
- Log requests and responses
- Handle cross-cutting concerns

## Basic Middleware

A basic middleware function has the signature `async fn(Request, Next) -> Result<Response>`:

```rust
use oxidite::prelude::*;

async fn basic_middleware(req: Request, next: Next) -> Result<Response> {
    // Process request before handler
    println!("Request received: {} {}", req.method(), req.uri());
    
    // Call the next handler in the chain
    let response = next.run(req).await?;
    
    // Process response after handler
    println!("Response sent with status: {}", response.status());
    
    Ok(response)
}
```

## Adding Middleware to Routes

You can add middleware to specific routes:

```rust
use oxidite::prelude::*;

async fn handler(_req: Request) -> Result<Response> {
    Ok(Response::text("Hello from protected route".to_string()))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    
    // Add middleware to a specific route
    router.get("/protected")
        .middleware(basic_middleware)
        .handler(handler);
    
    Server::new(router)
        .listen("127.0.0.1:3000".parse()?)
        .await
}
```

## Global Middleware

Add middleware to apply to all routes:

```rust
use oxidite::prelude::*;

async fn global_middleware(req: Request, next: Next) -> Result<Response> {
    println!("Global middleware: {}", req.uri());
    next.run(req).await
}

async fn home(_req: Request) -> Result<Response> {
    Ok(Response::text("Home page".to_string()))
}

async fn about(_req: Request) -> Result<Response> {
    Ok(Response::text("About page".to_string()))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    
    // Add global middleware
    router.middleware(global_middleware);
    
    router.get("/", home);
    router.get("/about", about);
    
    Server::new(router)
        .listen("127.0.0.1:3000".parse()?)
        .await
}
```

## Request/Response Modification

Middleware can modify both requests and responses:

```rust
use oxidite::prelude::*;

async fn request_modifier(req: Request, next: Next) -> Result<Response> {
    // Modify the request (e.g., add headers)
    let mut req = req;
    req.headers_mut().insert("X-Request-Processed", "true".parse().unwrap());
    
    let mut response = next.run(req).await?;
    
    // Modify the response
    response.headers_mut().insert("X-Response-Processed", "true".parse().unwrap());
    
    Ok(response)
}

async fn response_modifier(req: Request, next: Next) -> Result<Response> {
    let start_time = std::time::Instant::now();
    let mut response = next.run(req).await?;
    let duration = start_time.elapsed();
    
    // Add timing information to response
    response.headers_mut().insert(
        "X-Response-Time", 
        format!("{:.2?}", duration).parse().unwrap()
    );
    
    Ok(response)
}
```

## Authentication Middleware

A common use case is authentication:

```rust
use oxidite::prelude::*;

async fn auth_middleware(req: Request, next: Next) -> Result<Response> {
    // Check for authentication token
    let auth_header = req.headers()
        .get("authorization")
        .and_then(|hv| hv.to_str().ok());
    
    match auth_header {
        Some(token) if token.starts_with("Bearer ") => {
            let token = token.trim_start_matches("Bearer ");
            
            if validate_token(token).await {
                // Token is valid, continue with request
                next.run(req).await
            } else {
                // Invalid token
                Err(Error::Unauthorized("Invalid token".to_string()))
            }
        }
        _ => {
            // No valid token provided
            Err(Error::Unauthorized("Missing or invalid authorization header".to_string()))
        }
    }
}

async fn validate_token(_token: &str) -> bool {
    // In a real app, validate against your auth system
    _token == "valid-token"
}

async fn protected_route(_req: Request) -> Result<Response> {
    Ok(Response::json(serde_json::json!({ "message": "Access granted" })))
}
```

## Logging Middleware

A comprehensive logging middleware:

```rust
use oxidite::prelude::*;
use chrono::Utc;

async fn logging_middleware(req: Request, next: Next) -> Result<Response> {
    let start = std::time::Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();
    let user_agent = req.headers()
        .get("user-agent")
        .and_then(|hv| hv.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    
    println!(
        "[{}] {} {} - User-Agent: {}",
        Utc::now().format("%Y-%m-%d %H:%M:%S"),
        method,
        uri,
        user_agent
    );
    
    let response = next.run(req).await?;
    let duration = start.elapsed();
    
    println!(
        "[{}] {} {} - {} - {:.2?}",
        Utc::now().format("%Y-%m-%d %H:%M:%S"),
        method,
        uri,
        response.status(),
        duration
    );
    
    Ok(response)
}
```

## CORS Middleware

Cross-Origin Resource Sharing middleware:

```rust
use oxidite::prelude::*;

async fn cors_middleware(req: Request, next: Next) -> Result<Response> {
    // Handle preflight requests
    if req.method() == http::Method::OPTIONS {
        let mut response = Response::ok();
        set_cors_headers(response.headers_mut());
        return Ok(response);
    }
    
    let mut response = next.run(req).await?;
    set_cors_headers(response.headers_mut());
    
    Ok(response)
}

fn set_cors_headers(headers: &mut http::HeaderMap) {
    headers.insert(
        "Access-Control-Allow-Origin", 
        "*".parse().unwrap()
    );
    headers.insert(
        "Access-Control-Allow-Methods", 
        "GET, POST, PUT, DELETE, OPTIONS".parse().unwrap()
    );
    headers.insert(
        "Access-Control-Allow-Headers", 
        "Content-Type, Authorization".parse().unwrap()
    );
}
```

## Rate Limiting Middleware

Simple rate limiting middleware:

```rust
use oxidite::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests: u32,
    window_duration: Duration,
}

impl RateLimiter {
    fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window_duration: Duration::from_secs(window_seconds),
        }
    }
    
    fn is_allowed(&self, key: &str) -> bool {
        let mut requests = self.requests.lock().unwrap();
        
        let now = Instant::now();
        let window_start = now - self.window_duration;
        
        // Clean old requests
        if let Some(times) = requests.get_mut(key) {
            times.retain(|time| *time > window_start);
        }
        
        // Check if we're over the limit
        let current_count = requests
            .entry(key.to_string())
            .or_insert_with(Vec::new)
            .len();
        
        if current_count < self.max_requests as usize {
            requests.get_mut(key).unwrap().push(now);
            true
        } else {
            false
        }
    }
}

async fn rate_limit_middleware(
    req: Request, 
    next: Next
) -> Result<Response> {
    // Create a rate limiter (in a real app, this would be shared state)
    thread_local! {
        static RATE_LIMITER: RateLimiter = RateLimiter::new(10, 60); // 10 requests per minute
    }
    
    let client_ip = req.headers()
        .get("x-forwarded-for")
        .and_then(|hv| hv.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    
    if !RATE_LIMITER.with(|limiter| limiter.is_allowed(&client_ip)) {
        return Err(Error::RateLimited);
    }
    
    next.run(req).await
}
```

## Middleware Composition

You can compose multiple middleware functions:

```rust
use oxidite::prelude::*;

async fn middleware_a(req: Request, next: Next) -> Result<Response> {
    println!("A: Before");
    let result = next.run(req).await;
    println!("A: After");
    result
}

async fn middleware_b(req: Request, next: Next) -> Result<Response> {
    println!("B: Before");
    let result = next.run(req).await;
    println!("B: After");
    result
}

async fn middleware_c(req: Request, next: Next) -> Result<Response> {
    println!("C: Before");
    let result = next.run(req).await;
    println!("C: After");
    result
}

async fn handler(_req: Request) -> Result<Response> {
    println!("Handler executed");
    Ok(Response::text("Response".to_string()))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    
    // Middlewares are executed in the order they're added
    router.middleware(middleware_a);
    router.middleware(middleware_b);
    router.middleware(middleware_c);
    
    router.get("/", handler);
    
    Server::new(router)
        .listen("127.0.0.1:3000".parse()?)
        .await
}

// Output would be:
// A: Before
// B: Before
// C: Before
// Handler executed
// C: After
// B: After
// A: After
```

## Error Handling in Middleware

Middleware can catch and handle errors:

```rust
use oxidite::prelude::*;

async fn error_handling_middleware(req: Request, next: Next) -> Result<Response> {
    match next.run(req).await {
        Ok(response) => Ok(response),
        Err(Error::NotFound) => {
            Ok(Response::json(serde_json::json!({
                "error": "Resource not found",
                "code": 404
            })))
        }
        Err(Error::Unauthorized(msg)) => {
            Ok(Response::json(serde_json::json!({
                "error": "Unauthorized",
                "message": msg,
                "code": 401
            })))
        }
        Err(other_error) => Err(other_error),
    }
}
```

## Conditional Middleware

Apply middleware conditionally:

```rust
use oxidite::prelude::*;

async fn conditional_middleware(req: Request, next: Next) -> Result<Response> {
    // Only apply to certain paths
    if req.uri().path().starts_with("/api/") {
        println!("API request: {}", req.uri());
    }
    
    next.run(req).await
}

async fn api_handler(_req: Request) -> Result<Response> {
    Ok(Response::json(serde_json::json!({ "endpoint": "api" })))
}

async fn web_handler(_req: Request) -> Result<Response> {
    Ok(Response::text("Web page".to_string()))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    router.middleware(conditional_middleware);
    
    router.get("/api/data", api_handler);
    router.get("/web/page", web_handler);
    
    Server::new(router)
        .listen("127.0.0.1:3000".parse()?)
        .await
}
```

## Stateful Middleware

Middleware can use application state:

```rust
use oxidite::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    maintenance_mode: bool,
}

async fn stateful_middleware(
    req: Request,
    next: Next,
    State(state): State<Arc<AppState>>
) -> Result<Response> {
    if state.maintenance_mode && req.method() != http::Method::GET {
        return Err(Error::ServiceUnavailable("Maintenance mode".to_string()));
    }
    
    next.run(req).await
}

#[tokio::main]
async fn main() -> Result<()> {
    let app_state = Arc::new(AppState {
        maintenance_mode: false,
    });
    
    let mut router = Router::new();
    router.with_state(app_state);
    router.middleware(stateful_middleware);
    
    // ... add routes
    
    Server::new(router)
        .listen("127.0.0.1:3000".parse()?)
        .await
}
```

## Built-in Middleware

Oxidite provides several built-in middleware options:

```rust
use oxidite::prelude::*;
use oxidite_middleware::{Logger, RateLimiter, Cors};

// Logger middleware
async fn with_logger() -> Result<()> {
    let mut router = Router::new();
    
    // Add logging middleware
    router.middleware(Logger::new());
    
    // ... add routes
    
    Server::new(router)
        .listen("127.0.0.1:3000".parse()?)
        .await
}

// Rate limiting middleware
async fn with_rate_limit() -> Result<()> {
    let mut router = Router::new();
    
    // Add rate limiting middleware
    router.middleware(RateLimiter::new(100, std::time::Duration::from_secs(60))); // 100 requests per minute
    
    // ... add routes
    
    Server::new(router)
        .listen("127.0.0.1:3000".parse()?)
        .await
}
```

## Performance Considerations

1. **Order Matters**: Put fast-executing middleware first
2. **Avoid Heavy Computation**: Don't perform heavy operations in middleware
3. **Use Efficient Data Structures**: Use appropriate data structures for rate limiting, etc.
4. **Early Exit**: Return early when possible to avoid unnecessary processing

## Security Considerations

1. **Input Validation**: Validate inputs in middleware
2. **Rate Limiting**: Protect against abuse
3. **Authentication**: Verify credentials before processing
4. **Logging**: Log appropriately without exposing sensitive data

## Summary

Middleware in Oxidite is a powerful way to handle cross-cutting concerns:

- Use `async fn(Request, Next) -> Result<Response>` signature
- Apply globally with `router.middleware()` or to specific routes
- Modify requests before and responses after the handler
- Handle authentication, logging, CORS, rate limiting, etc.
- Compose multiple middleware functions
- Consider performance and security implications

Middleware provides a clean separation of concerns and keeps your route handlers focused on business logic.