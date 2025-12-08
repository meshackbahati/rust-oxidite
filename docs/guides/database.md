# Database & ORM Guide

Complete guide to using the Oxidite ORM for database operations.

## Installation

```toml
[dependencies]
oxidite = { version = "1.0", features = ["database"] }
```

## Setup

### Database Connection

Create `.env` file:

```env
DATABASE_URL=postgresql://user:password@localhost/mydb
```

Connect in your app (`src/main.rs`):

```rust
use oxidite::prelude::*;
use oxidite_db::Database;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    db: Arc<Database>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load config
    dotenv::dotenv().ok();
    
    // Connect to database
    let db = Arc::new(Database::connect(&std::env::var("DATABASE_URL")?).await?);

    let state = AppState { db };
    
    // ... your app setup
    let app = Router::new().with_state(state);
    // ...
    Ok(())
}
```

## Defining Models

```rust
use oxidite_db::Model;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Model, Serialize, Deserialize, Clone, Default)]
#[table_name = "users"]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

## CRUD Operations

### Create

```rust
let mut user = User {
    name: "Alice".to_string(),
    email: "alice@example.com".to_string(),
    password_hash: hash_password("password")?,
    ..Default::default()
};

user.save(&db).await?;
```

### Read

```rust
// Find by ID
let user = User::find(1, &db).await?;

// Find all
let users = User::all(&db).await?;

// Where clause
let user = User::query()
    .where_("email", "=", "alice@example.com")
    .first(&db)
    .await?;

// Multiple conditions
let users = User::query()
    .where_("active", "=", true)
    .where_("created_at", ">", yesterday)
    .limit(10)
    .get_all(&db)
    .await?;
```

### Update

```rust
let mut user = User::find(1, &db).await?;
user.name = "Alice Smith".to_string();
user.save(&db).await?;
```

### Delete

```rust
let user = User::find(1, &db).await?;
user.delete(&db).await?;
```

## Relationships

### Has Many

```rust
#[derive(Model)]
#[table_name = "posts"]
pub struct Post {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub content: String,
}

// Load user's posts
let user = User::find(1, &db).await?;
let posts = user.has_many::<Post>("user_id", &db).await?;
```

### Belongs To

```rust
// Load post's author
let post = Post::find(1, &db).await?;
let user = post.belongs_to::<User>("user_id", &db).await?;
```

## Migrations

### Create Migration

Using CLI:

```bash
oxidite migrate create create_users_table
```

This will create a new SQL file in the `migrations` directory.

### Run Migrations

```bash
oxidite migrate run
```

Or programmatically:

```rust
use oxidite_db::Migration;

Migration::run_all(&db).await?;
```

### Rollback

```bash
oxidite migrate revert
```

## Transactions

```rust
let tx = db.begin().await?;

// Perform operations
user.save(&tx).await?;
post.save(&tx).await?;

// Commit
tx.commit().await?;

// Or rollback on error
tx.rollback().await?;
```
