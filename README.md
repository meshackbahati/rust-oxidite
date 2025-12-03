# Oxidite

Oxidite is a batteries-included Rust backend web framework inspired by FastAPI, Laravel, Express, and Django.
It provides:

* High-performance HTTP server supporting all HTTP versions (1.0, 1.1, 2, 3)
* Advanced routing and typed request/response handling
* Middleware system
* CLI tooling for scaffolding, testing, linting, migrations
* Alembic-style models with SQL + NoSQL support
* Auth and security (JWT, sessions, role-based access)
* CORS and protocol policies
* Background job queues, caching, and configuration system
* Real-time support (WebSockets, SSE, pub/sub)
* Logging, observability, and plugin architecture
* Developer-first features (linting, testing, scaffolding)
* Extensible for future AI/analytics modules

## 🎯 Objectives

* Build a full-featured, production-ready Rust web framework
* Ensure developer ergonomics: easy-to-use CLI, scaffolding, and modular API
* Include all major features inspired by FastAPI, Laravel, Express, and Django
* Provide security-first defaults for backend developers
* Support multiple databases: SQL (Postgres, MySQL, SQLite) and NoSQL (MongoDB, Redis)
* Full support for CORS, protocol headers, HTTPS, and all HTTP versions
* Provide extensible architecture for plugins and future modules

## Repo Structure
```
rust-oxidite/
  oxidite-core/          # Router, extractors, HTTP handling
  oxidite-middleware/    # Middleware pipeline
  oxidite-auth/          # JWT, sessions, role-based access
  oxidite-db/            # SQL + NoSQL models, Alembic-style migrations
  oxidite-cli/           # Scaffolding, dev server, lint, test
  oxidite-queue/         # Background jobs, scheduling
  oxidite-cache/         # In-memory & Redis caching
  oxidite-config/        # Environment & config system
  oxidite-realtime/      # WebSockets, SSE, pub/sub
  oxidite-security/      # Rate limiting, CORS, headers, CSRF
  oxidite-utils/         # Logging, tracing, helpers
  examples/            # Sample apps
  docs/
    architecture/
    guides/
    reference/
```
