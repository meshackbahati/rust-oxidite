# High-Throughput Postgres Analytics Patterns

Use this for leaderboard and heavy reporting endpoints.

## Principles

- Keep hot paths in raw SQL if query planner quality matters.
- Minimize allocations in response shaping.
- Use explicit projections; avoid `SELECT *`.

## Query shape recommendations

- Pre-aggregate with CTEs when combining solves/rank windows.
- Use covering indexes for filter + order columns.
- Use keyset pagination for deep pages.

## Oxidite execution path

- Use `oxidite-db` query APIs for direct SQL execution.
- Map result rows into typed response structs.
- Add route-level timing + rows-scanned metrics.

## Leaderboard example checklist

- index on `(event_id, score DESC, updated_at DESC)`
- index on submission facts `(event_id, user_id, solved_at)`
- immutable event snapshots where possible

## Validation gates

- `EXPLAIN (ANALYZE, BUFFERS)` baseline before migration
- p95 latency comparison under representative load
- correctness checks on tie-break rules
