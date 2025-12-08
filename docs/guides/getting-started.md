# Getting Started with Oxidite

This guide will help you build your first web application with Oxidite.

## Installation

### Prerequisites

- Rust 1.70 or higher
- Cargo package manager

### Create a New Project with the CLI

The easiest way to get started is by using the `oxidite-cli`.

```bash
cargo install --path oxidite-cli
oxidite new my-app
```

This will create a new Oxidite project with a default structure.

## Your First Route

Navigate to your new project and open `src/main.rs`. It will look something like this:

```rust
use oxidite::prelude::*;

mod routes;
mod controllers;
mod models;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut router = Router::new();
    
    // Register routes
    routes::register(&mut router);

    let server = Server::new(router);
    println!("🚀 API Server running on http://127.0.0.1:8080");
    server.listen("127.0.0.1:8080".parse()?).await?;

    Ok(())
}
```

The routes are defined in `src/routes/mod.rs`:

```rust
use oxidite::prelude::*;
use serde_json::json;

pub fn register(router: &mut Router) {
    router.get("/api/health", health);
}

async fn health(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    Ok(OxiditeResponse::json(json!({"status": "ok"})))
}
```

### Run Your App

```bash
cargo run
```

Visit `http://localhost:8080/api/health` in your browser!

## JSON API Example

Let's create a simple JSON API for managing users.

First, generate a model and controller:
```bash
oxidite make model User
oxidite make controller UserController
```

Now, let's define the `User` model in `src/models/user.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
}
```

Next, let's add the routes in `src/routes/mod.rs`:
```rust
use crate::controllers::user_controller::{list_users, get_user, create_user};

pub fn register(router: &mut Router) {
    router.get("/api/users", list_users);
    router.get("/api/users/:id", get_user);
    router.post("/api/users", create_user);
}
```

Finally, implement the controller functions in `src/controllers/user_controller.rs`:
```rust
use oxidite::prelude::*;
use crate::models::user::User;
use serde_json::json;

pub async fn list_users(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    let users = vec![
        User { id: 1, name: "Alice".into(), email: "alice@example.com".into() },
        User { id: 2, name: "Bob".into(), email: "bob@example.com".into() },
    ];
    Ok(OxiditeResponse::json(json!(users)))
}

pub async fn get_user(req: OxiditeRequest) -> Result<OxiditeResponse> {
    let id: i64 = req.param("id")?;
    let user = User {
        id,
        name: "Alice".into(),
        email: "alice@example.com".into(),
    };
    Ok(OxiditeResponse::json(json!(user)))
}

#[derive(Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

pub async fn create_user(mut req: OxiditeRequest) -> Result<OxiditeResponse> {
    let data: CreateUserRequest = req.body_json().await?;
    let user = User {
        id: 3,
        name: data.name,
        email: data.email,
    };
    Ok(OxiditeResponse::json(json!(user)))
}
```

## Using Middleware

Add CORS and logging in `src/main.rs`:

```rust
use oxidite::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = Router::new();
    // ... register routes
    
    // Add middleware
    let app = ServiceBuilder::new()
        .layer(LoggerLayer)
        .layer(CorsLayer::permissive())
        .service(app);
    
    Server::new(app)
        .listen("127.0.0.1:3000".parse()?)
        .await
}
```

## Next Steps

- [Database Guide](database.md) - Learn about the ORM
- [Authentication Guide](authentication.md) - Add user authentication
- [Background Jobs](background-jobs.md) - Process async tasks
- [Testing Guide](testing.md) - Test your application
