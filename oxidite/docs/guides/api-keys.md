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
use oxidite::db::DbPool;
use oxidite::auth::api_key::*;

#[tokio::main]
async fn main() -> Result<()> {
    let db = DbPool::connect(&std::env::var("DATABASE_URL")?).await?;
    
    let mut app = Router::new();
    
    // Protected route
    app.get("/api/data", get_data)
        .middleware(ApiKeyMiddleware::new(db.clone()));
    
    Server::new(app).listen("127.0.0.1:3000".parse()?).await
}
```

## Generate API Keys

```rust
use oxidite::auth::api_key::*;

async fn create_api_key(user_id: i64, db: &Database) -> Result<String> {
    let api_key = ApiKey::generate(user_id)?;
    api_key.save(db).await?;
    
    // Return the plain key to user (only shown once)
    Ok(api_key.key)
}
```

## Validate API Keys

```rust
// In middleware or handler
async fn validate_key(key: &str, db: &Database) -> Result<ApiKey> {
    ApiKey::validate(key, db).await
}
```

## Using API Keys

Clients send the API key in the `Authorization` header:

```bash
curl -H "Authorization: Bearer your-api-key" \
     http://localhost:3000/api/data
```

## Complete Example

```rust
use oxidite::prelude::*;
use oxidite::auth::api_key::*;

#[derive(Serialize)]
struct ApiKeyResponse {
    key: String,
    user_id: i64,
}

async fn create_key(
    State(db): State<Database>,
    Json(req): Json<CreateKeyRequest>,
) -> Result<Json<ApiKeyResponse>> {
    let api_key = ApiKey::generate(req.user_id)?;
    api_key.save(&db).await?;
    
    Ok(Json(ApiKeyResponse {
        key: api_key.key,
        user_id: api_key.user_id,
    }))
}

async fn protected_route(api_key: ApiKey) -> Result<Json<Data>> {
    // api_key.user_id available
    Ok(Json(Data { message: "Success!".into() }))
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
    ApiKey::delete(db, key_id).await
}
```
