# Framework Guide: Building Real Applications with Oxidite

This guide is a practical map for building production services with Oxidite.

## How Oxidite is structured

At a high level:

1. `oxidite-core` handles HTTP primitives (request/response/router/server).
2. feature crates layer capabilities (db/auth/queue/realtime/cache/storage/etc).
3. `oxidite` umbrella crate re-exports these capabilities behind feature flags.

## Typical project structure

```text
src/
  main.rs
  routes/
  handlers/
  models/
  services/
  middleware/
  jobs/
```

Recommended ownership:

- handlers: HTTP boundary only
- services: business logic
- models/repositories: persistence logic
- jobs: async/background flows

## Request lifecycle

1. Router matches method + path.
2. Middleware stack runs (request ID, auth, rate limit, etc).
3. Extractors parse input (`Path`, `Query`, `Json`, `State`, `Cookies`, `Form`).
4. Handler executes business logic.
5. Response is serialized and returned.

## Error handling strategy

Use typed errors per domain and map them at the HTTP boundary.

- validation -> `400`
- auth errors -> `401`/`403`
- missing resources -> `404`
- conflicts -> `409`
- internal failures -> `500`

Prefer explicit error enums instead of stringly-typed errors.

## Data access strategy

Use `oxidite-db` with three tiers:

1. basic CRUD via `Model` derive
2. typed query composition via `ModelQuery`
3. raw SQL for advanced joins/analytics/hot paths

## Security baseline checklist

- hash passwords with `oxidite-auth` helpers
- validate and sanitize untrusted input (`oxidite-security`)
- apply rate limiting middleware
- enforce RBAC/PBAC checks in handlers/services
- keep secrets in config/env, not code

## Observability baseline checklist

- request IDs on all incoming requests
- structured logs at handler/service boundaries
- latency and error counters per route/domain
- retry/dead-letter metrics for async workers

## Testing strategy

- unit tests for pure business logic
- handler tests with `oxidite-testing` test server/request/response
- integration tests for migrations + DB transactions
- contract tests for public API payloads

## Performance strategy

- cache expensive read endpoints
- paginate list endpoints
- stream large responses where useful
- avoid N+1 query patterns (use eager loading)
- benchmark hot endpoints before/after changes

## Deployment strategy

- ship behind health checks
- use staged rollout (canary/weighted)
- preserve rollback path for each release
- run schema changes with backward compatibility windows
