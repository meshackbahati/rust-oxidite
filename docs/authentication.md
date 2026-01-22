# Authentication and Authorization

Oxidite provides comprehensive authentication and authorization features to secure your web applications.

## Overview

The authentication module includes:

- JWT token authentication
- Session management
- OAuth2 integration
- API key authentication
- Role-based access control (RBAC)
- Password hashing and verification
- Email verification
- Password reset
- Two-factor authentication (2FA)

## Installation

Add the auth feature to your `Cargo.toml`:

```toml
[dependencies]
oxidite = { version = "1.0", features = ["auth"] }
```

## JWT Authentication

JSON Web Tokens (JWT) provide stateless authentication for your applications.

### Setting up JWT Manager

```rust
use oxidite::auth::JwtManager;

let jwt_manager = JwtManager::new("your-super-secret-key-here".to_string());
```

### Creating Tokens

```rust
use oxidite::auth::{JwtManager, create_token, Claims};

let jwt_manager = JwtManager::new("secret-key".to_string());

let claims = Claims {
    sub: "user-id".to_string(),
    exp: 1234567890,  // Unix timestamp for expiration
    iat: 1234567800,  // Unix timestamp for issued at
    ..Default::default()  // Other default claims
};

let token = create_token(&jwt_manager, claims)?;
println!("Generated token: {}", token);
```

### Verifying Tokens

```rust
use oxidite::auth::{JwtManager, verify_token, Claims};

let jwt_manager = JwtManager::new("secret-key".to_string());
let token = "your.jwt.token.here";

match verify_token(&jwt_manager, &token) {
    Ok(claims) => {
        println!("Valid token for user: {}", claims.sub);
    }
    Err(e) => {
        println!("Invalid token: {}", e);
    }
}
```

### JWT Claims

The `Claims` struct contains standard JWT claims:

```rust
use oxidite::auth::Claims;

let claims = Claims {
    sub: "user-id".to_string(),      // Subject (user identifier)
    exp: 1234567890,                 // Expiration time (Unix timestamp)
    iat: 1234567800,                 // Issued at time (Unix timestamp)
    iss: "your-app".to_string(),     // Issuer
    aud: "your-audience".to_string(), // Audience
};
```

### JWT in Request Handlers

```rust
use oxidite::prelude::*;
use oxidite::auth::{JwtManager, verify_token};

async fn protected_handler(
    mut req: Request,
    jwt_manager: &JwtManager
) -> Result<Response> {
    // Extract token from Authorization header
    if let Some(auth_header) = req.headers().get("authorization") {
        let auth_str = auth_header.to_str().unwrap_or("");
        if auth_str.starts_with("Bearer ") {
            let token = &auth_str[7..];  // Remove "Bearer " prefix
            
            match verify_token(jwt_manager, token) {
                Ok(claims) => {
                    // Token is valid, proceed with request
                    Ok(response::json(serde_json::json!({
                        "message": "Access granted",
                        "user_id": claims.sub
                    })))
                }
                Err(_) => Err(Error::Unauthorized("Invalid token".to_string())),
            }
        } else {
            Err(Error::Unauthorized("Invalid authorization format".to_string()))
        }
    } else {
        Err(Error::Unauthorized("Missing authorization header".to_string()))
    }
}
```

## Session Management

Oxidite provides session management for stateful authentication.

### Session Stores

Oxidite supports different session storage backends:

```rust
use oxidite::auth::{SessionManager, InMemorySessionStore, RedisSessionStore};

// In-memory store (for development)
let session_store = InMemorySessionStore::new();
let session_manager = SessionManager::new(session_store);

// Redis store (for production)
// let redis_store = RedisSessionStore::new("redis://127.0.0.1:6379").await?;
// let session_manager = SessionManager::new(redis_store);
```

### Working with Sessions

```rust
use oxidite::auth::Session;

async fn login_handler(
    mut req: Request,
    session_manager: &SessionManager
) -> Result<Response> {
    // After successful authentication
    let mut session = session_manager.create_session().await?;
    session.set("user_id", "12345");
    session.set("username", "john_doe");
    
    // Save session and get session ID
    let session_id = session.save().await?;
    
    // Return session ID in response (typically stored in cookie)
    let mut response = response::json(serde_json::json!({
        "message": "Login successful",
        "session_id": session_id
    }));
    
    // Set session ID in cookie
    use http::header::{SET_COOKIE, HeaderValue};
    let cookie_value = format!("session_id={}; HttpOnly; Path=/", session_id);
    response.headers_mut().insert(SET_COOKIE, HeaderValue::from_str(&cookie_value)?);
    
    Ok(response)
}

async fn profile_handler(
    cookies: Cookies,
    session_manager: &SessionManager
) -> Result<Response> {
    // get session id from cookie
    if let Some(session_id) = cookies.get("session_id") {
        if let Some(mut session) = session_manager.get_session(session_id).await? {
            if let Some(user_id) = session.get("user_id") {
                return Ok(response::json(serde_json::json!({
                    "user_id": user_id,
                    "username": session.get("username")
                })));
            }
        }
    }
    
    Err(Error::Unauthorized("not authenticated".to_string()))
}
```

## Password Hashing

Oxidite provides secure password hashing and verification:

```rust
use oxidite::auth::{hash_password, verify_password};

// Hash a password
let password = "user-password";
let hashed = hash_password(password)?;

// Verify a password
if verify_password(password, &hashed)? {
    println!("Password is correct");
} else {
    println!("Password is incorrect");
}
```

## API Key Authentication

API keys provide an alternative authentication method for machine-to-machine communication:

```rust
use oxidite::auth::ApiKey;

// Create an API key
let api_key = ApiKey::generate("user-id", Some("description"))?;
let key_value = api_key.key();

// Verify an API key
if let Some(parsed_key) = ApiKey::parse(key_value) {
    if parsed_key.verify("user-id") {
        println!("Valid API key for user: {}", parsed_key.user_id());
    }
}
```

## Role-Based Access Control (RBAC)

Oxidite includes an RBAC system for fine-grained permission control:

```rust
use oxidite::auth::{Role, Permission};

// Define roles and permissions
let admin_role = Role::new("admin");
let user_role = Role::new("user");

let read_permission = Permission::new("read");
let write_permission = Permission::new("write");
let delete_permission = Permission::new("delete");

// Assign permissions to roles
admin_role.add_permission(read_permission);
admin_role.add_permission(write_permission);
admin_role.add_permission(delete_permission);

user_role.add_permission(read_permission);
```

### Authorization Middleware

```rust
use oxidite::prelude::*;
use oxidite::auth::{RequireRole, RequirePermission};

// Handler that requires admin role
async fn admin_handler(
    _req: Request
) -> Result<Response> {
    Ok(response::json(serde_json::json!({
        "message": "Admin access granted"
    })))
}

// Apply role requirement
let mut router = Router::new();
router.get("/admin", RequireRole::new("admin", admin_handler));
```

## OAuth2 Integration

Oxidite provides OAuth2 integration for third-party authentication:

```rust
use oxidite::auth::{OAuth2Client, OAuth2Config, OAuth2Provider};

// Configure OAuth2 with Google
let google_config = OAuth2Config {
    client_id: "your-client-id".to_string(),
    client_secret: "your-client-secret".to_string(),
    redirect_uri: "http://localhost:3000/auth/callback".to_string(),
    scopes: vec!["openid", "profile", "email"],
};

let oauth_client = OAuth2Client::new(OAuth2Provider::Google, google_config);

// Generate authorization URL
let auth_url = oauth_client.authorization_url()?;

// Handle callback after user authorization
async fn oauth_callback(
    Query(params): Query<serde_json::Value>,
    oauth_client: &OAuth2Client
) -> Result<OxiditeResponse> {
    let code = params["code"].as_str().unwrap_or("");
    
    match oauth_client.exchange_code(code).await {
        Ok(token) => {
            // Get user info
            let user_info = oauth_client.get_user_info(&token.access_token).await?;
            
            Ok(response::json(serde_json::json!({
                "message": "OAuth2 login successful",
                "user": user_info
            })))
        }
        Err(e) => Err(Error::Unauthorized(format!("OAuth2 error: {}", e))),
    }
}
```

## Complete Authentication Example

Here's a complete example showing user registration, login, and protected routes:

```rust
use oxidite::prelude::*;
use oxidite::auth::{JwtManager, hash_password, verify_password, create_token, Claims};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct AppState {
    jwt_manager: JwtManager,
    // In a real app, you'd use a proper database
    users: Arc<Mutex<HashMap<String, User>>>,
}

#[derive(Serialize, Deserialize, Clone)]
struct User {
    id: String,
    email: String,
    password_hash: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct RegisterRequest {
    email: String,
    password: String,
}

async fn register_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>
) -> Result<OxiditeResponse> {
    // Check if user already exists
    {
        let users = state.users.lock().unwrap();
        if users.values().any(|u| u.email == payload.email) {
            return Err(Error::BadRequest("User already exists".to_string()));
        }
    }
    
    // Hash password
    let password_hash = hash_password(&payload.password)?;
    
    // Create user
    let user = User {
        id: uuid::Uuid::new_v4().to_string(),
        email: payload.email,
        password_hash,
    };
    
    // Save user
    {
        let mut users = state.users.lock().unwrap();
        users.insert(user.id.clone(), user.clone());
    }
    
    Ok(response::json(serde_json::json!({
        "message": "User registered successfully",
        "user_id": user.id
    })))
}

async fn login_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>
) -> Result<OxiditeResponse> {
    // Find user by email
    let user = {
        let users = state.users.lock().unwrap();
        users.values().find(|u| u.email == payload.email).cloned()
    };
    
    if let Some(user) = user {
        // Verify password
        if verify_password(&payload.password, &user.password_hash)? {
            // Create JWT token
            let claims = Claims {
                sub: user.id.clone(),
                exp: (chrono::Utc::now() + chrono::Duration::days(7)).timestamp() as u64,
                ..Default::default()
            };
            
            let token = create_token(&state.jwt_manager, claims)?;
            
            return Ok(response::json(serde_json::json!({
                "message": "Login successful",
                "token": token,
                "user_id": user.id
            })));
        }
    }
    
    Err(Error::Unauthorized("Invalid credentials".to_string()))
}

async fn protected_handler(
    State(state): State<Arc<AppState>>,
    mut req: OxiditeRequest
) -> Result<OxiditeResponse> {
    // Extract and verify JWT token
    if let Some(auth_header) = req.headers().get("authorization") {
        let auth_str = auth_header.to_str().unwrap_or("");
        if auth_str.starts_with("Bearer ") {
            let token = &auth_str[7..];
            
            match verify_token(&state.jwt_manager, token) {
                Ok(claims) => {
                    return Ok(response::json(serde_json::json!({
                        "message": "Access to protected resource granted",
                        "user_id": claims.sub
                    })));
                }
                Err(_) => {
                    return Err(Error::Unauthorized("Invalid token".to_string()));
                }
            }
        }
    }
    
    Err(Error::Unauthorized("Missing or invalid authorization header".to_string()))
}

#[tokio::main]
async fn main() -> Result<()> {
    let jwt_manager = JwtManager::new("your-super-secret-key-change-in-production".to_string());
    let app_state = Arc::new(AppState {
        jwt_manager,
        users: Arc::new(Mutex::new(HashMap::new())),
    });
    
    let mut router = Router::new();
    router.post("/register", register_handler);
    router.post("/login", login_handler);
    router.get("/protected", protected_handler);
    
    let service = ServiceBuilder::new()
        .layer(AddExtensionLayer::new(app_state))
        .service(router);
    
    Server::new(service)
        .listen("127.0.0.1:3000".parse().unwrap())
        .await
}
```

## Security Best Practices

1. **Use strong secrets** for JWT signing and session management
2. **Implement proper rate limiting** to prevent brute-force attacks
3. **Use HTTPS in production** to encrypt all authentication data
4. **Hash passwords** with bcrypt or Argon2
5. **Validate and sanitize** all user inputs
6. **Implement proper session timeout** and cleanup mechanisms
7. **Use secure cookie settings** (HttpOnly, Secure, SameSite)
8. **Regularly rotate** secrets and API keys
9. **Log authentication attempts** for security monitoring
10. **Implement multi-factor authentication** for sensitive operations