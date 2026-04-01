# Oxidite Web Framework

<div align="center">

<img src="docs/logo/oxidite.svg" width="200" alt="Oxidite Logo">

A modern, high-performance web framework for Rust, inspired by FastAPI, Express.js, and Laravel.

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![GitHub](https://img.shields.io/badge/github-meshackbahati%2Frust--oxidite-black)](https://github.com/meshackbahati/rust-oxidite)
[![Stability](https://img.shields.io/badge/stability-beta-2ea043.svg)](ROADMAP.md)
[![Roadmap](https://img.shields.io/badge/roadmap-public-0a66c2.svg)](ROADMAP.md)

Built by [Meshack Bahati Ouma](https://github.com/meshackbahati)

</div>

---

## What is Oxidite?

Oxidite is a batteries-included web framework that combines Rust's performance with developer-friendly APIs. It provides a complete ecosystem for building scalable web applications, from REST APIs to fullstack server-side rendered apps.

## Key Features

- **High Performance**: Built on `hyper` and `tokio` for blazing speed
- **Advanced ORM**: Complete database layer with relationships, soft deletes, validation
- **Powerful CLI**: Scaffolding, migrations, hot-reload dev server, code generators
- **Batteries Included**: RBAC/PBAC, API Keys, Queues, Caching, Email, Storage, Plugins
- **Enterprise Security**: Password hashing, JWT, OAuth2, 2FA, rate limiting
- **Template Engine**: Jinja2-style templates with inheritance and auto-escaping
- **Real-time**: WebSockets and Redis pub/sub support
- **Type-Safe**: Strong typing for requests, responses, and database queries
- **Auto-Documentation**: OpenAPI/Swagger UI generation

> **Roadmap**: See [ROADMAP.md](ROADMAP.md) for public progress and upcoming milestones.

## Installation

Install the `oxidite-cli` package to get the `oxidite` executable:

```bash
# Install from crates.io
cargo install oxidite-cli

# Install this generated CLI build explicitly
cargo install oxidite-cli --version 2.1.0-gen

# Install from source (recommended for development)
cargo install --path oxidite-cli
```

## Usage Guide

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

async fn hello(_req: Request) -> Result<Response> {
    Ok(Response::json(json!({ "message": "Hello, Oxidite!" })))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    router.get("/", hello);

    let server = Server::new(router);
    println!("Server running on http://127.0.0.1:3000");
    server.listen("127.0.0.1:3000".parse()?).await
}
```

### 3. Database And Development

Navigate to your project, run migrations, and start the development server.

```bash
cd my-app
oxidite migrate
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

async fn index() -> Result<Response> {
    let engine = TemplateEngine::new("templates");
    let mut context = Context::new();
    context.insert("name", "Oxidite");
    let html = engine.render("index.html", &context)?;
    Ok(Response::html(html))
}
```

## Documentation

- [Getting Started](docs/getting-started.md)
- [Features](docs/features.md)
- [API Reference](docs/api.md)
- [CLI Reference](docs/cli.md)
- [Database Guide](docs/database.md)
- [Authentication](docs/authentication.md)
- [Realtime Features](docs/realtime.md)

## Architecture

Oxidite is composed of modular crates:

| Crate | Description |
|-------|-------------|
| `oxidite` | Top-level facade crate and prelude |
| `oxidite-core` | HTTP server, routing |
| `oxidite-macros` | Procedural macros and route/model derives |
| `oxidite-config` | Configuration loading and environment support |
| `oxidite-middleware` | Common middleware (logging, CORS, auth, etc.) |
| `oxidite-utils` | Shared helpers and utilities |
| `oxidite-template` | Template engine integration and SSR helpers |
| `oxidite-db` | ORM and database abstraction |
| `oxidite-auth` | Authentication, JWT/OAuth2, RBAC/PBAC primitives |
| `oxidite-cache` | Caching abstraction and adapters |
| `oxidite-queue` | Job queue and background processing APIs |
| `oxidite-realtime` | WebSockets/SSE and realtime integration |
| `oxidite-mail` | Mailer abstraction and providers |
| `oxidite-storage` | File/object storage abstractions |
| `oxidite-security` | Security utilities and hardening helpers |
| `oxidite-cli` | CLI scaffolding, dev workflow, and generators |
| `oxidite-testing` | Test helpers and integration testing utilities |
| `oxidite-openapi` | OpenAPI schema generation and docs support |
| `oxidite-graphql` | GraphQL integration |
| `oxidite-plugin` | Plugin system and extension APIs |

## Community & Support

Have questions or want to connect with other Oxidite developers? Join our community on [GitHub Discussions](https://github.com/meshackbahati/rust-oxidite/discussions).

## Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md).

## License

MIT License - see [LICENSE](LICENSE) for details.
