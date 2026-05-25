# Oxidite Roadmap

This roadmap tracks technical implementation work in this repository.

## Current release target

- Framework release line: `2.2.1`
- Stability: `beta`
- Source of truth: this file (`ROADMAP.md`)

## Completed Enterprise Scalability (v2.3.0 Stabilization) — COMPLETE

- [X] **Expanded Handler Capacity**: Support for up to 12 extractors per handler.
- [X] **Global Router Layering**: Unified middleware application via `Router::layer()`.
- [X] **Authenticated WebSockets**: Context preservation during protocol upgrades.
- [X] **Raw SQL Access**: Public accessors for underlying `sqlx` pools.
- [X] **Universal Primary Keys**: `Uuid` and `String` support in `Model` derive.

## Completed modernization stream (v2.2.0 Hardening) — COMPLETE

- [X] **Advanced ORM Validation**: Async validations for Models (`length`, `range`, `email`, `url`, `regex`, `custom`, `unique`).
- [X] **N+1 Eager Loading**: Batch `IN` queries support in derive macros (`eager_load_posts`, `eager_load_profile`) alongside lazy-loading relations.
- [X] **Unified Cloud Storage**: Complete `StorageFacade` supporting Local, S3, Cloudinary, and ImageKit backends.
- [X] **Ignition-style Diagnostics**: Rich HTML trace pages for development-mode 500 exceptions.
- [X] **Interactive REPL (`oxidite tinker`)**: Full cargo-integrated interactive console CLI command.
- [X] **Compile-Time Router Verification**: Added `IntoHandler` trait and `handler_fn` route helper to verify extractors at compile time.
- [X] **State Injection DX**: Scaffolded controllers and generators to use `State<Arc<AppState>>` out of the box.

## Batch A (v1.1 carry-over)

- [X] WebSocket presence tracking (`oxidite-realtime`)
- [ ] Advanced monitoring and metrics (`oxidite-core`, `oxidite-middleware`, `oxidite-utils`)
- [ ] Performance profiling tools (`oxidite-cli`, `oxidite-testing`)
- [ ] Deployment guides (AWS/GCP/Azure docs)
- [ ] Migration guide from other frameworks (`docs/` + examples)

## Batch B (v2.1 performance/scalability) — COMPLETE

- [X] Zero-copy oriented request/response path improvements (`oxidite-core`)
- [X] Async streaming response support (`oxidite-core`)
- [X] Connection pooling optimizations (`oxidite-db`)
- [X] Database connection multiplexing patterns and transaction ergonomics (`oxidite-db`)
- [X] Enhanced testing framework APIs (`oxidite-testing`)
- [X] Mock server support (`oxidite-testing`)
- [X] Integration testing helpers (`oxidite-testing`, `oxidite-cli`)
- [X] Benchmark tooling baseline (`oxidite-testing` + bench utilities)

Batch B completion marker date: `2026-03-29`.

## Batch C (v3 engineering-only subset)

- [ ] API gateway functionality
- [ ] Audit logging
- [ ] Compliance report generation primitives
- [ ] Multi-region deployment tooling primitives
- [ ] Disaster recovery tooling primitives
