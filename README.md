# Oxidite Web Framework

<div align="center">

<img src="docs/logo/oxidite.svg" width="200" alt="Oxidite Logo">

A modern, high-performance web framework for Rust, inspired by FastAPI and Express.js.

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![GitHub](https://img.shields.io/badge/github-Kyle6012%2Frust--oxidite-black)](https://github.com/Kyle6012/rust-oxidite)
[![Documentation](https://docs.rs/oxidite/badge.svg)](https://docs.rs/oxidite)

Built with ❤️ by [Meshack Bahati Ouma](https://github.com/Kyle6012)

</div>

---

## 🚀 What is Oxidite?

Oxidite is a batteries-included web framework that combines Rust's performance with developer-friendly APIs inspired by Django, Rails, and FastAPI. Build scalable web applications with confidence.

##  Features

### Core Framework
- ⚡ **High Performance** - Built on `hyper` and `tokio`
- 🛣️ **Expressive Routing** - Path parameters, regex routes, API versioning
- 🔐 **Built-in Security** - CORS, CSRF, encryption, sanitization
- 📦 **Middleware Stack** - Logger, compression, rate limiting, timeouts

### Data & Persistence
- 🗄️ **Database** - PostgreSQL, MySQL, SQLite with query builder
- 💾 **Caching** - Memory and Redis with TTL support
- 📁 **Storage** - Local filesystem and S3-compatible cloud storage
- 🔄 **Migrations** - Database migration management

### Real-time & Communication
- 🌐 **WebSockets** - Full WebSocket support with rooms
- 📡 **Server-Sent Events** - Real-time push updates
- 📬 **Pub/Sub** - Internal event messaging
- 📧 **Email** - SMTP with HTML templates and attachments

### Authentication & Authorization
- 🔑 **JWT** - Stateless authentication
- 🎫 **Sessions** - Memory and Redis-backed sessions
- 🔐 **OAuth2** - Google, GitHub, Microsoft providers
- 👤 **Password Hashing** - Argon2 secure hashing

### Developer Experience
- 🎨 **Template Engine** - Django-style templates with inheritance
- 🛠️ **CLI Tools** - Project scaffolding, migrations, code generation
- 📚 **Comprehensive Docs** - Guides, examples, API reference
- 🧰 **Utilities** - Date, ID generation, validation, string manipulation

## 📦 Quick Start

```bash
# Install CLI from local source (recommended for development)
cargo install --path oxidite-cli

# Create new project (interactive)
oxidite new my-app
cd my-app

# Run development server with hot reload
oxidite dev
```

Visit `http://localhost:3000`

## 🎮 Running the Demo App

To see Oxidite in action with a full-featured example:

```bash
cd examples/demo-app
cargo run
```

Visit `http://localhost:3000` to explore the demo application featuring:
- Authentication (Login/Register)
- Database interactions
- Real-time chat
- Template rendering

## 💡 Example

```rust
use oxidite_core::{Router, Server, Request, Response};

async fn hello(_req: Request) -> Result<Response, oxidite_core::Error> {
    Ok(Response::json(serde_json::json!({
        "message": "Hello, Oxidite!"
    })))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut router = Router::new();
    router.get("/api/hello", hello);
    
    let server = Server::new("127.0.0.1:8080".parse()?, router);
    println!("🚀 Server running on http://127.0.0.1:8080");
    server.run().await?;
    
    Ok(())
}
```

## 📖 Documentation

- [Getting Started](docs/guides/getting-started.md)
- [Database Guide](docs/guides/database.md)
- [Authentication Guide](docs/guides/authentication.md)
- [Templating Guide](docs/guides/templating.md)
- [Realtime Guide](docs/guides/realtime.md)

## 🏗️ Architecture

Oxidite is composed of 14 modular crates:

| Crate | Description |
|-------|-------------|
| `oxidite-core` | HTTP server, routing, versioning |
| `oxidite-auth` | JWT, sessions, OAuth2 |
| `oxidite-db` | Database abstraction, migrations |
| `oxidite-middleware` | CORS, CSRF, rate limiting, timeouts |
| `oxidite-template` | Template engine with inheritance |
| `oxidite-realtime` | WebSockets, SSE, Pub/Sub |
| `oxidite-storage` | File storage (Local/S3) |
| `oxidite-mail` | Email with SMTP |
| `oxidite-queue` | Background job processing |
| `oxidite-cache` | Memory and Redis caching |
| `oxidite-config` | Configuration management |
| `oxidite-security` | Encryption, hashing, sanitization |
| `oxidite-utils` | Common utilities |
| `oxidite-cli` | Command-line tools |

## 🤝 Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md).

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 👤 Author

**Meshack Bahati Ouma**
- GitHub: [@Kyle6012](https://github.com/Kyle6012)
- Email: bahatikylemeshack@gmail.com

## 🌟 Show Your Support

Give a ⭐️ if this project helped you!

---

<div align="center">

Made with ❤️ using Rust

</div>
