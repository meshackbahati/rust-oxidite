# Getting Started with Oxidite

This guide will help you build your first web application with Oxidite.

## Installation

### Prerequisites

- Rust 1.70 or higher
- Cargo package manager

### Create a New Project

```bash
cargo new my-oxidite-app
cd my-oxidite-app
```

### Add Oxidite

Add Oxidite to your `Cargo.toml`:

```toml
[dependencies]
oxidite = "1.0"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

## Your First Route

Replace the contents of `src/main.rs`:

```rust
use oxidite::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = Router::new();
    
    app.get("/", hello);
    app.get("/users/:id", get_user);
    
    println!("🚀 Server running on http://127.0.0.1:3000");
    
    Server::new(app)
        .listen("127.0.0.1:3000".parse().unwrap())
        .await
}

async fn hello(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    Ok(Response::text("Hello, Oxidite!"))
}

async fn get_user(Path(params): Path<std::collections::HashMap<String, String>>) -> Result<OxiditeResponse> {
    let user_id = params.get("id").unwrap();
    Ok(OxiditeResponse::text(format!("User ID: {}", user_id)))
}
```

### Run Your App

```bash
cargo run
```

Visit `http://localhost:3000` in your browser!

## JSON API Example

Let's create a simple JSON API:

```rust
use oxidite::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: i64,
    name: String,
    email: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = Router::new();
    
    app.get("/api/users", list_users);
    app.get("/api/users/:id", get_user);
    app.post("/api/users", create_user);
    
    Server::new(app)
        .listen("127.0.0.1:3000".parse().unwrap())
        .await
}

async fn list_users(_req: OxiditeRequest) -> Result<Json<Vec<User>>> {
    let users = vec![
        User { id: 1, name: "Alice".into(), email: "alice@example.com".into() },
        User { id: 2, name: "Bob".into(), email: "bob@example.com".into() },
    ];
    Ok(Json(users))
}

async fn get_user(Path(params): Path<std::collections::HashMap<String, String>>) -> Result<Json<User>> {
    let id = params.get("id").unwrap().parse().unwrap();
    Ok(Json(User {
        id,
        name: "Alice".into(),
        email: "alice@example.com".into(),
    }))
}

#[derive(Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

async fn create_user(Json(data): Json<CreateUserRequest>) -> Result<Json<User>> {
    Ok(Json(User {
        id: 3,
        name: data.name,
        email: data.email,
    }))
}
```

## Using Middleware

Add CORS and logging:

```rust
use oxidite::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = Router::new();
    app.get("/", hello);
    
    // Add middleware
    let app = ServiceBuilder::new()
        .layer(LoggerLayer)
        .layer(CorsLayer::permissive())
        .service(app);
    
    Server::new(app)
        .listen("127.0.0.1:3000".parse().unwrap())
        .await
}
```

## Next Steps

- [Database Guide](database.md) - Learn about the ORM
- [Authentication Guide](authentication.md) - Add user authentication
- [Background Jobs](background-jobs.md) - Process async tasks
- [Testing Guide](testing.md) - Test your application

## Using the CLI Tool

Install the `oxidite-cli` package to get the `oxidite` executable:

```bash
cargo install oxidite-cli

# Install this generated CLI build explicitly
cargo install oxidite-cli --version 2.1.0-gen
```

Create and run a project:

```bash
oxidite new myapp --type fullstack
cd myapp
oxidite migrate
oxidite dev
```

Generate code:

```bash
oxidite generate model User
oxidite generate model Profile display_name:string
oxidite generate controller UserController
oxidite generate middleware AuthMiddleware
oxidite generate route users
oxidite generate service Billing
oxidite generate validator CreateUser
oxidite generate job SendDigest
oxidite generate policy Post
oxidite generate event UserSignedUp
oxidite generate migration create_users_table
oxidite generate seeder users_seed
```

## Feature Flags

Use only what you need:

```toml
# Full framework (default)
[dependencies]
oxidite = "1.0"

# Minimal (HTTP only)
[dependencies]
oxidite = { version = "1.0", default-features = false }

# Custom features
[dependencies]
oxidite = { version = "1.0", features = ["database", "auth", "queue"] }
```

Available features:
- `database` - ORM and migrations
- `auth` - Authentication and authorization
- `queue` - Background job processing
- `cache` - Caching support
- `realtime` - WebSocket features
- `templates` - Server-side rendering
- `mail` - Email sending
- `storage` - File storage

## Common Patterns

### State Management

```rust
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    db: Arc<Database>,
}

app.get("/", |State(state): State<AppState>| async move {
    // Use state.db
    Ok(Response::text("OK"))
});
```

### Error Handling

```rust
async fn handler() -> Result<OxiditeResponse> {
    let data = fetch_data().await?;  // ? operator works
    Ok(Json(data))
}
```

## Troubleshooting

**Port already in use:**
```bash
# Change the port
Server::new(app).listen("127.0.0.1:8080".parse()?).await
```

**Dependency errors:**
```bash
cargo clean
cargo update
cargo build
```

## Resources

- [API Documentation](https://docs.rs/oxidite)
- [GitHub Repository](https://github.com/Kyle6012/rust-oxidite)
- [Example Applications](https://github.com/Kyle6012/rust-oxidite/tree/main/examples)
