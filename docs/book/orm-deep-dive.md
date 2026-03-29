# ORM Deep Dive

This chapter covers model design, query ergonomics, and practical escape hatches.

## Model Conventions

A typical model uses:

- `id: i64` primary key
- optional `created_at: i64`, `updated_at: i64`
- optional `deleted_at: Option<i64>` for soft deletes

```rust,ignore
use oxidite_db::{Model, sqlx};

#[derive(Model, sqlx::FromRow)]
#[model(table = "users")]
struct User {
    id: i64,
    name: String,
    email: String,
    created_at: i64,
    updated_at: i64,
    deleted_at: Option<i64>,
}
```

## Query API

Use `ModelQuery` for common cases:

- `filter_eq`, `filter_like`, `filter_is_null`, `filter_is_not_null`
- `order_by` + `SortDirection`
- `paginate(Pagination)`
- `with_deleted()` for soft-deleted records

For advanced DB-specific behavior, use raw SQL with bound parameters.

## Save Semantics

`Model::save()` delegates to `is_persisted()`:

- `true` => `update`
- `false` => `create`

Derived models implement `is_persisted()` as `id > 0` by default. Override when needed.

## Batch Operations

Use trait helpers for simple batches:

- `insert_many`
- `update_many`

For high-volume workloads, use explicit transaction + raw SQL/bulk SQL patterns.

## Error Handling

- `Result<T>` for SQL-layer operations (`sqlx::Error`)
- `OrmResult<T>` for ORM-layer typed errors (`OrmError`)

Prefer `OrmResult` at user-facing API boundaries where diagnostics matter.
