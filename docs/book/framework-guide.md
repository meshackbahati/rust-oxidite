# Framework Guide: Building Real Applications with Oxidite (v2.3.1)

This guide is a practical map for building production services with Oxidite.

## How Oxidite is structured

At a high level:

1. `oxidite-core` handles HTTP primitives (request/response/router/server).
2. Feature crates layer capabilities (db/auth/queue/realtime/cache/storage/etc).
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

## Configuration

Oxidite uses `oxidite.toml` as its primary config file, with optional `.env` support for secrets and per-developer overrides.

### oxidite.toml

```toml
[server]
host = "127.0.0.1"
port = 3000

[database]
url = "postgres://localhost/myapp_dev"

[env]
WATU_API_KEY = "sk-abc123"
STRIPE_SECRET = "whsec_test"
```

The `[env]` table injects variables into the process environment so `std::env::var("WATU_API_KEY")` works anywhere in your code.

### Loading config in code

```rust
use oxidite::prelude::*;
use oxidite_config::Config;

let config = Config::load()?;
let db_url = &config.database.url;
let port = config.server.port;
let api_key = std::env::var("WATU_API_KEY").ok();
```

### Override precedence (highest wins)

1. OS environment variables (exported in shell)
2. `.env` file values
3. `oxidite.toml [env]` table
4. `oxidite.toml` section defaults
5. Hardcoded defaults

Blank `.env` entries (`KEY=`) are treated as unset, so the `[env]` fallback applies. Set `OXIDITE_SKIP_DOTENV=1` to skip `.env` loading entirely.

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
