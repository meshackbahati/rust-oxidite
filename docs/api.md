# API Reference

This document provides a comprehensive reference for all Oxidite framework APIs.

## Core Module

### Router

The Router handles incoming HTTP requests and maps them to appropriate handlers.

```rust
use oxidite::prelude::*;

let mut router = Router::new();

// HTTP method routes
router.get(path, handler);
router.post(path, handler);
router.put(path, handler);
router.delete(path, handler);
router.patch(path, handler);
```

**Path Parameters**: Use `:name` syntax for named parameters
```rust
router.get("/users/:id", handler);  // Captures as "id"
router.get("/users/:user_id/posts/:post_id", handler);  // Captures "user_id" and "post_id"
```

**Wildcards**: Use `*` for wildcard matching
```rust
router.get("/files/*", handler);  // Matches /files/anything/here
```

### Server

The Server handles HTTP connections and dispatches requests to the router.

```rust
use oxidite::prelude::*;

let server = Server::new(router);
server.listen("127.0.0.1:3000".parse().unwrap()).await?;
```

### Request/Response Types

- `Request`: Alias for `OxiditeRequest`, wrapper around `hyper::Request`
- `Response`: Alias for `OxiditeResponse`, wrapper around `hyper::Response`
- `Result<T>`: Type alias for `std::result::Result<T, Error>`

## Extractors

Extractors implement `FromRequest` trait to extract data from requests.

### Json<T>

Extracts and deserializes JSON from request body.

```rust
use oxidite::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

async fn create_user(Json(payload): Json<CreateUser>) -> Result<Response> {
    // payload contains deserialized JSON
    Ok(response::json(serde_json::json!(payload)))
}
```

### Query<T>

Extracts and deserializes query parameters.

```rust
use oxidite::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    limit: Option<u32>,
}

async fn list_users(Query(params): Query<Pagination>) -> Result<Response> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(10);
    
    Ok(response::json(serde_json::json!({ "page": page, "limit": limit })))
}
```

### Path<T>

Extracts and deserializes path parameters.

```rust
use oxidite::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct UserId {
    id: u64,
}

async fn get_user(Path(params): Path<UserId>) -> Result<Response> {
    Ok(response::json(serde_json::json!({ "id": params.id })))
}
```

### State<T>

Extracts application state from request extensions.

```rust
use oxidite::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    db_url: String,
}

async fn handler(State(state): State<Arc<AppState>>) -> Result<Response> {
    Ok(response::json(serde_json::json!({ "db_url": state.db_url })))
}
```

## Error Handling

### Error Types

```rust
use oxidite::prelude::*;

enum Error {
    Server(String),           // Internal server error
    NotFound,                // Resource not found
    BadRequest(String),      // Bad request
    Unauthorized(String),    // Unauthorized access
    Hyper(hyper::Error),     // Hyper-specific error
    Io(std::io::Error),      // IO error
}

type Result<T> = std::result::Result<T, Error>;
```

### Error Responses

```rust
// In handlers, return appropriate errors
async fn handler(_req: Request) -> Result<Response> {
    // This will return a 404 Not Found
    if !resource_exists() {
        return Err(Error::NotFound);
    }
    
    // This will return a 400 Bad Request
    if !valid_input() {
        return Err(Error::BadRequest("Invalid input".to_string()));
    }
    
    // This will return a 500 Internal Server Error
    let data = some_operation_that_might_fail()?;
    
    Ok(response::json(serde_json::json!(data)))
}
```

## Response Utilities

### Creating Responses

```rust
use oxidite::response;

// JSON response
let json_resp = response::json(serde_json::json!({ "key": "value" }));

// HTML response
let html_resp = response::html("<h1>Hello</h1>");

// Text response
let text_resp = response::text("Plain text");
```

## Database Module

### Model Definition

```rust
use oxidite::prelude::*;
use oxidite::db::{Model, Database};
use serde::{Deserialize, Serialize};

#[derive(Model, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,  // Enables soft deletes
}
```

### Model Operations

```rust
// Create
let mut user = User {
    id: 0,  // Will be set by database
    name: "John".to_string(),
    email: "john@example.com".to_string(),
    created_at: 0,  // Set automatically
    updated_at: 0,  // Set automatically
    deleted_at: None,
};

user.create(&db).await?;

// Read
let user = User::find(&db, 1).await?;  // Returns Option<User>
let all_users = User::all(&db).await?;  // Returns Vec<User>

// Update
user.name = "Jane".to_string();
user.update(&db).await?;

// Delete (soft delete if deleted_at field exists)
user.delete(&db).await?;

// Force delete (hard delete)
user.force_delete(&db).await?;
```

### Database Connections

```rust
use oxidite::db::{DbPool, PoolOptions};

// Basic connection
let db = DbPool::connect("sqlite::memory:").await?;

// Connection with options
let options = PoolOptions {
    max_connections: 20,
    min_connections: 2,
    connect_timeout: std::time::Duration::from_secs(30),
    idle_timeout: Some(std::time::Duration::from_secs(600)),
};

let db = DbPool::connect_with_options("postgresql://...", options).await?;
```

## Authentication Module

### JWT Authentication

```rust
use oxidite::auth::{JwtManager, create_token, verify_token, Claims};

// Create JWT manager
let jwt_manager = JwtManager::new("your-secret-key".to_string());

// Create token
let claims = Claims {
    sub: "user-id".to_string(),
    exp: (chrono::Utc::now() + chrono::Duration::days(7)).timestamp() as u64,
    ..Default::default()
};

let token = create_token(&jwt_manager, claims)?;

// Verify token
let verified_claims = verify_token(&jwt_manager, &token)?;
```

### Password Hashing

```rust
use oxidite::auth::{hash_password, verify_password};

// Hash password
let password_hash = hash_password("user-password")?;

// Verify password
if verify_password("user-password", &password_hash)? {
    println!("Password is correct");
}
```

### API Keys

```rust
use oxidite::auth::ApiKey;

// Generate API key
let api_key = ApiKey::generate("user-id", Some("description"))?;
let key_value = api_key.key();

// Parse and verify API key
if let Some(parsed_key) = ApiKey::parse(key_value) {
    if parsed_key.verify("user-id") {
        println!("Valid API key");
    }
}
```

## Middleware

### Using Built-in Middleware

```rust
use oxidite::prelude::*;
use oxidite_middleware::{ServiceBuilder, LoggerLayer, CorsLayer, CompressionLayer};

let service = ServiceBuilder::new()
    .layer(LoggerLayer)                    // Request/response logging
    .layer(CorsLayer::permissive())       // Permissive CORS
    .layer(CompressionLayer::new())       // Response compression
    .service(router);
```

### Custom Middleware

```rust
use tower::{Layer, Service};
use futures::future::BoxFuture;
use http::{Request, Response};

// Custom middleware implementation
pub struct CustomMiddleware<S> {
    inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for CustomMiddleware<S>
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

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        // Pre-processing
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        
        Box::pin(async move {
            let response = inner.call(req).await?;
            // Post-processing
            Ok(response)
        })
    }
}
```

## Template Engine

### Template Creation and Rendering

```rust
use oxidite::template::{TemplateEngine, Context};

let mut engine = TemplateEngine::new();

// Add template from string
engine.add_template("welcome", "<h1>Hello, {{ name }}!</h1>")?;

// Load templates from directory
engine.load_dir("templates/")?;

// Create context
let mut context = Context::new();
context.set("name", "World");

// Render template
let html = engine.render("welcome", &context)?;
```

### Context Variables

```rust
use oxidite::template::Context;

let mut context = Context::new();

// Simple values
context.set("title", "My Page");
context.set("count", 42);

// Objects
context.set("user", serde_json::json!({
    "name": "John",
    "email": "john@example.com"
}));

// Arrays
context.set("items", vec!["apple", "banana", "cherry"]);
```

## CLI Commands

### Project Creation

```bash
# Create new project
oxidite new my-project

# Create specific project type
oxidite new my-api --project-type api
oxidite new my-fullstack --project-type fullstack
```

### Code Generation

```bash
# Generate model
oxidite make model User

# Generate controller
oxidite make controller UserController

# Generate middleware
oxidite make middleware AuthMiddleware
```

### Database Operations

```bash
# Create migration
oxidite migrate create create_users_table

# Run migrations
oxidite migrate run

# Revert migration
oxidite migrate revert

# Check status
oxidite migrate status
```

### Queue Management

```bash
# Start workers
oxidite queue work --workers 4

# List queue stats
oxidite queue list

# View dead letter queue
oxidite queue dlq

# Clear queue
oxidite queue clear
```

## Advanced Features

### API Versioning

```rust
// URL-based versioning
router.get("/api/v1/users", handler_v1);
router.get("/api/v2/users", handler_v2);

// Header-based versioning
async fn versioned_handler(req: Request) -> Result<Response> {
    let version = req.headers()
        .get("accept")
        .and_then(|hv| hv.to_str().ok())
        .and_then(|s| {
            if s.contains("version=2") { Some(2) } else { Some(1) }
        })
        .unwrap_or(1);
    
    match version {
        1 => handle_v1_api(req).await,
        2 => handle_v2_api(req).await,
        _ => Err(Error::BadRequest("Unsupported API version".to_string())),
    }
}
```

### Background Jobs

```rust
use oxidite::queue::{Job, Queue};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct EmailJob {
    recipient: String,
    subject: String,
    body: String,
}

impl Job for EmailJob {
    async fn execute(self) -> Result<(), Box<dyn std::error::Error>> {
        send_email(&self.recipient, &self.subject, &self.body).await?;
        Ok(())
    }
}

// Enqueue job
let queue = Queue::new_memory();
let job = EmailJob {
    recipient: "user@example.com".to_string(),
    subject: "Welcome!".to_string(),
    body: "Thank you for joining.".to_string(),
};

queue.enqueue(job).await?;
```

## Configuration

### Application State

```rust
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    db: DbPool,
    jwt_manager: JwtManager,
    template_engine: Arc<TemplateEngine>,
}

// Add to service
let state = Arc::new(AppState { /* ... */ });

let service = ServiceBuilder::new()
    .layer(AddExtensionLayer::new(state))
    .service(router);
```

### Environment Variables

Common environment variables:
- `DATABASE_URL`: Database connection string
- `PORT`: Server port (default: 3000)
- `HOST`: Server host (default: 127.0.0.1)
- `RUST_LOG`: Logging level (info, debug, warn, error)