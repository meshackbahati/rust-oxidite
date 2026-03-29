# Oxidite Roadmap

This roadmap tracks technical implementation work in this repository.

## Current release target

- Framework release line: `2.1.0`
- Stability: `beta`
- Source of truth: this file (`ROADMAP.md`)

## Completed modernization stream

- ORM ergonomics and typed errors (`oxidite-db`, `oxidite-macros`)
- Safer query APIs, pagination helpers, and relation eager loading
- Typed migration errors + backend-specific migration table SQL
- CLI hardening for model/migration/seed flows
- Shared SQL script executor in CLI commands
- CLI integration tests for project generation and generators
- Static HTML docs deployment pipeline to `doc/book/book`
- Beta-facing project messaging and roadmap-first README badges

## Batch A (v1.1 carry-over)

- [ ] WebSocket presence tracking (`oxidite-realtime`)
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
