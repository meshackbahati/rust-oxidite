# Authentication Guide

Complete guide to implementing authentication in Oxidite applications.

## Installation

```toml
[dependencies]
oxidite = { version = "1.0", features = ["auth", "database"] }
```

## Quick Start

This example demonstrates how to set up a simple login and a protected profile route.

### Main Application (`src/main.rs`)
```rust
use oxidite::prelude::*;
use oxidite_auth::AuthMiddleware;

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = Router::new();
    
    app.post("/login", login);
    app.get("/profile", profile).layer(AuthMiddleware);
    
    Server::new(app).listen("127.0.0.1:3000".parse()?).await
}
```

### Login Handler (`src/controllers/auth_controller.rs`)
```rust
use oxidite::prelude::*;
use oxidite_auth::{Auth, create_jwt, LoginRequest, TokenResponse};
use crate::models::user::User; // Your user model
use serde_json::json;

pub async fn login(Json(creds): Json<LoginRequest>) -> Result<OxiditeResponse> {
    // Verify credentials
    let user = User::find_by_email(&creds.email).await?;
    
    if !oxidite_auth::verify_password(&creds.password, &user.password_hash)? {
        return Err(Error::Unauthorized("Invalid credentials".to_string()));
    }
    
    // Create JWT
    let token = create_jwt(user.id, "your-secret-key")?;
    
    Ok(OxiditeResponse::json(json!(TokenResponse { token })))
}
```

### Protected Route (`src/controllers/user_controller.rs`)
```rust
use oxidite::prelude::*;
use oxidite_auth::Auth;
use crate::models::user::User;
use serde_json::json;

pub async fn profile(auth: Auth) -> Result<OxiditeResponse> {
    // The user is available via the Auth extractor
    let user = User::find(auth.user_id, &db).await?;
    Ok(OxiditeResponse::json(json!(user)))
}
```

## Password Hashing

Oxidite uses Argon2 for password hashing.

```rust
use oxidite_auth::hash_password;

// Hash password
let hash = hash_password("password123")?;

// Verification is handled by the login logic.
```

## JWT Authentication

The `AuthMiddleware` handles JWT verification automatically. You just need to provide a secret key in your configuration.

## RBAC (Roles & Permissions)

```rust
use oxidite_auth::RequirePermission;

// Middleware for a route that requires a specific permission
app.delete("/users/:id", delete_user)
    .layer(RequirePermission::new("users.delete"));
```

To assign roles and permissions, you'll typically interact with the database models provided by `oxidite-auth`.
