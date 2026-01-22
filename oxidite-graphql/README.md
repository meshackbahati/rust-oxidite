# oxidite-graphql

GraphQL integration for the Oxidite web framework.

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/oxidite-graphql.svg)](https://crates.io/crates/oxidite-graphql)
[![Docs.rs](https://docs.rs/oxidite-graphql/badge.svg)](https://docs.rs/oxidite-graphql)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

</div>

## Overview

`oxidite-graphql` provides seamless GraphQL integration for Oxidite applications. It leverages the Juniper and async-graphql ecosystems to offer type-safe GraphQL schemas, resolvers, and automatic schema generation. The crate integrates smoothly with Oxidite's routing and middleware systems.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
oxidite-graphql = "2.0"
oxidite = "2.0"
```

## Features

- **Type-Safe GraphQL Schemas**: Define schemas using Rust types with automatic mapping
- **Integration with Oxidite Router**: Easy endpoint registration for GraphQL APIs
- **Automatic Schema Generation**: Generate schemas from Rust types and resolvers
- **Query Complexity Analysis**: Built-in protection against complex queries
- **Introspection Support**: Full GraphQL introspection capabilities
- **Subscriptions**: Real-time GraphQL subscriptions with WebSocket support
- **Error Handling**: Proper GraphQL error formatting and reporting
- **Batching**: Support for batched GraphQL queries
- **Caching**: Integrated caching for GraphQL resolvers

## Usage

### Basic GraphQL Setup

```rust
use oxidite::prelude::*;
use oxidite_graphql::{GraphQLHandler, Schema, RootNode};
use juniper::{EmptyMutation, EmptySubscription, FieldResult};

// Define your GraphQL context
struct Context {
    // Your application context
}

impl juniper::Context for Context {}

// Define your GraphQL objects
#[derive(juniper::GraphQLObject)]
struct User {
    id: String,
    name: String,
    email: String,
}

// Define your resolvers
struct QueryRoot;

#[juniper::graphql_object(Context = Context)]
impl QueryRoot {
    fn users(context: &Context) -> FieldResult<Vec<User>> {
        // Fetch users from database
        Ok(vec![
            User {
                id: "1".to_string(),
                name: "John Doe".to_string(),
                email: "john@example.com".to_string(),
            }
        ])
    }

    fn user(context: &Context, id: String) -> FieldResult<Option<User>> {
        // Fetch specific user
        Ok(Some(User {
            id,
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
        }))
    }
}

// Create schema
type Schema = juniper::RootNode<'static, QueryRoot, EmptyMutation, EmptySubscription>;

fn create_schema() -> Schema {
    Schema::new(QueryRoot, EmptyMutation::new(), EmptySubscription::new())
}

#[tokio::main]
async fn main() -> Result<()> {
    let schema = std::sync::Arc::new(create_schema());
    let context_factory = || Context {};

    let mut router = Router::new();
    
    // Add GraphQL endpoint
    router.get("/graphql", GraphQLHandler::new(schema.clone(), context_factory));
    router.post("/graphql", GraphQLHandler::new(schema, context_factory));
    
    // Add GraphQL Playground for development
    router.get("/playground", GraphQLHandler::playground("/graphql"));

    Server::new(router)
        .listen("127.0.0.1:3000".parse().unwrap())
        .await
}
```

### Using async-graphql Alternative

```rust
use oxidite::prelude::*;
use oxidite_graphql::AsyncGraphQLHandler;
use async_graphql::*;

// Define your GraphQL object
#[derive(SimpleObject)]
struct User {
    id: ID,
    name: String,
    email: String,
}

// Define your query root
#[derive(Default)]
struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn users(&self) -> Vec<User> {
        vec![
            User {
                id: "1".into(),
                name: "John Doe".to_string(),
                email: "john@example.com".to_string(),
            }
        ]
    }

    async fn user(&self, id: ID) -> Option<User> {
        Some(User {
            id,
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
        })
    }
}

// Create schema
type AppSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

fn create_async_schema() -> AppSchema {
    Schema::build(QueryRoot::default(), EmptyMutation, EmptySubscription).finish()
}

async fn setup_async_graphql() -> Result<()> {
    let schema = create_async_schema();
    
    let mut router = Router::new();
    
    // Add GraphQL endpoint
    router.all("/graphql", AsyncGraphQLHandler::new(schema));
    
    Server::new(router)
        .listen("127.0.0.1:3000".parse().unwrap())
        .await
}
```

### GraphQL with Database Integration

```rust
use oxidite::prelude::*;
use oxidite_db::{Model, Database};
use oxidite_graphql::{GraphQLHandler, Context};

// Define database model
#[derive(Model, juniper::GraphQLObject)]
#[model(table = "users")]
struct User {
    id: i32,
    name: String,
    email: String,
    created_at: String,
}

// Extend context to include database
struct AppContext {
    db: Database,
}

impl juniper::Context for AppContext {}

// Define resolvers with database access
struct QueryRoot;

#[juniper::graphql_object(Context = AppContext)]
impl QueryRoot {
    fn users(context: &AppContext) -> FieldResult<Vec<User>> {
        let users = context.db.find_all::<User>()?;
        Ok(users)
    }

    fn user(context: &AppContext, id: i32) -> FieldResult<Option<User>> {
        let user = context.db.find_by_id::<User>(id)?;
        Ok(user)
    }
}
```

## Advanced Features

### Middleware Integration

```rust
use oxidite::prelude::*;
use oxidite_graphql::GraphQLHandler;

// Add authentication middleware to GraphQL endpoints
async fn auth_middleware(req: Request, next: Next) -> Result<Response> {
    // Check authentication
    if !is_authenticated(&req)? {
        return Err(Error::Unauthorized("Authentication required".to_string()));
    }
    
    next.run(req).await
}

let mut router = Router::new();
router.get("/graphql").middleware(auth_middleware);
router.post("/graphql").middleware(auth_middleware);
```

### Subscriptions

```rust
use oxidite_graphql::{Subscription, SubscriptionHandler};

// Define subscription resolvers
struct SubscriptionRoot;

#[juniper::graphql_subscription]
impl SubscriptionRoot {
    async fn user_events(&self) -> impl Stream<Item = UserEvent> {
        // Return a stream of events
        tokio_stream::iter(vec![
            UserEvent::UserCreated(User { /* ... */ }),
            UserEvent::UserUpdated(User { /* ... */ }),
        ])
    }
}
```

### Error Handling

```rust
use juniper::FieldError;

#[juniper::graphql_object(Context = Context)]
impl QueryRoot {
    fn user(context: &Context, id: String) -> FieldResult<User> {
        match get_user_by_id(id) {
            Ok(Some(user)) => Ok(user),
            Ok(None) => Err(FieldError::new("User not found", juniper::Value::null())),
            Err(e) => Err(FieldError::new("Database error", 
                juniper::Value::string(e.to_string()))),
        }
    }
}
```

## Performance

- **Query Optimization**: Automatic query optimization and field selection
- **Caching**: Built-in resolver caching with configurable TTL
- **Batching**: DataLoader integration for efficient N+1 query resolution
- **Compression**: Automatic response compression for large payloads

## License

MIT