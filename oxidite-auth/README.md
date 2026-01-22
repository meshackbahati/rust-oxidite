# oxidite-auth

Authentication and authorization for Oxidite (RBAC, JWT, OAuth2, 2FA, API keys).

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/oxidite-auth.svg)](https://crates.io/crates/oxidite-auth)
[![Docs.rs](https://docs.rs/oxidite-auth/badge.svg)](https://docs.rs/oxidite-auth)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

</div>

## Overview

`oxidite-auth` provides a comprehensive authentication and authorization system for the Oxidite web framework. It includes JWT token management, secure password hashing, role-based access control, API key authentication, and two-factor authentication.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
oxidite-auth = "0.1"
```

## Features

- **JWT token management** - Secure JSON Web Token generation and verification
- **Password hashing** - Industry-standard Argon2 password hashing
- **Role-Based Access Control (RBAC)** - Flexible role and permission system
- **API key authentication** - Secure API key generation and validation
- **Two-Factor Authentication (2FA)** - TOTP-based second factor authentication
- **OAuth2 integration** - Support for popular OAuth2 providers
- **Email verification** - Token-based email verification system
- **Password reset** - Secure password reset functionality
- **Rate limiting** - Account-based rate limiting to prevent abuse

## Usage

### JWT Authentication

Secure user authentication with JSON Web Tokens:

```rust
use oxidite_auth::{JwtManager, create_token, verify_token, Claims};

// Initialize JWT manager
let jwt_manager = JwtManager::new("your-secret-key".to_string());

// Create a token for a user
let claims = Claims {
    sub: "user-id".to_string(),
    exp: 1234567890, // Unix timestamp (will be set automatically in practice)
    ..Default::default()
};

let token = create_token(&jwt_manager, claims)?;

// Verify and extract claims from a token
let verified_claims = verify_token(&jwt_manager, &token)?;
```

### Password Hashing

Secure password storage using industry-standard Argon2:

```rust
use oxidite_auth::security::hasher::Hasher;

let hasher = Hasher::new();

// Hash a password
let password_hash = hasher.hash("user-password")?;

// Verify a password against its hash
let is_valid = hasher.verify(&password_hash, "user-password")?;

if is_valid {
    println!("Password is valid!");
} else {
    println!("Invalid password!");
}
```

### Role-Based Access Control (RBAC)

Manage roles and permissions for fine-grained access control:

```rust
use oxidite_auth::authorization::{Role, Permission};

// Create roles with permissions
let admin_role = Role {
    name: "admin".to_string(),
    permissions: vec![
        Permission::new("users.create"),
        Permission::new("users.read"),
        Permission::new("users.update"),
        Permission::new("users.delete"),
    ],
};

// Check if a user has a specific permission
if admin_role.has_permission("users.delete") {
    // Allow the delete operation
    println!("User has permission to delete");
}
```

### API Key Authentication

Secure API access with API key management:

```rust
use oxidite_auth::api_key::{ApiKey, ApiKeyManager};

let manager = ApiKeyManager::new();

// Generate a new API key for a user
let api_key = manager.generate_key("user-id")?;

// Validate an API key
let is_valid = manager.validate_key(&api_key.key).await?;

if is_valid {
    println!("API key is valid!");
} else {
    println!("Invalid API key!");
}
```

### Two-Factor Authentication (2FA)

Enhance security with TOTP-based two-factor authentication:

```rust
use oxidite_auth::security::totp::{generate_secret, verify_code};

// Generate a secret for a user
let secret = generate_secret();

// The user receives this secret and sets up their authenticator app
println!("Secret: {}", secret);

// Later, verify a code from the user's authenticator app
let user_code = "123456"; // Code entered by user
let is_valid = verify_code(&secret, user_code)?;

if is_valid {
    println!("2FA code is valid!");
} else {
    println!("Invalid 2FA code!");
}
```

### OAuth2 Integration

Integrate with popular OAuth2 providers:

```rust
use oxidite_auth::oauth2::{GoogleProvider, OAuth2Config};

let config = OAuth2Config {
    client_id: "your-client-id".to_string(),
    client_secret: "your-client-secret".to_string(),
    redirect_uri: "http://localhost:3000/auth/callback".to_string(),
};

let google_provider = GoogleProvider::new(config);

// Generate authorization URL
let auth_url = google_provider.authorize_url();

// After user authorization, exchange code for token
// let token = google_provider.exchange_code("authorization-code").await?;
```

### Authorization Middleware

Protect routes with authentication and authorization checks:

```rust
use oxidite_core::{Request, Response, Middleware};
use oxidite_auth::middleware::AuthMiddleware;

// Use authentication middleware to protect routes
// This would typically be integrated with Oxidite's middleware system
```

## Security Best Practices

- Always use strong, randomly generated secrets for JWT signing
- Implement proper rate limiting to prevent brute force attacks
- Store sensitive data securely and encrypt at rest when possible
- Regularly rotate secrets and API keys
- Use HTTPS in production environments
- Validate and sanitize all user inputs

## License

MIT
