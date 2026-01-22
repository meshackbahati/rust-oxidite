# Database ORM

Oxidite provides a comprehensive database ORM (Object-Relational Mapping) system that makes it easy to work with databases in Rust applications.

## Overview

The database module provides:

- Multi-database support (PostgreSQL, MySQL, SQLite)
- Connection pooling
- Model derive macro
- Migrations
- Soft deletes and timestamps
- Validation
- Relationships (planned)

## Installation

Add the database feature to your `Cargo.toml`:

```toml
[dependencies]
oxidite = { version = "1.0", features = ["database"] }
```

## Database Connection

### Connecting to a Database

```rust
use oxidite::db::DbPool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to PostgreSQL
    let db = DbPool::connect("postgresql://username:password@localhost/database").await?;
    
    // Connect to MySQL
    let db = DbPool::connect("mysql://username:password@localhost/database").await?;
    
    // Connect to SQLite
    let db = DbPool::connect("sqlite:my_database.db").await?;
    
    Ok(())
}
```

### Connection Pool Options

```rust
use oxidite::db::{DbPool, PoolOptions};
use std::time::Duration;

let options = PoolOptions {
    max_connections: 20,
    min_connections: 2,
    connect_timeout: Duration::from_secs(30),
    idle_timeout: Some(Duration::from_secs(600)),
};

let db = DbPool::connect_with_options("sqlite:my_database.db", options).await?;
```

## Model Definition

Define database models using the `#[derive(Model)]` macro:

```rust
use oxidite::db::{Model, Database};
use serde::{Deserialize, Serialize};

#[derive(Model, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub created_at: i64,
    pub updated_at: i64,
}
```

### Model Attributes

#### Timestamp Fields

Models can include automatic timestamp fields:

```rust
#[derive(Model, Serialize, Deserialize, Clone)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub created_at: i64,  // Will be set automatically on creation
    pub updated_at: i64,  // Will be set automatically on updates
}
```

#### Soft Deletes

To enable soft deletes, include a `deleted_at` field:

```rust
#[derive(Model, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,  // Enables soft deletes
}
```

With soft deletes enabled, the `delete()` method will set the `deleted_at` field instead of removing the record from the database. The `find()` and `all()` methods will automatically exclude soft-deleted records.

#### Field Validation

Models can include validation attributes:

```rust
#[derive(Model, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i64,
    pub name: String,
    #[validate(email)]
    pub email: String,  // Will be validated as email format
    pub created_at: i64,
    pub updated_at: i64,
}
```

## Model Operations

### Create

```rust
use oxidite::db::Database;

async fn create_user(db: &impl Database) -> Result<(), Box<dyn std::error::Error>> {
    let mut user = User {
        id: 0,  // Will be set by the database
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
        created_at: 0,  // Will be set automatically
        updated_at: 0,  // Will be set automatically
    };
    
    // Create the user in the database
    user.create(db).await?;
    
    println!("Created user with ID: {}", user.id);
    Ok(())
}
```

### Read

```rust
// Find by ID
async fn find_user(db: &impl Database) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(user) = User::find(db, 1).await? {
        println!("Found user: {}", user.name);
    } else {
        println!("User not found");
    }
    Ok(())
}

// Find all users
async fn find_all_users(db: &impl Database) -> Result<(), Box<dyn std::error::Error>> {
    let users = User::all(db).await?;
    println!("Found {} users", users.len());
    Ok(())
}
```

### Update

```rust
async fn update_user(db: &impl Database) -> Result<(), Box<dyn std::error::Error>> {
    let mut user = User::find(db, 1).await?.unwrap();
    user.name = "Jane Doe".to_string();
    
    // Update the user in the database
    user.update(db).await?;
    
    println!("Updated user: {}", user.name);
    Ok(())
}
```

### Delete

```rust
// Soft delete (if enabled) or hard delete
async fn delete_user(db: &impl Database) -> Result<(), Box<dyn std::error::Error>> {
    let user = User::find(db, 1).await?.unwrap();
    user.delete(db).await?;
    
    println!("User deleted");
    Ok(())
}

// Hard delete regardless of soft delete setting
async fn force_delete_user(db: &impl Database) -> Result<(), Box<dyn std::error::Error>> {
    let user = User::find(db, 1).await?.unwrap();
    user.force_delete(db).await?;
    
    println!("User permanently deleted");
    Ok(())
}
```

### Save (Create or Update)

```rust
async fn save_user(db: &impl Database) -> Result<(), Box<dyn std::error::Error>> {
    let mut user = User::find(db, 1).await?.unwrap_or_else(|| User {
        id: 0,
        name: "New User".to_string(),
        email: "new@example.com".to_string(),
        created_at: 0,
        updated_at: 0,
    });
    
    user.name = "Updated Name".to_string();
    
    // This will either create or update based on the implementation
    user.save(db).await?;
    
    Ok(())
}
```

## Raw Queries

For more complex operations, you can execute raw SQL queries:

```rust
use oxidite::db::Database;

async fn custom_query(db: &impl Database) -> Result<(), Box<dyn std::error::Error>> {
    // Execute a query that doesn't return results
    let rows_affected = db.execute("UPDATE users SET name = 'Anonymous' WHERE email IS NULL").await?;
    println!("Updated {} rows", rows_affected);
    
    // Query multiple rows
    let rows = db.query("SELECT id, name, email FROM users WHERE active = 1").await?;
    for row in rows {
        let id: i64 = row.try_get("id")?;
        let name: String = row.try_get("name")?;
        let email: String = row.try_get("email")?;
        
        println!("User {}: {} ({})", id, name, email);
    }
    
    Ok(())
}
```

## Transactions

Oxidite supports database transactions:

```rust
use oxidite::db::{Database, DbTransaction};

async fn transaction_example(db: &impl Database) -> Result<(), Box<dyn std::error::Error>> {
    let tx = db.begin_transaction().await?;
    
    // Perform operations within the transaction
    let mut user = User {
        id: 0,
        name: "Transaction User".to_string(),
        email: "transaction@example.com".to_string(),
        created_at: 0,
        updated_at: 0,
    };
    
    user.create(&tx).await?;
    
    // If everything succeeds, commit the transaction
    tx.commit().await?;
    
    println!("Transaction committed successfully");
    Ok(())
}

// Example with rollback on error
async fn transaction_with_rollback(db: &impl Database) -> Result<(), Box<dyn std::error::Error>> {
    let tx = db.begin_transaction().await?;
    
    // Perform operations
    let mut user = User {
        id: 0,
        name: "Test User".to_string(),
        email: "test@example.com".to_string(),
        created_at: 0,
        updated_at: 0,
    };
    
    user.create(&tx).await?;
    
    // Simulate an error condition
    if user.name == "Test User" {
        // Rollback the transaction
        tx.rollback().await?;
        println!("Transaction rolled back");
        return Ok(());
    }
    
    // If no error, commit
    tx.commit().await?;
    println!("Transaction committed");
    Ok(())
}
```

## Migrations

Oxidite provides a migration system to manage database schema changes:

```rust
use oxidite::db::{Migration, MigrationManager};

// Run all pending migrations
async fn run_migrations(db: &impl Database) -> Result<(), Box<dyn std::error::Error>> {
    MigrationManager::run_pending_migrations(db).await?;
    Ok(())
}
```

## Complete Example

Here's a complete example showing how to use the database functionality:

```rust
use oxidite::prelude::*;
use oxidite::db::{Model, Database, DbPool};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Model, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,  // Enable soft deletes
}

#[derive(Clone)]
struct AppState {
    db: DbPool,
}

async fn create_user_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<User>
) -> Result<OxiditeResponse> {
    let mut user = User {
        id: 0,
        name: payload.name,
        email: payload.email,
        created_at: 0,
        updated_at: 0,
        deleted_at: None,
    };
    
    user.create(&state.db).await
        .map_err(|e| Error::Server(e.to_string()))?;
    
    Ok(response::json(serde_json::json!(user)))
}

async fn get_user_handler(
    State(state): State<Arc<AppState>>,
    Path(params): Path<serde_json::Value>
) -> Result<OxiditeResponse> {
    let id = params["id"].as_i64().unwrap_or(0);
    
    match User::find(&state.db, id).await {
        Ok(Some(user)) => Ok(response::json(serde_json::json!(user))),
        Ok(None) => Err(Error::NotFound),
        Err(e) => Err(Error::Server(e.to_string())),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let db = DbPool::connect("sqlite::memory:").await?;
    
    let state = Arc::new(AppState { db });
    
    let mut router = Router::new();
    router.post("/users", create_user_handler);
    router.get("/users/:id", get_user_handler);
    
    let service = ServiceBuilder::new()
        .layer(AddExtensionLayer::new(state))
        .service(router);
    
    Server::new(service)
        .listen("127.0.0.1:3000".parse().unwrap())
        .await
}
```

## Best Practices

1. **Always use connection pooling** in production applications
2. **Validate input data** before saving to the database
3. **Use transactions** for operations that need to be atomic
4. **Handle errors appropriately** and provide meaningful error messages
5. **Use migrations** to manage database schema changes
6. **Consider using soft deletes** for data that might need to be recovered
7. **Index frequently queried columns** for better performance