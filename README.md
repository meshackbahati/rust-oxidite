# Oxidite Web Framework

<div align="center">

<img src="docs/logo/oxidite.svg" width="200" alt="Oxidite Logo">

A modern, high-performance web framework for Rust, inspired by FastAPI & Express.js.

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![GitHub](https://img.shields.io/badge/github-meshackbahati%2Frust--oxidite-black)](https://github.com/meshackbahati/rust-oxidite)
[![Stability](https://img.shields.io/badge/stability-beta-yellow.svg)](ROADMAP.md)

</div>

---

## What is Oxidite?

Oxidite is a modern, high-performance, developer-first web framework for Rust that provides a Rails-like, batteries-included experience. Built on top of the lightning-fast `hyper` and `tokio` asynchronous runtimes, it eliminates boilerplates by providing first-class identity & access management, a fully-featured custom ORM with async validation and eager-loading, a unified cloud/local storage manager, durable background queues, interactive REPL (`oxidite tinker`), hot-reload dev servers, and beautiful diagnostic pages. Oxidite is designed to provide maximum velocity and system-level performance, without compromising on developer ergonomics.

## Key Features

- **High Performance**: Built on `hyper` and `tokio` for blazing speed and high concurrency.
- **Advanced ORM**: Complete database layer with relationships, soft deletes, validation, and automated migrations.
- **Native Real-time**: First-class support for WebSockets and SSE with built-in connection orchestration.
- **Enterprise Security**: Built-in password hashing, JWT, OAuth2, 2FA, and granular RBAC/PBAC.
- **Developer Experience**: Powerful CLI for scaffolding, hot-reload development, and automated code generation.
- **Modular Ecosystem**: Optional crates for Caching, Queues, Email, Storage, and OpenAPI generation.

## Native WebSocket Support

Oxidite now features a native `WebSocketUpgrade` extractor that handles protocol handshakes automatically:

```rust
use oxidite::prelude::*;

async fn ws_handler(ws: WebSocketUpgrade) -> Result<Response> {
    Ok(ws.on_upgrade(|socket| async move {
        // Handle full-duplex communication here
    }))
}
```

## Installation

Install the `oxidite-cli` package to get started:

```bash
# Install from source
cargo install --path oxidite-cli
```

## Usage Example

```rust
use oxidite::prelude::*;
use serde_json::json;

async fn hello(_req: Request) -> Result<Response> {
    Ok(Response::json(json!({ "message": "Hello from Oxidite!" })))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    router.get("/", hello);

    Server::new(router)
        .listen("127.0.0.1:3000".parse()?)
        .await
}
```

## Architecture & Modular Ecosystem

Oxidite is architected as a collection of high-cohesion, low-coupling crates, allowing developers to opt-in to the specific capabilities they require:

| Crate                            | Purpose                                                 |
| -------------------------------- | ------------------------------------------------------- |
| **`oxidite`**            | Unified facade and development prelude.                 |
| **`oxidite-core`**       | High-performance HTTP kernel and routing engine.        |
| **`oxidite-macros`**     | Procedural macros for route and model derivation.       |
| **`oxidite-db`**         | Advanced ORM with migration and relationship support.   |
| **`oxidite-auth`**       | Identity and Access Management (JWT, OAuth2, RBAC).     |
| **`oxidite-realtime`**   | WebSocket, SSE, and event broadcasting orchestration.   |
| **`oxidite-middleware`** | Standard middleware suite (CORS, Logging, Compression). |
| **`oxidite-config`**     | Hierarchical configuration and environment management.  |
| **`oxidite-cache`**      | Multi-backend caching (Redis, In-memory).               |
| **`oxidite-queue`**      | Background job orchestration and task scheduling.       |
| **`oxidite-template`**   | Server-side rendering and static asset management.      |
| **`oxidite-mail`**       | SMTP and provider-based email delivery.                 |
| **`oxidite-storage`**    | Local and S3-compatible object storage abstractions.    |
| **`oxidite-security`**   | Cryptographic primitives and hardening utilities.       |
| **`oxidite-utils`**      | Common string, date, and validation helpers.            |
| **`oxidite-openapi`**    | Automated OpenAPI 3.0 schema generation.                |
| **`oxidite-graphql`**    | Type-safe GraphQL integration.                          |
| **`oxidite-testing`**    | Integrated testing and simulation utilities.            |
| **`oxidite-plugin`**     | Framework extension and lifecycle hook APIs.            |
| **`oxidite-cli`**        | The framework's primary command-line interface.         |

## 📊 Feature Comparison Matrix

Oxidite provides the most feature-rich, high-level developer experience in the Rust web ecosystem. Here is how it compares built-in capability-wise to other major frameworks:

| Feature                                |    Oxidite    |  Axum  |      Actix      |  Rocket  |    Loco    |  Poem  |  Salvo  |
| -------------------------------------- | :-----------: | :----: | :-------------: | :------: | :---------: | :----: | :------: |
| **Routing & Type-safe Handlers** |      ✅      |   ✅   |       ✅       |    ✅    |     ✅     |   ✅   |    ✅    |
| **Tower Middleware Support**     |      ✅      |   ✅   | ⚠️ Own system |    ❌    |     ✅     |   ✅   | ⚠️ Own |
| **Built-in ORM Engine**          |   ✅ Custom   | ❌ BYO |     ❌ BYO     |  ❌ BYO  |  ✅ SeaORM  | ❌ BYO |  ❌ BYO  |
| **Model Validation Rules**       |    ✅ Rich    | ❌ BYO |     ❌ BYO     | ✅ Basic | ⚠️ Manual | ❌ BYO |  ❌ BYO  |
| **ORM Relationships**            | ✅ Eager/Lazy |   ❌   |       ❌       |    ❌    |  ✅ SeaORM  |   ❌   |    ❌    |
| **Automated Migrations**         | ✅ Auto-Diff |   ❌   |       ❌       |    ❌    |     ❌     |   ❌   |    ❌    |
| **Interactive Console (Tinker)** |   ✅ Tinker   |   ❌   |       ❌       |    ❌    |    ⚠️    |   ❌   |    ❌    |
| **Unified Storage API**          |      ✅      |   ❌   |       ❌       |    ❌    |     ✅     |   ❌   |    ❌    |
| **Dev Diagnostics (Ignition)**   |    ✅ HTML    |   ❌   |       ❌       |    ❌    |    ⚠️    |   ❌   |    ❌    |
| **Auto OpenAPI Generation**      |      ✅      | ❌ BYO |     ❌ BYO     |    ❌    |     ❌     |   ✅   |    ✅    |

> ✅ = Built-in &nbsp; ⚠️ = Partial/limited &nbsp; ❌ = Not included (BYO = bring your own)

###  Our ORM Philosophy: Parity with SeaORM & Diesel

We believe developers shouldn't have to sacrifice ergonomics or safety when interacting with databases. Oxidite's ORM is designed to match the power, speed, and safety of the ecosystem's gold standards, **SeaORM** and **Diesel**:

- **Compile-Time Checks**: Through procedural macros and helper verification traits (`handler_fn`), we enforce valid model mappings and route bindings.
- **Eager Loading**: Prevent N+1 query patterns using our generated eager-loading methods (e.g. `eager_load_posts` and `eager_load_profile`), executing batched, optimized IN queries under the hood.
- **Auto-Diff Migrations**: Generate SQL migrations automatically by diffing your Rust structs directly against the database schema.

###  Benchmarks Notice

We don't believe in hype or synthetic bench-gaming. **We currently do not have verified public benchmarks.**
While Oxidite is built on top of the ultra-fast `hyper` and `tokio` runtimes, we plan to publish detailed throughput (RPS), latency profiles, and database resource overhead comparisons against Axum, Actix, and Loco in future updates using `criterion` and TechEmpower-style suites.

## Roadmap

See [ROADMAP.md](ROADMAP.md) for public progress and upcoming milestones as we move towards a stable 1.0 release.

## Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md).

## License

MIT License - see [LICENSE](LICENSE) for details.
