# Oxidite Framework Documentation

Welcome to the official documentation for **Oxidite**, a modern, high-performance web framework for Rust.

## 🚀 Introduction

Oxidite is designed to be familiar to developers coming from **FastAPI** (Python) or **Express.js** (Node.js), while leveraging the performance and safety of Rust.

### Key Features
- **Fast & Async**: Built on top of `hyper` and `tokio`.
- **Easy Routing**: Express-like routing syntax.
- **Auto-Documentation**: OpenAPI (Swagger UI) generation out of the box.
- **Full-Stack Ready**: Built-in template engine (Jinja2-like), static file serving, and WebSocket support.
- **Production Grade**: Includes middleware for CORS, CSRF, Rate Limiting, and more.

## 🛠️ Getting Started

### Installation

Add Oxidite to your `Cargo.toml`:

```toml
[dependencies]
oxidite = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
```

### Basic Example

```rust
use oxidite::{Oxidite, Request, Response, Router};

#[tokio::main]
async fn main() {
    let mut app = Oxidite::new();
    let mut router = Router::new();

    router.get("/", |req| async {
        Ok(Response::new("Hello, Oxidite!".into()))
    });

    app.router(router);
    app.listen("127.0.0.1:3000").await.unwrap();
}
```

## 📚 Guides

### 1. Building a REST API
Oxidite makes building APIs simple. Use `json()` helper for responses.

```rust
router.get("/api/users", |req| async {
    let users = vec!["Alice", "Bob"];
    Ok(oxidite::response::json(users))
});
```

### 2. Full-Stack Web Apps
Use the `oxidite-template` crate for server-side rendering.

```rust
router.get("/profile", |req| async {
    let ctx = context! { "username" => "Alice" };
    let html = templates.render("profile.html", &ctx)?;
    Ok(oxidite::response::html(html))
});
```

### 4. Automatic API Documentation
Oxidite includes built-in support for OpenAPI (Swagger) documentation.

**1. Add dependency:**
```toml
[dependencies]
oxidite-openapi = "0.1.0"
```

**2. Generate Spec:**
```rust
use oxidite_openapi::{OpenApiBuilder, Info};

router.get("/api/openapi.json", |req| async {
    let spec = OpenApiBuilder::new("My API", "1.0.0")
        .description("My awesome API")
        .build();
    Ok(oxidite::response::json(spec))
});
```

**3. Serve Docs UI:**
Create a route that renders the Swagger UI template (included in `oxidite-template` or custom).

### How-To Guides

Step-by-step guides for common tasks:

- **[Building a Fullstack Application](guides/fullstack.md)** - Create a complete web app with Oxidite
- **[CLI Tool Usage](guides/cli.md)** - Master the Oxidite command-line interface
- **[Static File Serving](guides/static-files.md)** - Serve static assets efficiently
- **[Authorization & Access Control](guides/authorization.md)** - Implement RBAC/PBAC
- **[API Key Authentication](guides/api-keys.md)** - Secure your API with key-based auth

### Project Status

- **[Implementation Status](../STATUS.md)** - Current feature completeness and roadmap progress
- **[Full Roadmap](../ROADMAP.md)** - Complete development roadmap

## 📖 Demo Application

The `examples/demo-app` directory contains a complete full-stack application showcasing:
- **Database Integration**: SQLite with migrations.
- **Authentication**: JWT-based auth.
- **Templates**: Beautiful UI with server-side rendering.
- **API Docs**: Auto-generated OpenAPI documentation.

### Running the Demo
```bash
cd examples/demo-app
cargo run
```
Visit `http://localhost:8080` to see it in action!

## 🔧 Advanced Usage

### Middleware
```rust
use oxidite::middleware::{CorsLayer, LoggerLayer};

let app = Oxidite::new()
    .layer(LoggerLayer::new())
    .layer(CorsLayer::permissive());
```

### Error Handling
Oxidite provides a robust error handling system. You can return `Result<Response, Error>` from any handler.

## 🤝 Contributing
We welcome contributions! Please see [CONTRIBUTING.md](../CONTRIBUTING.md) for details.

---
*Built with ❤️ by the Oxidite Team*
