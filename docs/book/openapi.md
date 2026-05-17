# OpenAPI & Swagger UI

Oxidite provides first-class support for automated API documentation via OpenAPI 3.1. By using the `openapi` feature, your framework automatically generates schema definitions and provides a built-in Swagger UI for testing.

## Enabling OpenAPI

To enable OpenAPI support, add the `openapi` feature to your `Cargo.toml`:

```toml
[dependencies]
oxidite = { version = "2.2", features = ["full", "openapi"] }
```

## Configuration

In your `oxidite.toml`, you can configure the OpenAPI metadata:

```toml
[api]
title = "My Professional API"
version = "1.0.0"
description = "A high-performance Rust API built with Oxidite."
contact_email = "dev@example.com"
license = "MIT"

[api.swagger]
enabled = true
path = "/docs"
theme = "classic" # options: classic, dark, modern
```

## Automated Schema Generation

Oxidite automatically inspects your `router` and `handlers` to derive the API schema. For types to appear in the schema, they should implement `JsonSchema` (from the `schemars` crate, which is re-exported).

### Example Model

```rust,ignore
use oxidite::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Model, sqlx::FromRow)]
#[model(table = "users")]
pub struct User {
    pub id: i64,
    pub email: String,
    pub name: String,
}
```

### Handler with Metadata

You can use the `#[api_operation]` macro to add specific metadata to your handlers:

```rust,ignore
use oxidite::prelude::*;

#[api_operation(
    summary = "List all users",
    description = "Returns a paginated list of active users from the database.",
    tags = ["Users"]
)]
async fn list_users(db: State<DbPool>) -> Result<Response> {
    let users = User::find_all(&db).await?;
    Ok(Response::json(users))
}
```

## Accessing the UI

Once your server is running, navigate to the path defined in your configuration (default is `/docs`). You will find a fully interactive Swagger UI where you can:

- **Explore Endpoints**: See all available routes, methods, and parameters.
- **Inspect Schemas**: View the JSON structure of requests and responses.
- **Try it out**: Execute real requests against your running server directly from the browser.

## Security Schemes

Oxidite's OpenAPI integration supports various security schemes:

### JWT Bearer Authentication

```rust,ignore
router.api_security("jwt", SecurityScheme::Http {
    scheme: "bearer".to_string(),
    bearer_format: Some("JWT".to_string()),
});
```

### API Key Authentication

```rust,ignore
router.api_security("api_key", SecurityScheme::ApiKey {
    name: "X-API-KEY".to_string(),
    in_: "header".to_string(),
});
```

## Best Practices

1. **Keep Summary Short**: Use the `summary` field for a concise overview.
2. **Use Tags**: Group related endpoints using `tags` for better navigation in the UI.
3. **Document Errors**: Use the `responses` attribute in `#[api_operation]` to document common error states (400, 401, 404).
4. **Version Your API**: Use the versioning features of Oxidite to keep your OpenAPI docs in sync with your API lifecycle.
