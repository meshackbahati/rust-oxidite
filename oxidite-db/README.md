# oxidite-db

Lightweight ORM and migration helpers for Oxidite, built on `sqlx::Any` with concrete pool escape hatches.

## What this crate provides

- `DbPool` and `DbTransaction` wrappers for multi-backend SQL access.
- **Concrete pool access** for PostgreSQL/MySQL/SQLite-specific features (see below).
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

## Concrete Pool Access (Escape Hatches)

**v2.3.1+**: `DbPool` now stores concrete connection pools alongside the `AnyPool` abstraction, enabling use of PostgreSQL/MySQL/SQLite-specific features that the `Any` driver doesn't support.

### Why This Matters

The `Any` database abstraction is great for write-once-run-anywhere code, but it has limitations:

- ❌ Doesn't support `#[derive(sqlx::FromRow)]` for complex types (JSONB, arrays, custom types)
- ❌ No PostgreSQL-specific operators (JSONB queries, array operations, etc.)
- ❌ Limited type mapping for database-specific features

**Solution**: Access the concrete pool directly when you need database-specific features.

### API

```rust
use oxidite_db::DbPool;

// Get the concrete pool for your database
let pg_pool = db.postgres_pool().expect("PostgreSQL required");
let mysql_pool = db.mysql_pool();  // Option<&MySqlPool>
let sqlite_pool = db.sqlite_pool();  // Option<&SqlitePool>
```

### Using PostgreSQL Pool with `query_as`

```rust
use oxidite::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(FromRow, Serialize, Deserialize)]
struct AdminAuditLog {
    id: i64,
    admin_id: i64,
    action: String,
    target_type: String,
    old_values: Option<String>,  // JSON stored as string
    new_values: Option<String>,
    reason: Option<String>,
    created_at: i64,
}

// In your handler:
async fn get_audit_logs(db: &DbPool) -> Result<Vec<AdminAuditLog>> {
    let pg_pool = db.postgres_pool()
        .ok_or(Error::InternalServerError("PostgreSQL required".into()))?;
    
    sqlx::query_as::<_, AdminAuditLog>(
        "SELECT * FROM admin_audit_logs ORDER BY created_at DESC LIMIT $1 OFFSET $2"
    )
    .bind(20i64)
    .bind(0i64)
    .fetch_all(pg_pool)
    .await
}
```

### Convenience Methods

For common patterns, `DbPool` provides typed query helpers:

```rust
use oxidite::prelude::*;

// fetch_all_as - fetch all rows as typed struct
let users: Vec<User> = db.fetch_all_as(
    "SELECT * FROM users WHERE status = $1",
    |q| q.bind("active")
).await?;

// fetch_optional_as - fetch one row or None
let user: Option<User> = db.fetch_optional_as(
    "SELECT * FROM users WHERE email = $1",
    |q| q.bind("user@example.com")
).await?;

// fetch_one_as - fetch one row or error
let user: User = db.fetch_one_as(
    "SELECT * FROM users WHERE id = $1",
    |q| q.bind(123i64)
).await?;
```

### Trade-offs

**Connection Usage**: When using PostgreSQL, `DbPool` creates two connection pools:
1. `AnyPool` - for backward compatibility with the `Database` trait
2. `PgPool` - for escape hatch access

This doubles the connection count but enables type-safe queries while maintaining API compatibility. Configure via `PoolOptions::max_connections`.

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

**New**: Use concrete pools for `query_as` with `#[derive(FromRow)]` models that use database-specific types.

## Query value support

`ModelQuery::filter_eq` supports common values including:

- integers and strings
- `bool` and `f64`
- `uuid::Uuid`
- `chrono::DateTime<Utc>`
- `serde_json::Value`
