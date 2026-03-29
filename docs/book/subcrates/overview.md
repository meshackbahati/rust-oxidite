# Subcrate Reference Overview

This section documents each Oxidite crate, when to use it, and the primary API entry points.

## Core runtime crates

- `oxidite`: umbrella crate and prelude
- `oxidite-core`: router, request/response, extractors, server
- `oxidite-middleware`: reusable HTTP middleware layers
- `oxidite-config`: typed application configuration
- `oxidite-utils`: utility helpers (ids, strings, validation, dates)

## Data and state crates

- `oxidite-db`: ORM and database abstraction
- `oxidite-macros`: derive macros (especially `Model`)
- `oxidite-cache`: memory/redis caching abstractions
- `oxidite-queue`: in-memory/redis/postgres job queues
- `oxidite-storage`: local + S3 file storage

## Security and identity crates

- `oxidite-auth`: JWT, RBAC, sessions, OAuth helpers
- `oxidite-security`: crypto/hash/random/sanitization helpers

## Web/API feature crates

- `oxidite-realtime`: websocket/sse/pubsub/event helpers
- `oxidite-template`: SSR templates + static file serving
- `oxidite-openapi`: OpenAPI spec and docs generation
- `oxidite-graphql`: GraphQL schema/handler utilities
- `oxidite-mail`: SMTP + message/attachment APIs
- `oxidite-plugin`: plugin loading and lifecycle hooks

## Tooling crates

- `oxidite-cli`: project generation and developer commands
- `oxidite-testing`: test server/request/response helpers
