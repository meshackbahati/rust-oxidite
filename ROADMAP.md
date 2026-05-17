# Oxidite Roadmap

This roadmap tracks technical implementation work in this repository.

## Current release target

- Framework release line: `2.2.0`
- Stability: `beta`
- Source of truth: this file (`ROADMAP.md`)

## Completed modernization stream (v2.2.0 Hardening) — COMPLETE

- [x] **Advanced ORM Validation**: Async validations for Models (`length`, `range`, `email`, `url`, `regex`, `custom`, `unique`).
- [x] **N+1 Eager Loading**: Batch `IN` queries support in derive macros (`eager_load_posts`, `eager_load_profile`) alongside lazy-loading relations.
- [x] **Unified Cloud Storage**: Complete `StorageFacade` supporting Local, S3, Cloudinary, and ImageKit backends.
- [x] **Ignition-style Diagnostics**: Rich HTML trace pages for development-mode 500 exceptions.
- [x] **Interactive REPL (`oxidite tinker`)**: Full cargo-integrated interactive console CLI command.
- [x] **Compile-Time Router Verification**: Added `IntoHandler` trait and `handler_fn` route helper to verify extractors at compile time.
- [x] **State Injection DX**: Scaffolded controllers and generators to use `State<Arc<AppState>>` out of the box.

## Batch A (v1.1 carry-over)

- [x] WebSocket presence tracking (`oxidite-realtime`)
- [ ] Advanced monitoring and metrics (`oxidite-core`, `oxidite-middleware`, `oxidite-utils`)
- [ ] Performance profiling tools (`oxidite-cli`, `oxidite-testing`)
- [ ] Deployment guides (AWS/GCP/Azure docs)
- [ ] Migration guide from other frameworks (`docs/` + examples)

## Batch B (v2.1 performance/scalability) — COMPLETE

- [x] Zero-copy oriented request/response path improvements (`oxidite-core`)
- [x] Async streaming response support (`oxidite-core`)
- [x] Connection pooling optimizations (`oxidite-db`)
- [x] Database connection multiplexing patterns and transaction ergonomics (`oxidite-db`)
- [x] Enhanced testing framework APIs (`oxidite-testing`)
- [x] Mock server support (`oxidite-testing`)
- [x] Integration testing helpers (`oxidite-testing`, `oxidite-cli`)
- [x] Benchmark tooling baseline (`oxidite-testing` + bench utilities)

Batch B completion marker date: `2026-03-29`.

## Batch C (v3 engineering-only subset)

- [ ] API gateway functionality
- [ ] Audit logging
- [ ] Compliance report generation primitives
- [ ] Multi-region deployment tooling primitives
- [ ] Disaster recovery tooling primitives

## Not executable only in-code

- Developer advocacy program
- Conference talks/workshops
- Online courses/training
- Marketplace/partner/certification programs
- Enterprise support/commercial operations
- Revenue and market-share outcomes

## Execution policy

- Ship crate-by-crate with tests and docs
- Preserve backward compatibility where practical
- Keep raw SQL escape hatches first-class
- Prefer typed, Rust-idiomatic APIs over hidden magic
