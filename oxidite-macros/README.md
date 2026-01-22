# oxidite-macros

Procedural macros for the Oxidite web framework.

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/oxidite-macros.svg)](https://crates.io/crates/oxidite-macros)
[![Docs.rs](https://docs.rs/oxidite-macros/badge.svg)](https://docs.rs/oxidite-macros)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

</div>

## Overview

`oxidite-macros` provides a collection of procedural macros that enhance the developer experience in Oxidite applications. These macros reduce boilerplate code, enable compile-time optimizations, and provide ergonomic abstractions for common patterns in web development.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
oxidite-macros = "2.0"
```

## Available Macros

### `#[derive(Model)]`

Derives a database model with automatic CRUD operations, relationships, and validation.

```rust
use oxidite_macros::Model;
use serde::{Deserialize, Serialize};

#[derive(Model, Serialize, Deserialize)]
#[model(table = "users")]
pub struct User {
    #[model(primary_key)]
    pub id: i32,
    #[model(unique, not_null)]
    pub email: String,
    #[model(not_null)]
    pub name: String,
    #[model(default = "now")]
    pub created_at: String,
}

// This generates:
// - Database operations (find, save, delete, etc.)
// - Relationship methods
// - Validation logic
// - Migration generation
```

### `#[handler]`

Creates a route handler with automatic extractor inference and error handling.

```rust
use oxidite_macros::handler;
use oxidite::prelude::*;

#[handler(get, "/users")]
async fn list_users(Json(filters): Json<UserFilters>) -> Result<Response> {
    let users = get_users(filters).await?;
    Ok(response::json(serde_json::json!(users)))
}

#[handler(post, "/users")]
async fn create_user(Json(input): Json<CreateUserInput>) -> Result<Response> {
    let user = create_user(input).await?;
    Ok(response::json(serde_json::json!(user)))
}
```

### `#[controller]`

Groups related handlers into a controller with automatic route registration.

```rust
use oxidite_macros::controller;
use oxidite::prelude::*;

#[controller("/api/v1/users")]
struct UserController;

impl UserController {
    #[handler(get, "")]  // Maps to /api/v1/users
    async fn list_users(&self) -> Result<Response> {
        // Implementation
        Ok(response::json(serde_json::json!([])))
    }

    #[handler(get, "/:id")]  // Maps to /api/v1/users/:id
    async fn get_user(&self, Path(id): Path<i32>) -> Result<Response> {
        // Implementation
        Ok(response::json(serde_json::json!({})))
    }

    #[handler(post, "")]  // Maps to /api/v1/users
    async fn create_user(&self, Json(input): Json<CreateUser>) -> Result<Response> {
        // Implementation
        Ok(response::json(serde_json::json!({})))
    }
}
```

### `#[extract]`

Defines custom extractors for request data.

```rust
use oxidite_macros::extract;
use oxidite::prelude::*;

#[extract]
struct AuthenticatedUser {
    user_id: i32,
    permissions: Vec<String>,
}

// Automatically implements FromRequest trait
impl AuthenticatedUser {
    async fn from_request(req: &Request) -> Result<Self, Error> {
        // Extract authentication data from request
        // Implementation details...
        Ok(AuthenticatedUser {
            user_id: 1,
            permissions: vec!["read".to_string()],
        })
    }
}

async fn protected_handler(user: AuthenticatedUser) -> Result<Response> {
    Ok(response::json(serde_json::json!({
        "user_id": user.user_id,
        "permissions": user.permissions
    })))
}
```

### `#[migration]`

Generates database migration code from struct definitions.

```rust
use oxidite_macros::migration;

#[migration("20231201000001_create_users_table")]
fn create_users_table() -> Migration {
    Migration::new()
        .create_table("users")
        .column("id", ColumnType::Integer, |c| c.primary_key().auto_increment())
        .column("email", ColumnType::Text, |c| c.unique().not_null())
        .column("name", ColumnType::Text, |c| c.not_null())
        .column("created_at", ColumnType::Timestamp, |c| c.default("now"))
        .build()
}
```

### `#[validate]`

Adds validation attributes to struct fields.

```rust
use oxidite_macros::validate;
use serde::Deserialize;

#[derive(Deserialize, validate::Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    
    #[validate(email)]
    pub email: String,
    
    #[validate(range(min = 18, max = 120))]
    pub age: u8,
    
    #[validate(custom = "validate_password")]
    pub password: String,
}

fn validate_password(password: &str) -> Result<(), &'static str> {
    if password.len() >= 8 {
        Ok(())
    } else {
        Err("Password must be at least 8 characters")
    }
}
```

### `#[route_group]`

Groups multiple routes with shared middleware and prefixes.

```rust
use oxidite_macros::route_group;
use oxidite::prelude::*;

#[route_group("/api/v1", middleware = "auth_middleware")]
mod api_routes {
    use super::*;

    #[handler(get, "/users")]
    async fn list_users() -> Result<Response> {
        Ok(response::json(serde_json::json!([])))
    }

    #[handler(get, "/posts")]
    async fn list_posts() -> Result<Response> {
        Ok(response::json(serde_json::json!([])))
    }
}
```

### `#[test_helper]`

Creates test helpers and fixtures for testing.

```rust
use oxidite_macros::test_helper;

#[test_helper]
struct TestApp {
    pub router: Router,
    pub db: TestDatabase,
}

impl TestApp {
    pub async fn new() -> Self {
        let mut router = Router::new();
        // Setup routes
        
        let db = TestDatabase::new().await;
        
        Self { router, db }
    }
}

// Generates helper functions for testing
#[tokio::test]
async fn test_user_creation() {
    let app = TestApp::setup().await;
    let client = app.test_client().await;
    
    let response = client.post("/users")
        .json(&serde_json::json!({
            "name": "Test User",
            "email": "test@example.com"
        }))
        .send()
        .await;
    
    assert_eq!(response.status(), 200);
}
```

## Advanced Features

### Macro Attributes

Most macros support various attributes for customization:

```rust
#[derive(Model)]
#[model(
    table = "users",
    schema = "public",
    timestamps,  // Automatically add created_at and updated_at
    soft_delete, // Enable soft delete capability
    cache_ttl = 300  // Cache for 5 minutes
)]
pub struct User {
    #[model(primary_key, auto_increment)]
    pub id: i32,
    
    #[model(unique, indexed)]
    pub email: String,
    
    #[model(not_null, length(max = 100))]
    pub name: String,
    
    #[model(encrypted)]  // Automatically encrypt/decrypt
    pub password_hash: String,
}
```

### Conditional Compilation

Macros support conditional compilation flags:

```rust
#[handler(get, "/admin", condition = "feature = \"admin\"")]
async fn admin_panel() -> Result<Response> {
    // Only compiled when admin feature is enabled
    Ok(response::html("<h1>Admin Panel</h1>"))
}
```

### Custom Derive Options

Customizable derive behavior:

```rust
#[derive(Model)]
#[model(
    table = "users",
    derive_serialize = true,     // Auto-derive Serialize
    derive_deserialize = true,   // Auto-derive Deserialize
    derive_debug = true,         // Auto-derive Debug
    skip_benchmarks = false      // Include in benchmarks
)]
pub struct User {
    // fields...
}
```

## Performance

- **Compile-time Generation**: All code is generated at compile time with zero runtime overhead
- **Optimized Implementations**: Generated code follows performance best practices
- **Minimal Dependencies**: Macros have minimal impact on compile times

## Best Practices

1. **Use `#[derive(Model)]`** for all database entities to get automatic CRUD operations
2. **Combine `#[handler]` and `#[controller]`** for organized route structure
3. **Apply `#[validate]`** to all input structs for automatic validation
4. **Use `#[route_group]`** to organize related endpoints with shared middleware

## License

MIT