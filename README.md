# Oxidite Web Framework

<div align="center">

<img src="docs/logo/oxidite.svg" width="200" alt="Oxidite Logo">

A modern, high-performance web framework for Rust, inspired by FastAPI, Express.js, and Laravel.

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![GitHub](https://img.shields.io/badge/github-Kyle6012%2Frust--oxidite-black)](https://github.com/Kyle6012/rust-oxidite)
[![Status](https://img.shields.io/badge/status-alpha-yellow.svg)](STATUS.md)

Built with ❤️ by [Meshack Bahati Ouma](https://github.com/Kyle6012)

</div>

---

## 🚀 What is Oxidite?

Oxidite is a batteries-included web framework that combines Rust's performance with developer-friendly APIs. It provides a complete ecosystem for building scalable web applications, from REST APIs to fullstack server-side rendered apps.

## ✨ Key Features

- **⚡ High Performance**: Built on `hyper` and `tokio` for blazing speed
- **🗄️ Advanced ORM**: Complete database layer with relationships, soft deletes, validation
- **🛠️ Powerful CLI**: Scaffolding, migrations, hot-reload dev server, code generators
- **🔋 Batteries Included**: RBAC/PBAC, API Keys, Queues, Caching, Email, Storage
- **🔐 Enterprise Security**: Password hashing, JWT, OAuth2, 2FA, rate limiting
- **🎨 Template Engine**: Jinja2-style templates with inheritance and auto-escaping
- **🔄 Real-time**: WebSockets and Redis pub/sub support
- **📝 Type-Safe**: Strong typing for requests, responses, and database queries
- **📊 Auto-Documentation**: OpenAPI/Swagger UI generation

> **Status**: See [STATUS.md](STATUS.md) for detailed feature completeness

## 📦 Installation

Install the Oxidite CLI tool to get started:

```bash
# Install from source (recommended for development)
cargo install --path oxidite-cli
```

## 🛠️ Usage Guide

### 1. Create a New Project

Oxidite provides an interactive wizard to set up your project.

```bash
oxidite new my-app
```

### 2. A Simple Example

Here's a basic "Hello, World!" example:

```rust
use oxidite::prelude::*;
use serde_json::json;

async fn hello(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    Ok(OxiditeResponse::json(json!({ "message": "Hello, Oxidite!" })))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    router.get("/", hello);

    let server = Server::new(router);
    println!("🚀 Server running on http://127.0.0.1:3000");
    server.listen("127.0.0.1:3000".parse()?).await
}
```

### 3. Development

Navigate to your project and start the development server.

```bash
cd my-app
oxidite dev
```

### 4. Serving Static Files

In a Fullstack project, static files in `public/` are served from the root URL by default.

```rust
use oxidite_template::serve_static;

// Serve static files from "public" directory (fallback route)
// Register this LAST to avoid blocking other routes
router.get("/*", serve_static);
```

### 5. Templates

Render templates in your handlers:

```rust
use oxidite_template::{TemplateEngine, Context};

async fn index() -> Result<OxiditeResponse> {
    let engine = TemplateEngine::new("templates");
    let mut context = Context::new();
    context.insert("name", "Oxidite");
    let html = engine.render("index.html", &context)?;
    Ok(OxiditeResponse::html(html))
}
```

## 📖 Documentation

- [Getting Started](docs/guides/getting-started.md)
- [CLI Reference](docs/guides/cli.md)
- [Fullstack Guide](docs/guides/fullstack.md)
- [Database Guide](docs/guides/database.md)
- [Authentication](docs/guides/authentication.md)
- [Realtime Features](docs/guides/realtime.md)

## 🏗️ Architecture

Oxidite is composed of modular crates:

| Crate | Description |
|-------|-------------|
| `oxidite-core` | HTTP server, routing |
| `oxidite-cli` | Command-line tools |
| `oxidite-auth` | Authentication & OAuth2 |
| `oxidite-db` | Database abstraction |
| `oxidite-template` | Template engine |
| `oxidite-realtime` | WebSockets & SSE |
| ...and more | |

## 🤝 Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md).

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.
