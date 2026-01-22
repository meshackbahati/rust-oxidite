# oxidite-db

Database ORM with relationships and migrations for Oxidite.

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/oxidite-db.svg)](https://crates.io/crates/oxidite-db)
[![Docs.rs](https://docs.rs/oxidite-db/badge.svg)](https://docs.rs/oxidite-db)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

</div>

## Overview

`oxidite-db` provides a powerful and intuitive Object-Relational Mapping (ORM) system for Rust. It includes model derivation, relationship management, migrations, and query building capabilities to make database interactions simple and type-safe.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
oxidite-db = "0.1"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

## Features

- **Model derive macro** - Automatic CRUD operations with the `#[derive(Model)]` macro
- **Relationships** - Support for HasOne, HasMany, and BelongsTo relationships
- **Migrations** - Database schema evolution with rollback support
- **Soft deletes** - Logical deletion with automatic filtering
- **Timestamps** - Automatic created_at and updated_at tracking
- **Field validation** - Built-in validation for model fields
- **Transaction support** - ACID-compliant transaction handling
- **Multi-database support** - Works with PostgreSQL, MySQL, and SQLite

## Usage

### Defining Models

Define your database models using the `#[derive(Model)]` macro:

```rust
use oxidite_db::{Model, Database};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Model, Serialize, Deserialize, Clone)]
#[table_name = "users"]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Database Connection

Establish a connection to your database:

```rust
use oxidite_db::DbPool;

// Connect to PostgreSQL
let pool = DbPool::connect("postgresql://user:password@localhost/database").await?;

// Connect to SQLite
let pool = DbPool::connect("sqlite::memory:").await?;

// Connect to MySQL
let pool = DbPool::connect("mysql://user:password@localhost/database").await?;
```

### CRUD Operations

Perform Create, Read, Update, and Delete operations:

```rust
// Create a new record
let mut user = User {
    id: 0, // Will be set by database
    name: "John Doe".to_string(),
    email: "john@example.com".to_string(),
    created_at: Utc::now(),
    updated_at: Utc::now(),
};

user.create(&pool).await?; // User now has an assigned ID

// Read records
let all_users = User::all(&pool).await?;
let specific_user = User::find(&pool, 1).await?;

// Update a record
user.name = "Jane Doe".to_string();
user.update(&pool).await?;

// Delete a record
user.delete(&pool).await?;
```

### Query Building

Build complex queries with the fluent interface:

```rust
// Find with conditions
let users = User::where_eq(&pool, "email", "john@example.com")
    .order_by("created_at", "DESC")
    .limit(10)
    .get()
    .await?;

// Count records
let count = User::count(&pool).await?;

// Find first matching record
let user = User::where_eq(&pool, "name", "John")
    .first()
    .await?;
```

### Relationships

Define and use relationships between models:

```rust
#[derive(Model, Serialize, Deserialize, Clone)]
#[table_name = "posts"]
pub struct Post {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Get user's posts
let user_posts = user.has_many::<Post>(&pool, "user_id").await?;

// Get post's author
let author = post.belongs_to::<User>(&pool, "user_id", "id").await?;
```

### Migrations

Manage your database schema with migrations:

```rust
use oxidite_db::{Migration, DbPool};

struct CreateUsersTable;

impl Migration for CreateUsersTable {
    fn up(&self) -> String {
        "CREATE TABLE users (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            email VARCHAR(255) UNIQUE NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
        )".to_string()
    }
    
    fn down(&self) -> String {
        "DROP TABLE users".to_string()
    }
}

// Run migrations
CreateUsersTable.run_up(&pool).await?;
```

### Transactions

Use transactions for atomic database operations:

```rust
use oxidite_db::Transaction;

let tx = pool.begin_transaction().await?;

// Perform operations within transaction
let mut user = User { /* ... */ };
user.create(&tx).await?;

let mut post = Post { /* ... */ };
post.create(&tx).await?;

// Commit the transaction
tx.commit().await?;

// Or rollback if something goes wrong
// tx.rollback().await?;
```

## License

MIT
