# Authentication Guide

## Overview

Oxidite provides built-in authentication support with JWT tokens, sessions, and OAuth2.

## JWT Authentication

### Setup

```rust
use oxidite_auth::{JwtManager, Claims};

// Create JWT manager with secret
let jwt = JwtManager::new("your-secret-key");

// Create claims
let claims = Claims::new("user123")
    .with_role("admin")
    .with_expiration_minutes(60);

// Generate token
let token = jwt.encode(&claims)?;

// Verify and decode token
let decoded = jwt.decode(&token)?;
```

### Custom Claims

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct CustomClaims {
    sub: String,
    role: String,
    permissions: Vec<String>,
    exp: usize,
}

let custom_claims = CustomClaims {
    sub: "user123".to_string(),
    role: "admin".to_string(),
    permissions: vec!["read".to_string(), "write".to_string()],
    exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
};

let token = jwt.encode(&custom_claims)?;
```

## Sessions

### Memory-based Sessions

```rust
use oxidite_auth::{SessionManager, Session};

let manager = SessionManager::new_memory();

// Create session
let session = Session::new("user123");
let session_id = manager.create(session).await?;

// Retrieve session
if let Some(session) = manager.get(&session_id).await? {
    println!("User: {}", session.user_id);
}

// Delete session (logout)
manager.delete(&session_id).await?;
```

### Redis-based Sessions

```rust
let manager = SessionManager::new_redis("redis://localhost")?;

// Same API as memory-based
let session_id = manager.create(Session::new("user123")).await?;
```

## Password Hashing

Use Argon2 for secure password hashing:

```rust
use oxidite_auth::hash_password;

// Hash password
let hashed = hash_password("user_password")?;

// Verify password
let is_valid = oxidite_auth::verify_password("user_password", &hashed)?;
```

## OAuth2

### Setup OAuth2 Client

```rust
use oxidite_auth::{OAuth2Client, OAuth2Config};

let config = OAuth2Config {
    client_id: "your-client-id".to_string(),
    client_secret: "your-client-secret".to_string(),
    auth_url: "https://provider.com/oauth/authorize".to_string(),
    token_url: "https://provider.com/oauth/token".to_string(),
    redirect_url: "http://localhost:8080/callback".to_string(),
};

let client = OAuth2Client::new(config);

// Get authorization URL
let auth_url = client.get_authorize_url(&["read", "write"]);
// Redirect user to auth_url
```

### Handle Callback

```rust
// After user authorizes and provider redirects to /callback?code=...
async fn oauth_callback(code: String, client: &OAuth2Client) -> Result<String, Box<dyn std::error::Error>> {
    // Exchange code for access token
    let token_response = client.exchange_code(&code).await?;
    
    // Get user info
    let user_info = client.get_user_info(&token_response.access_token).await?;
    
    // Create session/JWT for user
    // ...
    
    Ok(token_response.access_token)
}
```

## Middleware Integration

```rust
use oxidite_middleware::AuthMiddleware;
use oxidite_auth::JwtManager;

let jwt_manager = JwtManager::new("secret");
let auth_middleware = AuthMiddleware::new(jwt_manager);

// In your router
// router.use_middleware(auth_middleware);
```

## Complete Example

```rust
use oxidite_auth::{JwtManager, Claims, hash_password, verify_password};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
    user: UserInfo,
}

async fn register(email: String, password: String) -> Result<(), Box<dyn std::error::Error>> {
    let hashed = hash_password(&password)?;
    
    // Store in database
    // db.execute(&format!(
    //     "INSERT INTO users (email, password_hash) VALUES ('{}', '{}')",
    //     email, hashed
    // )).await?;
    
    Ok(())
}

async fn login(
    req: LoginRequest,
    jwt: &JwtManager,
) -> Result<LoginResponse, Box<dyn std::error::Error>> {
    // Fetch user from database
    // let user = db.query_one(&format!(
    //     "SELECT id, email, password_hash FROM users WHERE email = '{}'",
    //     req.email
    // )).await?;
    
    // Verify password
    // let password_hash: String = user.try_get("password_hash")?;
    // if !verify_password(&req.password, &password_hash)? {
    //     return Err("Invalid credentials".into());
    // }
    
    // Create JWT
    let claims = Claims::new("user_id")
        .with_role("user")
        .with_expiration_minutes(60);
    
    let token = jwt.encode(&claims)?;
    
    Ok(LoginResponse {
        token,
        user: UserInfo {
            id: "user_id".to_string(),
            email: req.email,
        },
    })
}

#[derive(Serialize)]
struct UserInfo {
    id: String,
    email: String,
}
```

## Best Practices

1. **Use strong secrets** - Generate cryptographically secure keys for JWT
2. **Short expiration** - Keep JWT expiration times short (< 1 hour)
3. **HTTPS only** - Always use HTTPS in production
4. **Secure password storage** - Use Argon2 for password hashing
5. **Logout properly** - Invalidate sessions/tokens on logout
6. **Refresh tokens** - Implement refresh token rotation for long-lived sessions

## Next Steps

- [Database Guide](database.md) - Store user data
- [Realtime Guide](realtime.md) - Authenticate WebSocket connections
