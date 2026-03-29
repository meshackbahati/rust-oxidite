# oxidite-db

Lightweight ORM and migration helpers for Oxidite, built on `sqlx::Any`.

## What this crate provides

- `DbPool` and `DbTransaction` wrappers for multi-backend SQL access.
- `#[derive(Model)]` CRUD generation via `oxidite-macros`.
- Relationship helpers: `HasMany`, `HasOne`, `BelongsTo`.
- File-based migrations with `MigrationManager`.
- Typed query ergonomics through `ModelQuery`.
- Strongly-typed ORM-side errors with `OrmError` for ergonomic APIs.
- Eager-loading helpers for has-many/has-one relations.

## Quick start

```rust
use oxidite_db::{DbPool, Model, Pagination, SortDirection, sqlx};

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

# async fn demo() -> Result<(), Box<dyn std::error::Error>> {
let db = DbPool::connect("sqlite::memory:").await?;

let active_users = User::query()
    .filter_like("email", "%@example.com")
    .order_by("id", SortDirection::Desc)
    .paginate(Pagination::from_page(1, 20)?)
    .fetch_all(&db)
    .await?;

let first = User::find_or_fail(&db, 1).await?;
let many = User::find_many(&db, &[1, 2, 3]).await?;

let _ = (active_users, first, many);
# Ok(())
# }
```

## Model derive notes

`#[derive(Model)]` expects a named struct with an `id: i64` field.

Supported model attributes:

- `#[model(table_name = "...")]`
- `#[model(table = "...")]` (alias)

Conventions:

- `created_at: i64` and `updated_at: i64` are auto-maintained when present.
- `deleted_at: Option<i64>` enables soft deletes.
- `#[validate(email)]` on `String` fields adds email validation.
- `save()` uses `is_persisted()` (derived models use `id > 0`).

## Transaction ergonomics

```rust
# use oxidite_db::DbPool;
# async fn tx(pool: &DbPool) -> Result<(), sqlx::Error> {
pool.with_transaction(|tx| async move {
    tx.execute("UPDATE users SET updated_at = strftime('%s','now')").await?;
    Ok(())
}).await?;
# Ok(())
# }
```

## Escape hatch: raw SQL remains first-class

All high-level APIs compose with raw SQL through `Database` methods:

- `execute(&str)`
- `query(&str)`
- `query_one(&str)`
- `execute_query(sqlx::query(...))`
- `fetch_all(sqlx::query(...))`
- `fetch_one(sqlx::query(...))`

## Query value support

`ModelQuery::filter_eq` supports common values including:

- integers and strings
- `bool` and `f64`
- `uuid::Uuid`
- `chrono::DateTime<Utc>`
- `serde_json::Value`
