# Database Guide

## Overview

Oxidite provides a database abstraction layer built on top of SQLx, supporting PostgreSQL, MySQL, and SQLite.

## Setup

### Add Dependencies

```toml
[dependencies]
oxidite-db = "0.1"
tokio = { version = "1", features = ["full"] }
```

### Connection

```rust
use oxidite_db::{DbPool, Database};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // PostgreSQL
    let db = DbPool::connect("postgres://user:pass@localhost/mydb").await?;
    
    // MySQL
    // let db = DbPool::connect("mysql://user:pass@localhost/mydb").await?;
    
    // SQLITE
    // let db = DbPool::connect("sqlite://data.db").await?;
    
    // Test connection
    db.ping().await?;
    
    Ok(())
}
```

## Executing Queries

### Execute (INSERT, UPDATE, DELETE)

```rust
// Insert
let rows = db.execute(
    "INSERT INTO users (name, email) VALUES ('Alice', 'alice@example.com')"
).await?;

// Update
let rows = db.execute(
    "UPDATE users SET name = 'Bob' WHERE id = 1"
).await?;

// Delete
let rows = db.execute(
    "DELETE FROM users WHERE id = 1"
).await?;
```

### Query Multiple Rows

```rust
use sqlx::Row;

let rows = db.query("SELECT * FROM users").await?;

for row in rows {
    let id: i32 = row.try_get("id")?;
    let name: String = row.try_get("name")?;
    println!("User {}: {}", id, name);
}
```

### Query One Row

```rust
if let Some(row) = db.query_one("SELECT * FROM users WHERE id = 1").await? {
    let name: String = row.try_get("name")?;
    println!("User name: {}", name);
}
```

## Query Builder

Use the query builder for dynamic queries:

```rust
use oxidite_db::QueryBuilder;

let query = QueryBuilder::new("users")
    .select(&["id", "name", "email"])
    .where_eq("status", "active")
    .order_by("created_at", "DESC")
    .limit(10)
    .offset(0)
    .build();

let rows = db.query(&query).await?;
```

## Transactions

Transactions ensure atomicity - either all operations succeed or none:

```rust
use oxidite_db::Database;

async fn transfer_funds(db: &DbPool, from: i32, to: i32, amount: f64) -> Result<(), sqlx::Error> {
    let mut tx = db.begin_transaction().await?;
    
    // Debit from account
    tx.execute(&format!(
        "UPDATE accounts SET balance = balance - {} WHERE id = {}",
        amount, from
    )).await?;
    
    // Credit to account
    tx.execute(&format!(
        "UPDATE accounts SET balance = balance + {} WHERE id = {}",
        amount, to
    )).await?;
    
    // Commit transaction (or rollback on error)
    tx.commit().await?;
    
    Ok(())
}
```

### Transaction Rollback

```rust
let mut tx = db.begin_transaction().await?;

match process_order(&mut tx).await {
    Ok(_) => tx.commit().await?,
    Err(e) => {
        tx.rollback().await?;
        return Err(e);
    }
}
```

Transactions automatically rollback if dropped without calling `commit()`.

## Connection Pooling

Oxidite uses connection pooling by default:

```rust
// Default: 5 connections (1 for SQLite in-memory)
let db = DbPool::connect(url).await?;
```

## Migrations

> **Coming Soon**: Built-in migration support is planned for a future release.

For now, use [SQLx CLI](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli):

```bash
# Install
cargo install sqlx-cli

# Create migration
sqlx migrate add create_users_table

# Run migrations
sqlx migrate run --database-url postgres://localhost/mydb
```

## Best Practices

1. **Use transactions** - For operations that must be atomic
2. **Prepared statements** - SQLx uses prepared statements by default
3. **Connection pooling** - Reuse connections for better performance
4. **Error handling** - Always handle database errors gracefully
5. **Type safety** - Use `Row::try_get()` for safe column access

## Next Steps

- [Authentication Guide](authentication.md) - Add user authentication
- [Realtime Guide](realtime.md) - WebSockets and SSE
