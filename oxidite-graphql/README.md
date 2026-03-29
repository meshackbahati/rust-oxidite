# oxidite-graphql

GraphQL integration for Oxidite using Juniper.

## Installation

```toml
[dependencies]
oxidite-graphql = "2.1.0"
```

## What This Crate Provides

- A default Juniper schema (`QueryRoot`, `MutationRoot`) via `create_schema()`
- `GraphQLHandler` for mounting GraphQL POST + playground GET endpoints
- `Context` with extension storage and optional database integration (`database` feature)

## Basic Usage

```rust
use oxidite::prelude::*;
use oxidite_graphql::create_handler;

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();

    // Mount at /graphql (POST for queries, GET for playground)
    create_handler().mount(&mut router)?;

    // or custom path
    // create_handler().mount_at(&mut router, "/api/graphql")?;

    Server::new(router)
        .listen("127.0.0.1:3000".parse().unwrap())
        .await
}
```

## Notes

- POST endpoint accepts both single and batch GraphQL requests.
- Context extensions can be used to pass request-scoped data to resolvers.
- The optional `database` feature enables `Context::with_database(...)`.
