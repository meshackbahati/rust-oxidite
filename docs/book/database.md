# Database ORM

Oxidite provides a powerful Object-Relational Mapping (ORM) system that allows you to work with databases using Rust structs. This chapter covers how to define models, perform database operations, and use relationships.

## Overview

The Oxidite ORM provides:
- Type-safe database operations
- Model definitions with derive macros
- Relationship management
- Migrations and schema management
- Query building capabilities
- Validation and hooks

## Model Definition

Define your database models using the `Model` derive macro:

```rust,ignore
use oxidite::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Model, sqlx::FromRow, Serialize, Deserialize)]
#[model(table = "users")]
pub struct User {
    pub id: i64,
    pub email: String,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
}
```

## Basic CRUD Operations

### Creating Records

```rust,ignore
use oxidite_db::{DbPool, Model};

async fn create_user(db: &DbPool) -> Result<()> {
    let user = User {
        id: 0, 
        email: "john@example.com".to_string(),
        name: "John Doe".to_string(),
        created_at: 0,
        updated_at: 0,
    };
    
    let saved_user = user.save(db).await?;
    println!("Created user with ID: {}", saved_user.id);
    
    Ok(())
}
```

### Reading Records

```rust,ignore
async fn find_users(db: &DbPool) -> Result<()> {
    // Find all users
    let all_users = User::find_all(db).await?;
    println!("Found {} users", all_users.len());
    
    // Find user by ID
    if let Some(user) = User::find_by_id(db, 1).await? {
        println!("Found user: {}", user.name);
    }
    
    // Find users with conditions
    let active_users = User::query()
        .filter_eq("active", true)
        .fetch_all(db)
        .await?;
    
    Ok(())
}
```

### Updating Records

```rust,ignore
async fn update_user(db: &DbPool) -> Result<()> {
    if let Some(mut user) = User::find_by_id(db, 1).await? {
        user.name = "John Updated".to_string();
        let updated_user = user.save(db).await?;
    }
    Ok(())
}
```

### Deleting Records

```rust,ignore
async fn delete_user(db: &DbPool) -> Result<()> {
    if let Some(user) = User::find_by_id(db, 1).await? {
        user.delete(db).await?;
    }
    Ok(())
}
```

## Relationships

Define relationships between models:

```rust,ignore
use oxidite::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Model, Serialize, Deserialize)]
#[model(table = "posts")]
pub struct Post {
    #[model(primary_key)]
    pub id: i32,
    pub title: String,
    pub content: String,
    pub user_id: i32,  // Foreign key
    #[model(created_at)]
    pub created_at: String,
}

#[derive(Model, Serialize, Deserialize)]
#[model(table = "comments")]
pub struct Comment {
    #[model(primary_key)]
    pub id: i32,
    pub content: String,
    pub user_id: i32,   // Foreign key
    pub post_id: i32,  // Foreign key
    #[model(created_at)]
    pub created_at: String,
}

// Update User model to include relationships
#[derive(Model, Serialize, Deserialize)]
#[model(table = "users")]
pub struct User {
    #[model(primary_key)]
    pub id: i32,
    #[model(unique, not_null)]
    pub email: String,
    #[model(not_null)]
    pub name: String,
    #[model(default = "now")]
    pub created_at: String,
    #[model(updated_at)]
    pub updated_at: String,
    #[model(default = "false")]
    pub active: bool,
}

// Access related records
async fn work_with_relationships() -> Result<()> {
    // Find a user
    if let Some(user) = User::find_by_id(1).await? {
        // Find user's posts
        let posts = Post::find_where(&format!("user_id = {}", user.id)).await?;
        println!("User {} has {} posts", user.name, posts.len());
        
        // Find user's comments
        let comments = Comment::find_where(&format!("user_id = {}", user.id)).await?;
        println!("User {} has {} comments", user.name, comments.len());
    }
    
    Ok(())
}
```

## Query Building

Use the query builder for complex queries:

```rust,ignore
use oxidite::prelude::*;

async fn complex_queries() -> Result<()> {
    // Find users with custom conditions
    let users = User::find_where("name LIKE '%John%' AND active = true").await?;
    println!("Found {} users matching criteria", users.len());
    
    // Find with ordering
    let recent_users = User::find_where("active = true")
        .order_by("created_at DESC")
        .limit(10)
        .await?;
    
    // Find with joins (conceptual - exact syntax may vary)
    let users_with_posts = execute_raw_query("
        SELECT u.*, COUNT(p.id) as post_count 
        FROM users u 
        LEFT JOIN posts p ON u.id = p.user_id 
        WHERE u.active = true 
        GROUP BY u.id 
        ORDER BY post_count DESC
    ").await?;
    
    Ok(())
}

async fn execute_raw_query<T>(_sql: &str) -> Result<Vec<T>> {
    // Implementation would depend on the specific database connector
    Ok(vec![])
}
```

## Migrations
 
Oxidite uses SQL-based migrations managed by the `oxidite-cli`. 
 
Create a migration:
```bash
oxidite generate migration create_users_table
```

This creates a `.sql` file in `migrations/`:
```sql
-- migrate:up
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    created_at BIGINT DEFAULT 0,
    updated_at BIGINT DEFAULT 0
);

-- migrate:down
DROP TABLE users;
```

Run migrations:
```bash
oxidite migrate run
```

## Validation

Add validation to your models:

```rust,ignore
use oxidite::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Model, Serialize, Deserialize)]
#[model(table = "users")]
pub struct ValidatedUser {
    #[model(primary_key)]
    pub id: i32,
    #[model(unique, not_null)]
    pub email: String,
    #[model(not_null)]
    pub name: String,
    #[model(validate = "validate_age")]
    pub age: u8,
    #[model(default = "now")]
    pub created_at: String,
    #[model(updated_at)]
    pub updated_at: String,
}

impl ValidatedUser {
    // Validation method
    fn validate_age(&self) -> Result<(), String> {
        if self.age < 13 {
            Err("User must be at least 13 years old".to_string())
        } else if self.age > 120 {
            Err("Invalid age".to_string())
        } else {
            Ok(())
        }
    }
    
    // Hook methods
    fn before_save(&mut self) -> Result<(), String> {
        self.updated_at = now();
        self.validate_age()  // Run validation before saving
    }
    
    fn after_save(&self) -> Result<(), String> {
        println!("User {} saved with ID {}", self.name, self.id);
        Ok(())
    }
}
```

## Transactions

Perform operations within transactions:

```rust,ignore
use oxidite::prelude::*;

async fn transaction_example() -> Result<()> {
    // Start a transaction
    let tx = begin_transaction().await?;
    
    match async {
        // Create user
        let user = User {
            id: 0,
            email: "transaction@example.com".to_string(),
            name: "Transaction User".to_string(),
            created_at: now(),
            updated_at: now(),
            active: true,
        };
        let saved_user = user.save().await?;
        
        // Create a post for the user
        let post = Post {
            id: 0,
            title: "First Post".to_string(),
            content: "Hello, world!".to_string(),
            user_id: saved_user.id,
            created_at: now(),
        };
        post.save().await?;
        
        Ok::<_, Error>(saved_user.id)
    }.await {
        Ok(user_id) => {
            // Commit the transaction
            tx.commit().await?;
            println!("Successfully created user {} and associated post", user_id);
        }
        Err(e) => {
            // Rollback the transaction
            tx.rollback().await?;
            println!("Transaction failed: {:?}", e);
            return Err(e);
        }
    }
    
    Ok(())
}

async fn begin_transaction() -> Result<Transaction> {
    // Implementation would depend on the database connector
    Ok(Transaction {})
}

pub struct Transaction;

impl Transaction {
    pub async fn commit(self) -> Result<()> {
        Ok(())
    }
    
    pub async fn rollback(self) -> Result<()> {
        Ok(())
    }
}
```

## Soft Deletes

Models can support soft deletes:

```rust,ignore
use oxidite::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Model, Serialize, Deserialize)]
#[model(table = "soft_delete_users", soft_delete = true)]
pub struct SoftDeleteUser {
    #[model(primary_key)]
    pub id: i32,
    #[model(unique, not_null)]
    pub email: String,
    #[model(not_null)]
    pub name: String,
    #[model(deleted_at)]  // Special field for soft deletes
    pub deleted_at: Option<String>,
    #[model(default = "now")]
    pub created_at: String,
    #[model(updated_at)]
    pub updated_at: String,
}

async fn soft_delete_example() -> Result<()> {
    // Find all users (includes soft-deleted ones)
    let all_users = SoftDeleteUser::find_all_with_deleted().await?;
    
    // Find only active users (excludes soft-deleted ones)
    let active_users = SoftDeleteUser::find_all().await?;
    
    // Soft delete a user
    if let Some(mut user) = SoftDeleteUser::find_by_id(1).await? {
        user.delete().await?;  // This sets deleted_at instead of removing the record
        println!("User soft-deleted");
    }
    
    // Restore a soft-deleted user
    if let Some(mut user) = SoftDeleteUser::find_by_id_trashed(1).await? {
        user.restore().await?;  // This clears the deleted_at field
        println!("User restored");
    }
    
    Ok(())
}
```

## Connection Management

Configure database connections:

```rust,ignore
use oxidite::prelude::*;

async fn configure_database() -> Result<()> {
    // Configure database connection
    let db_config = DatabaseConfig {
        url: std::env::var("DATABASE_URL").unwrap_or("sqlite::memory:".to_string()),
        pool_size: 10,
        timeout: std::time::Duration::from_secs(30),
    };
    
    // Initialize the database connection
    init_database(db_config).await?;
    
    Ok(())
}

pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: usize,
    pub timeout: std::time::Duration,
}

async fn init_database(_config: DatabaseConfig) -> Result<()> {
    // Implementation would depend on the specific database connector
    Ok(())
}
```

## Error Handling

Handle database errors appropriately:

```rust,ignore
use oxidite::prelude::*;

async fn error_handling_example() -> Result<()> {
    match User::find_by_id(999999).await {
        Ok(Some(user)) => {
            println!("Found user: {}", user.name);
        }
        Ok(None) => {
            println!("User not found");
        }
        Err(Error::InternalServerError(msg)) => {
            eprintln!("Database error: {}", msg);
            return Err(Error::InternalServerError(msg));
        }
        Err(e) => {
            eprintln!("Unexpected error: {:?}", e);
            return Err(e);
        }
    }
    
    Ok(())
}
```

## Performance Considerations

1. **Use Indexes**: Add database indexes for frequently queried fields
2. **Batch Operations**: Use batch operations when possible
3. **Connection Pooling**: Use connection pooling for better performance
4. **N+1 Queries**: Be aware of N+1 query problems with relationships
5. **Caching**: Consider caching frequently accessed data

## Security Considerations

1. **SQL Injection**: The ORM protects against SQL injection by using parameterized queries
2. **Input Validation**: Always validate input before saving to the database
3. **Access Control**: Implement proper access control for database operations
4. **Data Encryption**: Consider encrypting sensitive data at rest

## Summary

The Oxidite ORM provides a comprehensive solution for database operations:

- Define models with the `Model` derive macro
- Perform CRUD operations with type safety
- Define and work with relationships
- Handle migrations for schema management
- Add validation and hooks to models
- Use transactions for data consistency
- Support for soft deletes
- Proper error handling and security considerations

The ORM abstracts away the complexity of raw SQL while providing the flexibility to execute custom queries when needed.