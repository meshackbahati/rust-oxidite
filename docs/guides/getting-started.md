# Getting Started with Oxidite

Welcome to Oxidite! This guide will help you build your first web application with Oxidite.

---

## Prerequisites

- **Rust 1.75 or later** ([Install Rust](https://www.rust-lang.org/tools/install))
- **Basic knowledge of Rust** and async programming
- A text editor or IDE (VS Code with rust-analyzer recommended)

---

## Installation

Currently, Oxidite is in development. To use it:

```bash
# Clone the repository
git clone https://github.com/your username/oxidite.git
cd oxidite

# Build the project
cargo build
```

---

## Your First Application

### 1. Create a New Binary Project

Since Oxidite doesn't have a `new` command yet, create manually:

```bash
mkdir my-oxidite-app
cd my-oxidite-app
cargo init
```

### 2. Add Oxidite as a Dependency

Edit `Cargo.toml`:

```toml
[package]
name = "my-oxidite-app"
version = "0.1.0"
edition = "2021"

[dependencies]
oxidite-core = { path = "../oxidite/oxidite-core" }
oxidite-middleware = { path = "../oxidite/oxidite-middleware" }
tokio = { version = "1", features = ["full"] }
hyper = { version = "1", features = ["full"] }
http-body-util = "0.1"
bytes = "1"
```

### 3. Write Your First Handler

Edit `src/main.rs`:

```rust
use oxidite_core::{Router, Server, OxiditeRequest, OxiditeResponse, Result};
use oxidite_middleware::{ServiceBuilder, LoggerLayer};
use http_body_util::Full;
use bytes::Bytes;

async fn hello(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    Ok(hyper::Response::new(Full::new(Bytes::from("Hello, World!"))))
}

async fn greet(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    Ok(hyper::Response::new(Full::new(Bytes::from("Greetings from Oxidite!"))))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create a router
    let mut router = Router::new();
    
    // Register routes
    router.get("/", hello);
    router.get("/greet", greet);
    
    // Add middleware
    let service = ServiceBuilder::new()
        .layer(LoggerLayer)
        .service(router);
    
    // Start the server
    let server = Server::new(service);
    println!("🚀 Server running on http://127.0.0.1:3000");
    server.listen("127.0.0.1:3000".parse().unwrap()).await
}
```

### 4. Run Your Application

```bash
cargo run
```

Visit `http://127.0.0.1:3000` in your browser!

---

## Building a JSON API

Let's build a simple user API:

```rust
use oxidite_core::{Router, Server, OxiditeRequest, OxiditeResponse, Result};
use serde::{Deserialize, Serialize};
use http_body_util::Full;
use bytes::Bytes;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
}

async fn list_users(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    let users = vec![
        User { id: 1, name: "Alice".into(), email: "alice@example.com".into() },
        User { id: 2, name: "Bob".into(), email: "bob@example.com".into() },
    ];
    
    let json = serde_json::to_vec(&users).unwrap();
    
    Ok(hyper::Response::builder()
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(json)))
        .unwrap())
}

async fn create_user(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    // In a real app, parse JSON body and save to database
    let user = User {
        id: 3,
        name: "Charlie".into(),
        email: "charlie@example.com".into(),
    };
    
    let json = serde_json::to_vec(&user).unwrap();
    
    Ok(hyper::Response::builder()
        .status(201)
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(json)))
        .unwrap())
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    router.get("/users", list_users);
    router.post("/users", create_user);
    
    let server = Server::new(router);
    println!("🚀 API running on http://127.0.0.1:3000");
    server.listen("127.0.0.1:3000".parse().unwrap()).await
}
```

Test it:

```bash
# List users
curl http://127.0.0.1:3000/users

# Create user
curl -X POST http://127.0.0.1:3000/users \
  -H "Content-Type: application/json" \
  -d '{"name":"Charlie","email":"charlie@example.com"}'
```

---

## Middleware

Oxidite uses Tower middleware. Here's how to add CORS:

```rust
use oxidite_middleware::{ServiceBuilder, LoggerLayer, tower_http};
use tower_http::cors::{CorsLayer, Any};

let service = ServiceBuilder::new()
    .layer(LoggerLayer)
    .layer(
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
    )
    .service(router);
```

---

## Error Handling

Oxidite has built-in error handling:

```rust
use oxidite_core::Error;

async fn might_fail(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    if some_condition {
        return Err(Error::BadRequest("Invalid input".to_string()));
    }
    
    // ... handle request
    Ok(response)
}
```

---

## Next Steps

- [Architecture Overview](../architecture/overview.md)
- [Routing Guide](routing.md) _(coming soon)_
- [Database Guide](database.md) _(coming soon)_
- [Authentication Guide](auth.md) _(coming soon)_

---

## Examples

Check out the `examples/` directory:

- `hello-world.rs` - Basic hello world
- `full-api.rs` - Complete REST API example

Run them with:

```bash
cargo run --example hello-world
cargo run --example full-api
```

---

## Common Issues

### Error: `cannot find type OxiditeRequest`

Make sure you've imported the prelude:

```rust
use oxidite_core::{OxiditeRequest, OxiditeResponse, Result};
```

### Slow compile times

Oxidite uses many dependencies. First compilation is slow but subsequent builds use incremental compilation.

### Port already in use

Change the port in your `listen call:

```rust
server.listen("127.0.0.1:8080".parse().unwrap()).await
```

---

## Getting Help

- [GitHub Discussions](https://github.com/yourusername/oxidite/discussions)
- [GitHub Issues](https://github.com/yourusername/oxidite/issues)
- Read the [Architecture Documentation](../architecture/overview.md)

---

Happy coding with Oxidite! 🦀
