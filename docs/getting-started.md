# Getting Started with Oxidite v2

This guide will walk you through creating your first Oxidite application.

## Prerequisites

Before you begin, ensure you have:

- Rust 1.75 or higher
- Cargo package manager
- Git (for project templates)

## Installation

### Install the Oxidite CLI

The easiest way to start is by installing the Oxidite CLI tool:

```bash
# Install from the repository
cargo install --path oxidite-cli

# Or if you have the crate published
cargo install oxidite-cli
```

### Create a New Project

Use the CLI to create a new project:

```bash
oxidite new my-awesome-app
cd my-awesome-app
```

This creates a project with the following structure:

```
my-awesome-app/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── routes/
│   ├── controllers/
│   └── models/
├── templates/
├── public/
├── migrations/
└── README.md
```

## Basic Application Structure

Let's look at the main application file (`src/main.rs`):

```rust
use oxidite::prelude::*;

async fn hello_world(_req: Request) -> Result<Response> {
    Ok(response::text("Hello, Oxidite!"))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = Router::new();
    
    app.get("/", hello_world);
    
    Server::new(app)
        .listen("127.0.0.1:3000".parse().unwrap())
        .await
}
```

### Running Your Application

```bash
# Start the development server
oxidite dev

# Or build and run normally
cargo run
```

## Routing Basics

Oxidite provides flexible routing with support for different HTTP methods and path parameters.

### Basic Routes

```rust
use oxidite::prelude::*;

let mut router = Router::new();

// Different HTTP methods
router.get("/users", list_users);
router.post("/users", create_user);
router.put("/users/:id", update_user);
router.delete("/users/:id", delete_user);
```

### Path Parameters

Routes can include parameters using the `:name` syntax:

```rust
// This route captures the user ID
router.get("/users/:id", get_user);

// Multiple parameters
router.get("/users/:user_id/posts/:post_id", get_post);

// Wildcards (matches any path after /files/)
router.get("/files/*", serve_file);
```

## Request Handling

Handlers are async functions that receive a `Request` (alias for `OxiditeRequest`) and return a `Result<Response>` (alias for `OxiditeResponse`).

### Simple Handler

```rust
use oxidite::prelude::*;

async fn simple_handler(_req: Request) -> Result<Response> {
    Ok(response::text("Hello from a handler!"))
}
```

### Handler with JSON Response

```rust
use oxidite::prelude::*;

async fn api_handler(_req: Request) -> Result<Response> {
    let data = serde_json::json!({
        "message": "Hello from API",
        "status": "success"
    });
    
    Ok(response::json(data))
}
```

## Using Extractors

Extractors make it easy to extract data from requests in a type-safe way.

### JSON Extractor

```rust
use oxidite::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

async fn create_user(Json(payload): Json<CreateUserRequest>) -> Result<Response> {
    // payload contains the deserialized JSON data
    let user = serde_json::json!({
        "id": 1,
        "name": payload.name,
        "email": payload.email
    });
    
    Ok(response::json(user))
}
```

### Query Parameters Extractor

```rust
use oxidite::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct ListQuery {
    page: Option<u32>,
    limit: Option<u32>,
}

async fn list_items(Query(params): Query<ListQuery>) -> Result<Response> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(10);
    
    Ok(response::json(serde_json::json!({
        "page": page,
        "limit": limit,
        "items": []
    })))
}
```

### Path Parameters Extractor

```rust
use oxidite::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct UserId {
    id: u64,
}

async fn get_user(Path(params): Path<UserId>) -> Result<OxiditeResponse> {
    Ok(response::json(serde_json::json!({
        "id": params.id,
        "name": "User Name"
    })))
}
```

## Database Integration

To use the database features, enable the database feature in your `Cargo.toml`:

```toml
[dependencies]
oxidite = { version = "1.0", features = ["database"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

### Define a Model

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

### Database Operations

```rust
use oxidite::prelude::*;
use oxidite::db::Database;

// Create a user
async fn create_user_handler(
    db: &impl Database,
    name: String,
    email: String
) -> Result<User> {
    let mut user = User {
        id: 0,
        name,
        email,
        created_at: chrono::Utc::now().timestamp(),
        updated_at: chrono::Utc::now().timestamp(),
        deleted_at: None,
    };
    
    user.create(db).await?;
    Ok(user)
}

// Find a user by ID
async fn find_user_handler(db: &impl Database, id: i64) -> Result<Option<User>> {
    User::find(db, id).await
}

// Get all users
async fn all_users_handler(db: &impl Database) -> Result<Vec<User>> {
    User::all(db).await
}
```

## Adding Middleware

Oxidite integrates with the `tower` ecosystem for middleware. Use `ServiceBuilder` to compose middleware:

```rust
use oxidite::prelude::*;
use oxidite_middleware::{ServiceBuilder, LoggerLayer, CorsLayer};

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    router.get("/", handler);
    
    // Add middleware
    let service = ServiceBuilder::new()
        .layer(LoggerLayer)              // Log requests/responses
        .layer(CorsLayer::permissive()) // Allow cross-origin requests
        .service(router);
    
    Server::new(service)
        .listen("127.0.0.1:3000".parse().unwrap())
        .await
}
```

## Complete Example: User API

Here's a complete example of a user management API:

```rust
use oxidite::prelude::*;
use oxidite::db::{Model, Database, DbPool};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Model, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone)]
struct AppState {
    db: DbPool,
}

#[derive(Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[derive(Deserialize)]
struct UserIdParam {
    id: i64,
}

async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserRequest>
) -> Result<OxiditeResponse> {
    let mut user = User {
        id: 0,
        name: payload.name,
        email: payload.email,
        created_at: chrono::Utc::now().timestamp(),
        updated_at: chrono::Utc::now().timestamp(),
    };
    
    user.create(&state.db).await
        .map_err(|e| Error::Server(format!("Database error: {}", e)))?;
    
    Ok(response::json(serde_json::json!(user)))
}

async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(params): Path<UserIdParam>
) -> Result<OxiditeResponse> {
    match User::find(&state.db, params.id).await {
        Ok(Some(user)) => Ok(response::json(serde_json::json!(user))),
        Ok(None) => Err(Error::NotFound),
        Err(e) => Err(Error::Server(format!("Database error: {}", e))),
    }
}

async fn list_users(
    State(state): State<Arc<AppState>>
) -> Result<OxiditeResponse> {
    let users = User::all(&state.db).await
        .map_err(|e| Error::Server(format!("Database error: {}", e)))?;
    
    Ok(response::json(serde_json::json!({
        "users": users,
        "total": users.len()
    })))
}

async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(params): Path<UserIdParam>,
    Json(payload): Json<CreateUserRequest>
) -> Result<OxiditeResponse> {
    let mut user = match User::find(&state.db, params.id).await {
        Ok(Some(u)) => u,
        Ok(None) => return Err(Error::NotFound),
        Err(e) => return Err(Error::Server(format!("Database error: {}", e))),
    };
    
    user.name = payload.name;
    user.email = payload.email;
    user.updated_at = chrono::Utc::now().timestamp();
    
    user.update(&state.db).await
        .map_err(|e| Error::Server(format!("Database error: {}", e)))?;
    
    Ok(response::json(serde_json::json!(user)))
}

async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(params): Path<UserIdParam>
) -> Result<OxiditeResponse> {
    let user = match User::find(&state.db, params.id).await {
        Ok(Some(u)) => u,
        Ok(None) => return Err(Error::NotFound),
        Err(e) => return Err(Error::Server(format!("Database error: {}", e))),
    };
    
    user.delete(&state.db).await
        .map_err(|e| Error::Server(format!("Database error: {}", e)))?;
    
    Ok(response::json(serde_json::json!({ "message": "User deleted" })))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize database
    let db = DbPool::connect("sqlite::memory:").await?;
    
    // Prepare application state
    let state = Arc::new(AppState { db });
    
    // Create router with routes
    let mut router = Router::new();
    router.get("/users", list_users);
    router.post("/users", create_user);
    router.get("/users/:id", get_user);
    router.put("/users/:id", update_user);
    router.delete("/users/:id", delete_user);
    
    // Add middleware
    let service = ServiceBuilder::new()
        .layer(LoggerLayer)
        .layer(AddExtensionLayer::new(state))
        .service(router);
    
    println!("🚀 User API server starting on http://127.0.0.1:3000");
    println!("📋 Available endpoints:");
    println!("   GET    /users          - List all users");
    println!("   POST   /users          - Create a new user");
    println!("   GET    /users/:id      - Get user by ID");
    println!("   PUT    /users/:id      - Update user");
    println!("   DELETE /users/:id      - Delete user");
    
    Server::new(service)
        .listen("127.0.0.1:3000".parse().unwrap())
        .await
}
```

## Testing Your Application

To run tests:

```bash
# Run all tests
cargo test

# Run tests with specific features
cargo test --features database

# Run tests with output
cargo test -- --nocapture
```

## Development Workflow

A typical development workflow:

1. **Create project**: `oxidite new my-app`
2. **Generate models**: `oxidite make model User`
3. **Create migrations**: `oxidite migrate create create_users_table`
4. **Run migrations**: `oxidite migrate run`
5. **Start development server**: `oxidite dev`
6. **Iterate on code**

## Next Steps

- [Database Guide](database.md) - Learn more about the ORM
- [Authentication Guide](authentication.md) - Add user authentication
- [Templating Guide](templating.md) - Server-side rendering
- [Middleware Guide](middleware.md) - Add functionality with middleware
- [API Reference](api-reference.md) - Complete API documentation
- [Advanced Features](advanced-features.md) - Explore advanced functionality