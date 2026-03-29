# Handler and Service Patterns

This chapter shows patterns that keep Oxidite apps maintainable at scale.

## Thin handlers, thick services

Handler responsibilities:

- parse input
- call service
- map domain result to HTTP response

Service responsibilities:

- validate business rules
- coordinate repositories/external calls
- return typed domain errors

## Pattern: command/query split

Use separate methods for:

- command paths (writes)
- query paths (reads)

Benefits:

- clearer performance tuning
- easier authorization policies
- simpler testing

## Pattern: explicit transactions

For multi-step writes:

1. open transaction
2. perform all related changes
3. commit only on full success
4. rollback on any failure

Use `DbPool::with_transaction` for concise transaction boundaries.

## Pattern: pagination first

All list endpoints should accept:

- page/per_page or limit/offset
- deterministic sort order

Use `Pagination::from_page(...)` + `order_by(...)` for stable paging.

## Pattern: idempotent writes

For retry-prone endpoints/jobs:

- accept idempotency key
- persist dedup marker in DB
- return prior result on duplicate key

## Pattern: explicit authorization

Run authorization checks close to business decisions.

- route-level guards for broad access
- service-level checks for resource ownership rules

## Pattern: consistent response envelopes

Adopt stable JSON envelopes:

- success: data + metadata
- error: code + message + details

This simplifies frontend and monitoring integration.
