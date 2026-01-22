# Appendix: Common Patterns and Recipes

This appendix contains common patterns, recipes, and solutions to frequently encountered scenarios when building applications with Oxidite.

## Request Data Extraction Patterns

### Extracting Multiple Types of Data from One Request

```rust
use oxidite::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct SearchParams {
    q: String,
    page: Option<u32>,
    limit: Option<u32>,
}

#[derive(Deserialize)]
struct SearchPayload {
    query: String,
    filters: Option<serde_json::Value>,
}

// Handler that extracts path, query, and JSON body
async fn advanced_search(
    Path(category): Path<String>,
    Query(params): Query<SearchParams>,
    Json(payload): Json<SearchPayload>,
) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "category": category,
        "search_params": params,
        "payload": payload,
        "message": "Advanced search executed"
    })))
}
```

### Working with Cookies

```rust
use oxidite::prelude::*;

async fn handle_cookies(cookies: Cookies) -> Result<Response> {
    let session_id = cookies.get("session_id");
    let theme = cookies.get("theme").unwrap_or("light");
    
    let mut response_data = serde_json::json!({
        "theme": theme,
        "has_session": session_id.is_some()
    });
    
    if let Some(sid) = session_id {
        response_data["session_id"] = serde_json::Value::String(sid.to_string());
    }
    
    Ok(Response::json(response_data))
}
```

## Response Patterns

### Conditional Responses

```rust
use oxidite::prelude::*;

async fn conditional_response(query: Query<serde_json::Value>) -> Result<Response> {
    let format = query.0.get("format")
        .and_then(|v| v.as_str())
        .unwrap_or("json");
    
    match format {
        "html" => Ok(Response::html("<h1>HTML Response</h1>".to_string())),
        "text" => Ok(Response::text("Text Response".to_string())),
        _ => Ok(Response::json(serde_json::json!({ "message": "JSON Response" }))),
    }
}
```

### Streaming Large Data

```rust
use oxidite::prelude::*;
use futures::stream::{self, StreamExt};
use http_body_util::StreamBody;
use hyper::body::Frame;
use bytes::Bytes;

async fn stream_large_data(_req: Request) -> Result<Response> {
    // Create a stream of data chunks
    let chunks = vec![
        "data-chunk-1",
        "data-chunk-2", 
        "data-chunk-3",
        "data-chunk-4",
    ];
    
    let stream = stream::iter(chunks.into_iter().map(|chunk| {
        Ok::<_, hyper::Error>(Frame::data(Bytes::from(chunk)))
    }));
    
    let body = StreamBody::new(stream);
    
    let response = hyper::Response::builder()
        .status(http::StatusCode::OK)
        .header(hyper::header::CONTENT_TYPE, "text/plain")
        .body(body.boxed())
        .map_err(|e| Error::Server(e.to_string()))?;
    
    Ok(response)
}
```

## Error Handling Patterns

### Custom Error Responses

```rust
use oxidite::prelude::*;

// Create a custom error response
fn custom_error_response(message: &str, status: u16) -> Response {
    Response::json(serde_json::json!({
        "error": message,
        "status": status
    }))
}

async fn custom_error_handler(_req: Request) -> Result<Response> {
    // Simulate a validation error
    let is_valid = false;
    
    if !is_valid {
        let error_response = custom_error_response("Validation failed", 422);
        return Ok(error_response);
    }
    
    Ok(Response::json(serde_json::json!({ "status": "success" })))
}
```

### Error Recovery Pattern

```rust
use oxidite::prelude::*;

async fn recoverable_operation(_req: Request) -> Result<Response> {
    // Attempt operation that might fail
    let result = some_risky_operation().await;
    
    match result {
        Ok(data) => Ok(Response::json(data)),
        Err(_) => {
            // Return a fallback response instead of error
            Ok(Response::json(serde_json::json!({
                "warning": "Using cached data",
                "data": get_cached_data()
            })))
        }
    }
}

async fn some_risky_operation() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    // Simulate an operation that might fail
    Err("Operation failed".into())
}

fn get_cached_data() -> serde_json::Value {
    serde_json::json!({ "cached": true, "data": "fallback" })
}
```

## Middleware Patterns

### Authentication Middleware

```rust
use oxidite::prelude::*;

async fn auth_middleware(req: Request, next: Next) -> Result<Response> {
    // Check for auth token in headers
    let auth_header = req.headers().get("authorization")
        .and_then(|hv| hv.to_str().ok());
    
    match auth_header {
        Some(token) if token.starts_with("Bearer ") => {
            // Validate token (simplified)
            let token = token.trim_start_matches("Bearer ");
            if validate_token(token) {
                // Add user info to request extensions
                let mut req = req;
                req.extensions_mut().insert(CurrentUser { id: 1, role: "user".to_string() });
                next.run(req).await
            } else {
                Err(Error::Unauthorized("Invalid token".to_string()))
            }
        }
        _ => Err(Error::Unauthorized("Missing or invalid token".to_string()))
    }
}

fn validate_token(_token: &str) -> bool {
    // In a real app, validate against your auth system
    true
}

#[derive(Clone)]
struct CurrentUser {
    id: u32,
    role: String,
}

async fn protected_route(user: CurrentUser) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "message": "Access granted",
        "user_id": user.id,
        "role": user.role
    })))
}
```

## Database Patterns

### Repository Pattern

```rust
use oxidite::prelude::*;
use serde::Deserialize;

// Simplified repository pattern
struct UserRepository;

impl UserRepository {
    async fn find_by_id(&self, id: u32) -> Result<Option<User>, Error> {
        // In a real app, query your database
        if id == 1 {
            Ok(Some(User {
                id,
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
            }))
        } else {
            Ok(None)
        }
    }
    
    async fn find_all(&self, limit: u32, offset: u32) -> Result<Vec<User>, Error> {
        // In a real app, query your database
        Ok(vec![
            User {
                id: 1,
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
            },
            User {
                id: 2,
                name: "Bob".to_string(),
                email: "bob@example.com".to_string(),
            },
        ])
    }
}

#[derive(serde::Serialize, Clone)]
struct User {
    id: u32,
    name: String,
    email: String,
}

async fn get_user(
    Path(user_id): Path<u32>,
    State(repo): State<std::sync::Arc<UserRepository>>
) -> Result<Response> {
    match repo.find_by_id(user_id).await? {
        Some(user) => Ok(Response::json(serde_json::json!(user))),
        None => Err(Error::NotFound),
    }
}

async fn get_users(
    Query(params): Query<PageParams>,
    State(repo): State<std::sync::Arc<UserRepository>>
) -> Result<Response> {
    let users = repo.find_all(params.limit.unwrap_or(10), params.offset.unwrap_or(0)).await?;
    Ok(Response::json(serde_json::json!(users)))
}

#[derive(Deserialize)]
struct PageParams {
    limit: Option<u32>,
    offset: Option<u32>,
}
```

## Configuration Patterns

### Environment-Based Configuration

```rust
use oxidite::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
struct AppConfig {
    database_url: String,
    server_port: u16,
    debug_mode: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL").unwrap_or("sqlite::memory:".to_string()),
            server_port: std::env::var("PORT")
                .unwrap_or("3000".to_string())
                .parse()
                .unwrap_or(3000),
            debug_mode: std::env::var("DEBUG").unwrap_or("false".to_string()) == "true",
        }
    }
}

async fn config_endpoint(State(config): State<AppConfig>) -> Result<Response> {
    Ok(Response::json(serde_json::json!(config)))
}
```

## Testing Patterns

### Unit Testing Handlers

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use oxidite_testing::TestClient;
    use tokio;

    #[tokio::test]
    async fn test_home_route() {
        let mut router = Router::new();
        router.get("/", home);
        
        let client = TestClient::new(router);
        let response = client.get("/").send().await;
        
        assert_eq!(response.status(), 200);
        let body = response.text().await;
        assert!(body.contains("Welcome to Oxidite!"));
    }
    
    #[tokio::test]
    async fn test_api_route() {
        let mut router = Router::new();
        router.get("/api/hello", api_hello);
        
        let client = TestClient::new(router);
        let response = client.get("/api/hello").send().await;
        
        assert_eq!(response.status(), 200);
        let json: serde_json::Value = response.json().await;
        assert_eq!(json["message"], "Hello from API");
    }
}
```

## Performance Patterns

### Caching with Memoization

```rust
use oxidite::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

// Simple in-memory cache
struct SimpleCache {
    data: Arc<Mutex<HashMap<String, (serde_json::Value, u64)>>>,
    ttl_seconds: u64,
}

impl SimpleCache {
    fn new(ttl_seconds: u64) -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
            ttl_seconds,
        }
    }
    
    fn get(&self, key: &str) -> Option<serde_json::Value> {
        let data = self.data.lock().unwrap();
        if let Some((value, timestamp)) = data.get(key) {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
                
            if current_time - timestamp < self.ttl_seconds {
                Some(value.clone())
            } else {
                // Entry expired
                drop(data);
                self.remove(key);
                None
            }
        } else {
            None
        }
    }
    
    fn set(&self, key: String, value: serde_json::Value) {
        let mut data = self.data.lock().unwrap();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        data.insert(key, (value, timestamp));
    }
    
    fn remove(&self, key: &str) {
        let mut data = self.data.lock().unwrap();
        data.remove(key);
    }
}

async fn cached_computation(
    State(cache): State<Arc<SimpleCache>>,
    Path(computation_type): Path<String>
) -> Result<Response> {
    let cache_key = format!("computation_{}", computation_type);
    
    // Check cache first
    if let Some(cached_result) = cache.get(&cache_key) {
        return Ok(Response::json(serde_json::json!({
            "result": cached_result,
            "from_cache": true
        })));
    }
    
    // Perform expensive computation
    let result = perform_expensive_computation(&computation_type).await;
    
    // Cache the result
    cache.set(cache_key, result.clone());
    
    Ok(Response::json(serde_json::json!({
        "result": result,
        "from_cache": false
    })))
}

async fn perform_expensive_computation(_input: &str) -> serde_json::Value {
    // Simulate expensive computation
    serde_json::json!({ "computed": true, "value": 42 })
}
```

## Common Anti-Patterns to Avoid

### Blocking Operations in Async Context

❌ Don't do this:
```rust
// BAD: This blocks the async runtime
async fn bad_handler(_req: Request) -> Result<Response> {
    let result = std::process::Command::new("slow_command").output().unwrap();
    Ok(Response::text(format!("{:?}", result)))
}
```

✅ Do this instead:
```rust
// GOOD: Use spawn_blocking for CPU-intensive operations
use tokio::task;

async fn good_handler(_req: Request) -> Result<Response> {
    let result = task::spawn_blocking(|| {
        std::process::Command::new("slow_command").output().unwrap()
    }).await.map_err(|e| Error::Server(e.to_string()))?;
    
    Ok(Response::text(format!("{:?}", result)))
}
```

### Improper Error Handling

❌ Don't do this:
```rust
// BAD: Converting errors to strings loses context
async fn bad_error_handling(_req: Request) -> Result<Response> {
    let data = some_operation().await.map_err(|e| Error::Server(e.to_string()))?;
    Ok(Response::json(data))
}
```

✅ Do this instead:
```rust
// GOOD: Preserve error types when possible
async fn good_error_handling(_req: Request) -> Result<Response> {
    let data = some_operation().await?;
    Ok(Response::json(data))
}

async fn some_operation() -> Result<serde_json::Value, Error> {
    // Return specific error types that map to appropriate HTTP statuses
    Err(Error::NotFound)
}
```

This appendix provides practical patterns for common scenarios you'll encounter when building Oxidite applications. Use these as starting points for your own implementations, adapting them to your specific needs.