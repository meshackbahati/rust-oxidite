# API Keys Guide

Learn how to implement API key authentication in Oxidite.

## Installation

```toml
[dependencies]
oxidite = { version = "1.0", features = ["auth", "database"] }
```

## Setup

```rust
use oxidite::prelude::*;
use oxidite_auth::ApiKeyMiddleware;
use oxidite_db::Database;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    db: Arc<Database>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let db = Arc::new(Database::connect(&std::env::var("DATABASE_URL")?).await?);
    let state = AppState { db };
    
    let mut app = Router::new();
    
    // Protected route
    app.get("/api/data", get_data)
        .layer(ApiKeyMiddleware);

    let app = app.with_state(state);
    
    Server::new(app).listen("127.0.0.1:3000".parse()?).await
}
```

## Generate API Keys

```rust
use oxidite_auth::ApiKey;

async fn create_api_key(user_id: i64, db: &Database) -> Result<String> {
    let (api_key_model, plain_key) = ApiKey::generate(user_id);
    api_key_model.save(db).await?;
    
    // Return the plain key to user (only shown once)
    Ok(plain_key)
}
```

## Validate API Keys

The `ApiKeyMiddleware` handles validation automatically. It expects the API key in the `Authorization: Bearer <key>` header.

## Using API Keys

Clients send the API key in the `Authorization` header:

```bash
curl -H "Authorization: Bearer your-api-key" \
     http://localhost:3000/api/data
```

## Complete Example

```rust
use oxidite::prelude::*;
use oxidite_auth::{ApiKey, ApiKeyAuth};
use serde::Serialize;

#[derive(Serialize)]
struct ApiKeyResponse {
    key: String,
}

async fn create_key(
    State(db): State<Arc<Database>>,
    Json(req): Json<CreateKeyRequest>,
) -> Result<OxiditeResponse> {
    let (api_key_model, plain_key) = ApiKey::generate(req.user_id);
    api_key_model.save(&db).await?;
    
    Ok(OxiditeResponse::json(json!(ApiKeyResponse { key: plain_key })))
}

async fn protected_route(auth: ApiKeyAuth) -> Result<OxiditeResponse> {
    // The user_id is available via the ApiKeyAuth extractor
    let user_id = auth.user_id;
    Ok(OxiditeResponse::json(json!({ "message": "Success!", "user_id": user_id })))
}
```

## Best Practices

1. Never log API keys
2. Hash keys before storing
3. Allow key rotation
4. Set expiration dates
5. Rate limit by API key

## Revoke Keys

```rust
async fn revoke_key(key_id: i64, db: &Database) -> Result<()> {
    ApiKey::delete(key_id, db).await
}
```
