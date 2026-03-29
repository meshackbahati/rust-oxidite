# Migration and Adoption Guide

This chapter explains how to adopt Oxidite in existing systems without risky rewrites.

## Adoption principles

- Prefer incremental migration over big-bang rewrites.
- Keep external API contracts stable while internals change.
- Preserve existing database schema first; refactor schema later.
- Keep raw SQL for critical paths where planner behavior matters.
- Add observability before moving production traffic.

## Common migration patterns

## 1. Strangler Pattern (recommended)

Use a reverse proxy and route selected endpoints to Oxidite first.

1. Keep current app as primary.
2. Introduce Oxidite service behind the same domain.
3. Move low-risk read endpoints first.
4. Move write endpoints after parity tests.
5. Decommission old routes gradually.

## 2. Domain-by-domain cutover

Move one domain at a time:

- auth
- users/profile
- feed/content
- payments
- notifications

This reduces blast radius and speeds rollback.

## 3. Data-first migration

When schema compatibility is the hardest part:

1. connect Oxidite to the existing database
2. port models with `#[derive(Model)]`
3. keep complex SQL via raw query escape hatch
4. replace ORM paths incrementally

## Compatibility checklist

Before moving traffic:

- response JSON shape parity
- error status + message parity
- auth/session behavior parity
- idempotency parity for retries
- latency/error-rate baseline parity

## Session and auth compatibility

When migrating from session-heavy systems:

- keep cookie names/flags (`Secure`, `HttpOnly`, `SameSite`) unchanged during transition
- keep redis key prefixes and TTL policy stable
- verify logout and token/session revocation paths

## Realtime compatibility

When clients depend on stable event contracts:

- freeze room naming
- freeze event names
- freeze payload fields
- use an adapter bridge until all producers are migrated

## Background jobs and events

- design workers as at-least-once consumers
- enforce idempotency at database boundary
- commit message offsets only after durable side effects
- dead-letter poison messages with context

## Production rollout playbook

1. Shadow read traffic
2. Dual-write or compare mode (where safe)
3. Weighted traffic shifting (1% -> 10% -> 25% -> 50% -> 100%)
4. Hold periods with SLO checks between each stage
5. Keep one-click rollback path

## Signals you are ready for full cutover

- parity test suite is green across auth/data/realtime paths
- p95 and p99 latency are stable or improved
- no spike in 4xx/5xx error rates
- no data integrity drift in reconciliation checks
