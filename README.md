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

- **⚡ High Performance**: Built on `hyper` and `tokio`.
- **🛠️ Powerful CLI**: Scaffolding, hot-reloading dev server, code generation.
- **🔋 Batteries Included**: Auth, Database (ORM), Queues, Caching, Email, Storage.
- **🎨 Template Engine**: Django-style templates with inheritance.
- **🔄 Real-time**: WebSockets and Pub/Sub support.
- **📝 Type-Safe**: Strong typing for requests, responses, and database queries.

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

You will be prompted to select a project type:

- **Fullstack Application**: Complete setup with templates, static files, and database.
- **REST API**: Optimized for backend services (JSON only).
- **Microservice**: Minimal setup for specialized services.
- **Serverless Function**: Lightweight event handler.

### 2. Development

Navigate to your project and start the development server.

```bash
cd my-app
oxidite dev
```

The server will start on `http://127.0.0.1:8080`. The `dev` command watches your files and automatically restarts the server when you make changes.

### 3. Project Structure

A typical Fullstack project looks like this:

```
my-app/
├── src/
│   ├── controllers/   # Request handlers
│   ├── models/        # Database structs
│   ├── routes/        # Route definitions
│   ├── services/      # Business logic
│   ├── middleware/    # Custom middleware
│   └── main.rs        # Entry point
├── templates/         # HTML templates
├── public/            # Static assets (css, js, images)
├── config.toml        # Configuration
└── Cargo.toml         # Dependencies
```

### 4. Building APIs

Generate a new controller and model using the CLI:

```bash
oxidite make model User
oxidite make controller Users
```

This creates `src/models/user.rs` and `src/controllers/users.rs` with boilerplate CRUD operations.

### 5. Serving Static Files

In a Fullstack project, static files in `public/` are served from the root URL by default.

- `public/css/style.css` -> `http://localhost:8080/css/style.css`

You can configure this in `src/main.rs`:

```rust
// Serve static files from "public" directory (fallback route)
// Register this LAST to avoid blocking other routes
router.get("/*", serve_static);
```

### 6. Templates

Oxidite uses a powerful template engine. Create views in `templates/`:

```html
<!-- templates/index.html -->
{% extends "base.html" %}

{% block content %}
    <h1>Hello, {{ name }}!</h1>
{% endblock %}
```

Render them in your controller:

```rust
async fn index(req: Request) -> Result<Response> {
    let mut context = Context::new();
    context.insert("name", "Oxidite");
    Ok(Response::html(engine.render("index.html", &context)?))
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
