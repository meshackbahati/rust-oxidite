# Architecture

This chapter explains how Oxidite is split across crates and how requests move through the system.

## Workspace Structure

- `oxidite`: top-level facade and feature flags.
- `oxidite-core`: router, request/response, server primitives.
- `oxidite-middleware`: common cross-cutting layers.
- `oxidite-db` + `oxidite-macros`: ORM, derive macros, migrations.
- `oxidite-auth`, `oxidite-cache`, `oxidite-queue`, `oxidite-realtime`, `oxidite-template`: batteries-included runtime capabilities.
- `oxidite-cli`: scaffolding, migration, and developer workflow tooling.

## Request Lifecycle

1. The server accepts an HTTP request in `oxidite-core`.
2. The router matches method/path and prepares extractors.
3. Middleware chain runs pre-handler logic.
4. Handler executes with typed extractors.
5. Handler returns typed response.
6. Middleware chain runs post-handler logic.
7. Response is serialized and returned to the client.

## Database Layer Design

Oxidite ORM sits on top of `sqlx::Any`:

- `Database` trait abstracts pool/transaction execution.
- `Model` trait provides typed CRUD and validation hooks.
- `ModelQuery` offers builder ergonomics.
- Relationship helpers (`HasMany`, `HasOne`, `BelongsTo`) keep joins and loading explicit.
- Raw SQL remains first-class through `execute_query`/`fetch_all`/`fetch_one`.

## Extension Strategy

Prefer adding capabilities in dedicated crates and surfacing stable public APIs through `oxidite`.

This keeps compile times predictable and avoids making core crates monolithic.
