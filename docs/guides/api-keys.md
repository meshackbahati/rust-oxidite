# API Key Authentication

Quick guide for API key authentication in Oxidite.

## Usage

### Generating Keys
```rust
use oxidite_auth::ApiKey;

// Create API key for user
let (api_key, key_plaintext) = ApiKey::create_for_user(
    &db,
    user_id,
    "My App",
    Some(expires_timestamp)
).await?;

// Give key_plaintext to user (shown only once!)
println!("Your API key: {}", key_plaintext);  // ox_xxxxxx
```

### Authentication Methods

**1. Authorization Header (Recommended)**
```bash
curl -H "Authorization: Bearer ox_your_key_here" https://api.example.com/endpoint
```

**2. X-API-Key Header**
```bash
curl -H "X-API-Key: ox_your_key_here" https://api.example.com/endpoint
```

**3. Query Parameter**
```bash
curl https://api.example.com/endpoint?api_key=ox_your_key_here
```

### Middleware Setup
```rust
use oxidite_auth::ApiKeyMiddleware;

let api_auth = ApiKeyMiddleware::new(db.clone());

router.get("/api/protected", move |mut req| async move {
    let user_id = api_auth.authenticate(&mut req).await?;
    // User is authenticated, proceed
    Ok(Response::new("Protected data"))
});
```

### Managing Keys
```rust
// List user's keys
let keys = ApiKey::get_user_keys(&db, user_id).await?;

// Revoke a key
ApiKey::revoke(&db, key_id, user_id).await?;
```

## Security Notes
- Keys are hashed with SHA-256 before storage
- Plain keys start with `ox_` prefix
- Keys are base64url-encoded (URL-safe)
- Set expiration dates for temporary access
- Track `last_used_at` for auditing
