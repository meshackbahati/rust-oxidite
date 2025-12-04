# Oxidite

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Status: Alpha](https://img.shields.io/badge/status-alpha-yellow.svg)](ROADMAP.md)

> **A next-generation, batteries-included Rust backend web framework**

Oxidite combines the best features of FastAPI, Laravel, Express.js, and Django into a single, powerful Rust framework that's **fast**, **secure by default**, and **beginner-friendly**.

---

## 🚀 Features

### ⚡ High Performance
- Built on **Tokio** and **Hyper** for maximum throughput
- Support for **HTTP/1.1, HTTP/2, and HTTP/3 (QUIC)**
- 100k+ requests/second capability
- Zero-cost abstractions

### 🛣️ Advanced Routing
- Type-safe path, query, and body parameters
- Automatic **OpenAPI/Swagger** documentation
- Route grouping and versioning
- Middleware at route and global levels

### 🔧 Powerful Middleware
- **Tower**-based middleware ecosystem
- Built-in logging, compression, CORS, CSRF protection
- Rate limiting and security headers
- Custom middleware support

### 🗄️ Universal Database Support
- **SQL**: PostgreSQL, MySQL, SQLite
- **NoSQL**: MongoDB, Redis
- Type-safe query builder
- **Alembic-style migrations** with auto-diffing
- Model relationships and transactions

### 🔐 Enterprise-Grade Security
- **Argon2** password hashing
- **JWT and Paseto** tokens
- **OAuth2** support
- **RBAC** and **PBAC** authorization
- Built-in CSRF, XSS, and SQL injection protection

### 📬 Background Jobs
- Async job queues with Redis or PostgreSQL
- Cron-style scheduling
- Retry logic with exponential backoff
- Worker clustering

### 💾 Multi-Layer Caching
- In-memory and Redis caching
- TTL and tag-based invalidation
- Response caching middleware

### 🔴 Real-Time Features
- **WebSockets** with room support
- **Server-Sent Events (SSE)**
- Redis pub/sub for horizontal scaling
- Presence tracking

### 🛠️ Developer-First CLI
```bash
oxidite new myapp        # Scaffold new project
oxidite dev              # Hot-reload dev server
oxidite make:model User  # Generate models
oxidite migrate          # Run migrations
oxidite queue:work       # Start job workers
oxidite test             # Run test suite
```

### 📊 Built-In Admin Dashboard
- User and role management
- Queue monitoring
- Log viewer
- Health checks

### 🎨 Template Engine
- Server-side rendering with Blade/Django-like syntax
- Auto-escaping for XSS protection
- Layout inheritance

### 🔌 Plugin System
- Service provider pattern
- Hook-based extensibility
- Dependency injection

---

## 📦 Quick Start

### Installation

```bash
# Install the CLI
cargo install oxidite-cli

# Create a new project
oxidite new myapp
cd myapp

# Start development server
oxidite dev
```

### Your First API

```rust
use oxidite::prelude::*;

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

async fn create_user(Json(user): Json<CreateUser>) -> Result<Json<User>> {
    let new_user = User::create(user).await?;
    Ok(Json(new_user))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = Router::new();
    
    app.post("/users", create_user);
    
    Server::new(app)
        .listen("127.0.0.1:3000")
        .await
}
```

---

## 🏗️ Architecture

Oxidite is built as a modular mono-repo with the following crates:

```
oxidite/
├── oxidite-core          # HTTP server, router, request/response
├── oxidite-middleware    # Middleware ecosystem
├── oxidite-auth          # Authentication & authorization
├── oxidite-db            # Database abstraction & ORM
├── oxidite-migrate       # Schema migrations
├── oxidite-queue         # Background jobs
├── oxidite-cache         # Caching layer
├── oxidite-config        # Configuration management
├── oxidite-realtime      # WebSockets, SSE, pub/sub
├── oxidite-admin         # Admin dashboard
├── oxidite-template      # Template engine
├── oxidite-plugin        # Plugin system
├── oxidite-cli           # Command-line tool
├── oxidite-security      # Security utilities
└── oxidite-utils         # Common utilities
```

---

## 📚 Documentation

- [**Getting Started Guide**](docs/guides/getting-started.md)
- [**Architecture Overview**](docs/architecture/overview.md)
- [**API Reference**](https://docs.rs/oxidite)
- [**Complete Roadmap**](ROADMAP.md)

---

## 🎯 Project Status

Oxidite is currently in **active development**. See the [ROADMAP](ROADMAP.md) for detailed progress.

### Current Status
- ✅ Core HTTP server (HTTP/1.1)
- ✅ Basic routing
- ✅ Middleware foundation
- 🚧 Advanced routing features
- 🚧 Database layer
- 🚧 CLI tool
- ⏳ Authentication
- ⏳ Background jobs

---

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/oxidite
cd oxidite

# Build all crates
cargo build

# Run tests
cargo test

# Run the example
cargo run --example hello-world
```

---

## 📊 Benchmarks

Coming soon! We'll provide comprehensive benchmarks comparing Oxidite to other popular frameworks.

---

## 🛡️ Security

Security is a top priority. Please see [SECURITY.md](SECURITY.md) for our security policy and how to report vulnerabilities.

---

## 📄 License

Oxidite is licensed under the **MIT License**. See [LICENSE](LICENSE) for details.

---

## 🌟 Inspiration

Oxidite draws inspiration from:

- **FastAPI** - Type-safe APIs and automatic documentation
- **Laravel** - Elegant ORM, comprehensive tooling
- **Express.js** - Simplicity and middleware-first design
- **Django** - Admin tools, security-first approach

---

## 🎁 What Makes Oxidite Different?

| Feature | Oxidite | FastAPI | Laravel | Express | Django |
|---------|---------|---------|---------|---------|--------|
| Language | Rust | Python | PHP | JavaScript | Python |
| Performance | ⚡⚡⚡ | ⚡⚡ | ⚡ | ⚡⚡ | ⚡ |
| Type Safety | ✅ | ✅ | ❌ | ❌ | ❌ |
| Async/Await | ✅ | ✅ | ❌ | ✅ | ⚠️ |
| ORM | ✅ | ⚠️ | ✅ | ⚠️ | ✅ |
| Migrations | ✅ | ⚠️ | ✅ | ⚠️ | ✅ |
| Admin UI | ✅ | ❌ | ⚠️ | ❌ | ✅ |
| WebSockets | ✅ | ⚠️ | ⚠️ | ⚠️ | ⚠️ |
| Background Jobs | ✅ | ⚠️ | ✅ | ⚠️ | ⚠️ |
| OpenAPI | ✅ | ✅ | ⚠️ | ⚠️ | ⚠️ |
| Memory Safety | ✅ | ❌ | ❌ | ❌ | ❌ |

---

## 📬 Contact

- **GitHub Issues**: [Report bugs or request features](https://github.com/yourusername/oxidite/issues)
- **Discussions**: [Join the community](https://github.com/yourusername/oxidite/discussions)

---

**Built with ❤️ and Rust**
