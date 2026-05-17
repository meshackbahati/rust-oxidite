# Oxidite Web Framework

<div align="center">

<img src="../docs/logo/oxidite.svg" width="200" alt="Oxidite Logo">

A modern, high-performance web framework for Rust, inspired by FastAPI, Express.js, and Laravel.

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../LICENSE)
[![GitHub](https://img.shields.io/badge/github-Kyle6012%2Frust--oxidite-black)](https://github.com/Kyle6012/rust-oxidite)
[![Status](https://img.shields.io/badge/status-beta-yellow.svg)](../STATUS.md)

Built with ❤️ by [Meshack Bahati Ouma](https://github.com/Kyle6012)

</div>

---

## 🚀 What is Oxidite?

Oxidite is a modern, high-performance, developer-first web framework for Rust that provides a Rails-like, batteries-included experience. Built on top of the lightning-fast `hyper` and `tokio` asynchronous runtimes, it eliminates boilerplates by providing first-class identity & access management, a fully-featured custom ORM with async validation and eager-loading, a unified cloud/local storage manager, durable background queues, interactive REPL (`oxidite tinker`), hot-reload dev servers, and beautiful diagnostic pages. Oxidite is designed to provide maximum velocity and system-level performance, without compromising on developer ergonomics.

## ✨ What's Included

Oxidite provides a complete out-of-the-box toolkit for modern application development:
- **`oxidite-core`**: High-performance routing, hyper HTTP server, and type-safe extractors (`Json`, `Path`, `Query`, `State`, `Form`, `Cookies`, `Body`).
- **`oxidite-db`**: Advanced custom ORM featuring async validation rules (`length`, `range`, `email`, `url`, `regex`, `custom`, `unique`), relationships (`has_many`, `has_one`, `belongs_to`), and auto-diff schema migrations.
- **`oxidite-auth`**: End-to-end Identity & Access Management supporting RBAC/PBAC, API keys, JWT session handling, and OAuth2 integration.
- **`oxidite-realtime`**: Full-duplex WebSockets, Server-Sent Events (SSE), and Redis-backed event broadcasting.
- **`oxidite-queue`**: Durable background job execution with automatic retries and dead-letter queues.
- **`oxidite-cache`**: Transparent caching supporting in-memory and Redis backends.
- **`oxidite-storage`**: Unified storage API with local disk and AWS S3/Cloudinary/ImageKit compatibility.
- **`oxidite-template`**: Lightweight server-side rendering (SSR) templates.
- **`oxidite-openapi`**: Automatic Swagger UI/OpenAPI 3.0 document generation.
- **`oxidite-cli`**: Command-line scaffolding, migrations, code generators, and `oxidite tinker` interactive console.

### 🗄️ Our ORM Goal: Parity with SeaORM & Diesel

We designed our built-in ORM to match the ergonomics, power, and safety of the ecosystem's industry-standard libraries like **SeaORM** and **Diesel**:
- **Compile-Time Checks**: Using our procedural macros and `handler_fn` route helper, handler extractor bindings and model queries are verified at compile time.
- **Solve the N+1 Problem**: Prevents N+1 database queries with static eager-loading helper methods (`eager_load_posts`, `eager_load_profile`) that execute batched SQL `IN` queries.
- **Auto-Diff Migrations**: Eliminate manually written SQL migration scripts. Oxidite's CLI parses and diffs your Rust struct models against the live database schema to generate migrations automatically.

### ⚡ Honest Benchmarks Notice

**We currently do not have public benchmarks.**
While Oxidite leverages `hyper` and `tokio` to achieve very high performance, we prioritised completing the unified framework ecosystem first. Detailed performance profiles (RPS, latency, memory usage) comparing Oxidite against Axum, Actix Web, and Loco will be published in future releases.

> **Status**: See [STATUS.md](../STATUS.md) for detailed feature completeness

## 📦 Installation

Install the Oxidite CLI tool to get started:

```bash
# Install from source (recommended for development)
cargo install --path ../oxidite-cli

# Or install from crates.io (when published)
cargo install oxidite-cli
```

## 🚀 Getting Started

### Quick Start

Create a new Oxidite project in seconds:

```bash
oxidite new my-app
```

Then run your application:

```bash
# Navigate to your project
cd my-app

# Start the development server
oxidite dev
```

Your application will be available at `http://127.0.0.1:8080`.

### Hello World Example

Here's a simple "Hello World" example:

```rust
use oxidite::prelude::*;

async fn hello(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    Ok(response::text("Hello, Oxidite!"))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = Router::new();
    
    app.get("/", hello);
    
    Server::new(app)
        .listen("127.0.0.1:3000".parse().unwrap())
        .await
}
```

### Using Extractors

Oxidite provides powerful type-safe extractors for handling different types of requests:

```rust
use oxidite::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

// Handle JSON requests
async fn create_user(Json(payload): Json<CreateUser>) -> Result<OxiditeResponse> {
    // payload contains deserialized JSON data
    Ok(response::json(serde_json::json!({
        "message": "User created successfully",
        "user": payload
    })))
}

// Handle form data
async fn create_user_form(Form(payload): Form<CreateUser>) -> Result<OxiditeResponse> {
    // payload contains deserialized form data
    Ok(response::json(serde_json::json!({
        "message": "User created from form",
        "user": payload
    })))
}

// Handle query parameters
async fn search_users(Query(params): Query<CreateUser>) -> Result<OxiditeResponse> {
    // params contains deserialized query parameters
    Ok(response::json(serde_json::json!({
        "message": "Search results",
        "query": params
    })))
}

// Handle path parameters
#[derive(Deserialize)]
struct UserId {
    id: u32,
}

async fn get_user(Path(params): Path<UserId>) -> Result<OxiditeResponse> {
    // params.id contains the path parameter
    Ok(response::json(serde_json::json!({
        "id": params.id,
        "name": "Sample User"
    })))
}

// Access cookies
async fn get_cookies(Cookies(cookies): Cookies) -> Result<OxiditeResponse> {
    // cookies is a HashMap<String, String> containing request cookies
    Ok(response::json(serde_json::json!(cookies)))
}

// Access raw body
async fn handle_raw_body(Body(body): Body) -> Result<OxiditeResponse> {
    // body is a String containing the raw request body
    Ok(response::text(format!("Received {} characters", body.len())))
}
```

## 🛠️ Core Concepts

### Routers and Handlers

Oxidite uses a clean routing system with async handlers:

```rust
use oxidite::prelude::*;

async fn home(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    Ok(response::html("<h1>Welcome to Oxidite!</h1>"))
}

async fn api_handler(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    Ok(response::json(serde_json::json!({
        "message": "Hello from API"
    })))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    
    // Basic routes
    router.get("/", home);
    router.get("/api", api_handler);
    
    // Routes with parameters
    router.get("/users/:id", get_user);
    router.post("/users", create_user);
    router.put("/users/:id", update_user);
    router.delete("/users/:id", delete_user);
    
    Server::new(router)
        .listen("127.0.0.1:3000".parse().unwrap())
        .await
}
```

### Request Handling

Oxidite provides several extractors to handle different types of requests:

- **Json<T>**: Extracts and deserializes JSON from request body
- **Form<T>**: Extracts and deserializes form data
- **Query<T>**: Extracts and deserializes query parameters
- **Path<T>**: Extracts and deserializes path parameters
- **Cookies**: Extracts cookies as HashMap
- **Body**: Extracts raw request body as String
- **State<T>**: Extracts application state from request extensions

### API Versioning

Oxidite supports multiple API versioning strategies:

```rust
use oxidite::prelude::*;

// URL-based versioning
async fn api_v1_handler(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    Ok(response::json(serde_json::json!({
        "version": "1.0",
        "data": "API v1 response"
    })))
}

async fn api_v2_handler(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    Ok(response::json(serde_json::json!({
        "version": "2.0",
        "data": "API v2 response",
        "enhanced": true
    })))
}

// Version-specific routes
router.get("/api/v1/users", api_v1_handler);
router.get("/api/v2/users", api_v2_handler);
```

### Error Handling

Oxidite provides comprehensive error handling with appropriate HTTP status codes:

```rust
use oxidite::prelude::*;

async fn protected_route(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    // Simulate a forbidden request
    Err(OxiditeError::Forbidden("Access denied".to_string()))
}

async fn conflict_route(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    // Simulate a conflict error
    Err(OxiditeError::Conflict("Resource conflict".to_string()))
}

async fn validation_route(Json(data): Json<MyData>) -> Result<OxiditeResponse> {
    // Validate the data
    if data.is_valid() {
        Ok(response::json(serde_json::json!("Valid")))
    } else {
        Err(OxiditeError::Validation("Invalid data".to_string()))
    }
}
```

## 📚 Documentation

Complete documentation is available in the [docs/](../docs/) directory:

- [Getting Started](../docs/getting-started.md) - Your first Oxidite application
- [Core Concepts](../docs/core-concepts.md) - Fundamental architecture and concepts
- [API Documentation](../docs/api.md) - Complete API reference
- [Framework Features](../docs/framework.md) - Framework features and capabilities
- [Added Features](../docs/features-added.md) - Recently added features and improvements
- [Database Guide](../docs/database.md) - ORM and database operations
- [Authentication Guide](../docs/authentication.md) - Authentication and authorization
- [Templating Guide](../docs/templating.md) - Server-side rendering
- [Middleware Guide](../docs/middleware.md) - Adding functionality with middleware
- [CLI Tools](../docs/cli.md) - Command-line interface

## 🏗️ Architecture

Oxidite is composed of modular crates that can be used independently:

| Crate | Description |
|-------|-------------|
| `oxidite` | Main crate with prelude and convenience exports |
| `oxidite-core` | Core HTTP server, routing, and extractors |
| `oxidite-db` | Database ORM with migrations and relationships |
| `oxidite-auth` | Authentication and authorization |
| `oxidite-template` | Template engine for server-side rendering |
| `oxidite-middleware` | Common middleware implementations |
| `oxidite-cli` | Command-line tools for project management |
| `oxidite-config` | Configuration management |
| `oxidite-cache` | Caching utilities |
| `oxidite-queue` | Background job processing |
| `oxidite-realtime` | Real-time features (WebSockets, SSE) |
| `oxidite-mail` | Email sending capabilities |
| `oxidite-storage` | File storage (local and S3) |
| `oxidite-openapi` | OpenAPI/Swagger integration |
| `oxidite-macros` | Procedural macros |
| `oxidite-security` | Security utilities |
| `oxidite-testing` | Testing utilities |
| `oxidite-utils` | Common utilities |

## 🧪 Testing

Oxidite provides comprehensive testing utilities:

```rust
use oxidite::prelude::*;
use oxidite_testing::TestClient;

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    
    #[tokio::test]
    async fn test_hello_endpoint() {
        let mut router = Router::new();
        router.get("/", hello);
        
        let client = TestClient::new(router);
        let response = client.get("/").send().await;
        
        assert_eq!(response.status(), 200);
        assert_eq!(response.text().await, "Hello, Oxidite!");
    }
}
```

## 🚀 Deployment

Deploy your Oxidite application with any Rust-compatible hosting provider:

```bash
# Build for release
cargo build --release

# Run the application
./target/release/my-app
```

For containerized deployment:

```dockerfile
FROM rust:latest
WORKDIR /app
COPY . .
RUN cargo build --release
CMD ["./target/release/my-app"]
```

## 🤝 Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md).

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.
