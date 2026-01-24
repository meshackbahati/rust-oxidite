# Database & ORM Guide

Complete guide to using the Oxidite ORM for database operations.

## Installation

```toml
[dependencies]
oxidite = { version = "1.0", features = ["database"] }
```

Or use the full framework:

```toml
[dependencies]
oxidite = "1.0"  # Includes database features
```

## Setup

### Database Connection

Create `.env` file:

```env
DATABASE_URL=postgresql://user:password@localhost/mydb
```

Connect in your app:

```rust
use oxidite::prelude::*;
use oxidite::db::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Load config
    dotenv::dotenv().ok();
    
    // Connect to database
    let db = DbPool::connect(&std::env::var("DATABASE_URL")?).await?;
    
    // Your app code
    Ok(())
}
```

## Defining Models

```rust
use oxidite::db::*;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Model, Serialize, Deserialize, Clone)]
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
let user = User {
    id: 0,  // Auto-generated
    name: "Alice".to_string(),
    email: "alice@example.com".to_string(),
    password_hash: hash_password("password")?,
    created_at: Utc::now(),
    updated_at: Utc::now(),
};

user.save(&db).await?;
```

### Read

```rust
// Find by ID
let user = User::find(&db, 1).await?;

// Find all
let users = User::all(&db).await?;

// Where clause
let user = User::where_eq(&db, "email", "alice@example.com")
    .first()
    .await?;

// Multiple conditions
let users = User::query(&db)
    .where_eq("active", true)
    .where_gt("created_at", yesterday)
    .limit(10)
    .get()
    .await?;
```

### Update

```rust
let mut user = User::find(&db, 1).await?;
user.name = "Alice Smith".to_string();
user.updated_at = Utc::now();
user.save(&db).await?;
```

### Delete

```rust
let user = User::find(&db, 1).await?;
user.delete(&db).await?;

// Soft delete (if enabled)
user.soft_delete(&db).await?;
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
let user = User::find(&db, 1).await?;
let posts = user.has_many::<Post>(&db, "user_id").await?;
```

### Belongs To

```rust
// Load post's author
let post = Post::find(&db, 1).await?;
let user = post.belongs_to::<User>(&db, "user_id").await?;
```

### Has One

```rust
#[derive(Model)]
#[table_name = "profiles"]
pub struct Profile {
    pub id: i64,
    pub user_id: i64,
    pub bio: String,
}

// Load user's profile
let profile = user.has_one::<Profile>(&db, "user_id").await?;
```

## Migrations

### Create Migration

Using CLI:

```bash
oxidite migrate create create_users_table
```

Or manually create `migrations/TIMESTAMP_create_users.sql`:

```sql
-- Up Migration
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Down Migration
DROP TABLE users;
```

### Run Migrations

```bash
oxidite migrate run
```

Or programmatically:

```rust
use oxidite::db::Migration;

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

## Advanced Queries

### Raw SQL

```rust
let results: Vec<User> = db.query_as("SELECT * FROM users WHERE active = $1")
    .bind(true)
    .fetch_all()
    .await?;
```

### Pagination

```rust
let page = 1;
let per_page = 20;

let users = User::query(&db)
    .offset((page - 1) * per_page)
    .limit(per_page)
    .get()
    .await?;
```

### Aggregations

```rust
let count: i64 = db.query_scalar("SELECT COUNT(*) FROM users")
    .fetch_one()
    .await?;
```

## Best Practices

1. **Use transactions** for related operations
2. **Index frequently queried columns**
3. **Validate data** before saving
4. **Use soft deletes** for important data
5. **Lazy load** relationships when needed

## Example: Complete Model

```rust
use oxidite::db::*;

#[derive(Model, Serialize, Deserialize)]
#[table_name = "users"]
pub struct User {
    pub id: i64,
    
    #[validate(length(min = 3, max = 50))]
    pub name: String,
    
    #[validate(email)]
    pub email: String,
    
    pub password_hash: String,
    
    #[serde(skip)]
    pub  deleted_at: Option<DateTime<Utc>>,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub async fn create(db: &Database, name: String, email: String, password: String) -> Result<Self> {
        let mut user = Self {
            id: 0,
            name,
            email,
            password_hash: hash_password(&password)?,
            deleted_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        user.save(db).await?;
        Ok(user)
    }
    
    pub async fn find_by_email(db: &Database, email: &str) -> Result<Option<Self>> {
        User::where_eq(db, "email", email).first().await.ok()
    }
}
```
