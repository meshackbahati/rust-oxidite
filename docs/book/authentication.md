# Authentication

Authentication in Oxidite provides multiple methods to verify user identity. This chapter covers various authentication mechanisms including JWT, sessions, API keys, and OAuth2.

## Overview

Oxidite provides comprehensive authentication support including:
- JSON Web Tokens (JWT)
- Session-based authentication
- API key authentication
- OAuth2 integration
- Role-based access control (RBAC)
- Password hashing and verification

## JWT Authentication

JSON Web Tokens provide stateless authentication:

```rust
use oxidite::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,  // Subject (user ID)
    exp: i64,     // Expiration time
    iat: i64,     // Issued at time
    role: String, // User role
}

async fn generate_jwt(user_id: &str, role: &str) -> Result<String> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();
    
    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration,
        iat: Utc::now().timestamp(),
        role: role.to_string(),
    };
    
    // In a real app, use a proper JWT library like jsonwebtoken
    // This is a simplified example
    let token = create_jwt_token(&claims)?;
    Ok(token)
}

fn create_jwt_token(_claims: &Claims) -> Result<String> {
    // Implementation would use a proper JWT library
    Ok("fake.jwt.token".to_string())
}

async fn verify_jwt(token: &str) -> Result<Claims> {
    // In a real app, verify the JWT token
    // This is a simplified example
    verify_jwt_token(token)
}

fn verify_jwt_token(_token: &str) -> Result<Claims> {
    // Implementation would use a proper JWT library
    Ok(Claims {
        sub: "123".to_string(),
        exp: chrono::Utc::now().timestamp() + 86400, // 24 hours
        iat: chrono::Utc::now().timestamp(),
        role: "user".to_string(),
    })
}

// Login endpoint
#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

async fn login(Json(credentials): Json<LoginRequest>) -> Result<Response> {
    // Verify credentials (simplified)
    if verify_credentials(&credentials.username, &credentials.password).await {
        let token = generate_jwt(&credentials.username, "user").await?;
        
        Ok(Response::json(serde_json::json!({
            "token": token,
            "expires_in": 86400 // 24 hours in seconds
        })))
    } else {
        Err(Error::Unauthorized("Invalid credentials".to_string()))
    }
}

async fn verify_credentials(_username: &str, _password: &str) -> bool {
    // In a real app, verify against your user database
    _username == "admin" && _password == "password"
}
```

## JWT Middleware

Create middleware to protect routes with JWT authentication:

```rust
use oxidite::prelude::*;

async fn jwt_auth_middleware(req: Request, next: Next) -> Result<Response> {
    // Extract token from Authorization header
    let auth_header = req.headers()
        .get("authorization")
        .and_then(|hv| hv.to_str().ok());
    
    match auth_header {
        Some(auth) if auth.starts_with("Bearer ") => {
            let token = auth.trim_start_matches("Bearer ").trim();
            
            match verify_jwt(token).await {
                Ok(claims) => {
                    // Add user info to request extensions
                    let mut req = req;
                    req.extensions_mut().insert(AuthenticatedUser {
                        id: claims.sub,
                        role: claims.role,
                    });
                    
                    next.run(req).await
                }
                Err(_) => Err(Error::Unauthorized("Invalid or expired token".to_string())),
            }
        }
        _ => Err(Error::Unauthorized("Missing or invalid authorization header".to_string())),
    }
}

#[derive(Clone)]
struct AuthenticatedUser {
    id: String,
    role: String,
}

// Protected route using authenticated user
async fn protected_route(user: AuthenticatedUser) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "message": "Access granted",
        "user_id": user.id,
        "role": user.role
    })))
}
```

## Session Authentication

Session-based authentication stores user state on the server:

```rust
use oxidite::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone)]
struct SessionStore {
    sessions: Arc<Mutex<HashMap<String, Session>>>,
}

#[derive(Clone)]
struct Session {
    user_id: String,
    role: String,
    expires_at: u64,
}

impl SessionStore {
    fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    fn create_session(&self, user_id: String, role: String) -> String {
        let session_id = generate_session_id();
        let expires_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() + (24 * 3600); // 24 hours
        
        let session = Session {
            user_id,
            role,
            expires_at,
        };
        
        self.sessions.lock().unwrap().insert(session_id.clone(), session);
        session_id
    }
    
    fn validate_session(&self, session_id: &str) -> Option<Session> {
        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(session_id) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            if now < session.expires_at {
                Some(session.clone())
            } else {
                // Session expired, remove it
                drop(sessions); // Release the lock
                self.sessions.lock().unwrap().remove(session_id);
                None
            }
        } else {
            None
        }
    }
    
    fn destroy_session(&self, session_id: &str) {
        self.sessions.lock().unwrap().remove(session_id);
    }
}

fn generate_session_id() -> String {
    use uuid::Uuid;
    Uuid::new_v4().to_string()
}

// Session authentication middleware
async fn session_auth_middleware(
    req: Request,
    next: Next,
    State(session_store): State<Arc<SessionStore>>
) -> Result<Response> {
    // Get session ID from cookies
    let cookies = Cookies::from_request(&req).await?;
    let session_id = cookies.get("session_id");
    
    match session_id {
        Some(id) => {
            if let Some(session) = session_store.validate_session(id) {
                let mut req = req;
                req.extensions_mut().insert(AuthenticatedUser {
                    id: session.user_id,
                    role: session.role,
                });
                
                next.run(req).await
            } else {
                Err(Error::Unauthorized("Invalid or expired session".to_string()))
            }
        }
        None => Err(Error::Unauthorized("No session found".to_string())),
    }
}

// Login handler for session-based auth
async fn session_login(
    Json(credentials): Json<LoginRequest>,
    State(session_store): State<Arc<SessionStore>>
) -> Result<Response> {
    if verify_credentials(&credentials.username, &credentials.password).await {
        let session_id = session_store.create_session(
            credentials.username,
            "user".to_string()
        );
        
        // Create response with session cookie
        let mut response = Response::json(serde_json::json!({
            "message": "Login successful",
            "session_id": session_id
        }));
        
        // Add session cookie
        use http::header::{SET_COOKIE, HeaderValue};
        let cookie_header = format!("session_id={}; HttpOnly; Secure; Max-Age={}; Path=/", 
            session_id, 24 * 3600); // 24 hours
        response.headers_mut().insert(SET_COOKIE, HeaderValue::from_str(&cookie_header).unwrap());
        
        Ok(response)
    } else {
        Err(Error::Unauthorized("Invalid credentials".to_string()))
    }
}
```

## API Key Authentication

API key authentication for service-to-service communication:

```rust
use oxidite::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct ApiKeyStore {
    keys: Arc<Mutex<HashMap<String, ApiKey>>>,
}

#[derive(Clone)]
struct ApiKey {
    user_id: String,
    permissions: Vec<String>,
    created_at: String,
}

impl ApiKeyStore {
    fn new() -> Self {
        let mut keys = HashMap::new();
        
        // Add some example keys (in a real app, load from database)
        keys.insert(
            "sk_live_abc123".to_string(),
            ApiKey {
                user_id: "user123".to_string(),
                permissions: vec!["read".to_string(), "write".to_string()],
                created_at: chrono::Utc::now().to_rfc3339(),
            }
        );
        
        Self {
            keys: Arc::new(Mutex::new(keys)),
        }
    }
    
    fn validate_key(&self, key: &str) -> Option<ApiKey> {
        let keys = self.keys.lock().unwrap();
        keys.get(key).cloned()
    }
}

// API key authentication middleware
async fn api_key_auth_middleware(
    req: Request,
    next: Next,
    State(api_keys): State<Arc<ApiKeyStore>>
) -> Result<Response> {
    // Check for API key in header
    let auth_header = req.headers()
        .get("authorization")
        .and_then(|hv| hv.to_str().ok());
    
    if let Some(auth) = auth_header {
        let api_key = if auth.starts_with("Bearer ") {
            auth.trim_start_matches("Bearer ").trim()
        } else {
            auth
        };
        
        if let Some(key_info) = api_keys.validate_key(api_key) {
            let mut req = req;
            req.extensions_mut().insert(ApiKeyUser {
                user_id: key_info.user_id,
                permissions: key_info.permissions,
            });
            
            return next.run(req).await;
        }
    }
    
    // Check for API key in query parameter as fallback
    use serde::Deserialize;
    #[derive(Deserialize)]
    struct ApiKeyQuery {
        api_key: Option<String>,
    }
    
    if let Ok(Query(query)) = Query::<ApiKeyQuery>::from_request(&req).await {
        if let Some(api_key) = query.api_key {
            if let Some(key_info) = api_keys.validate_key(&api_key) {
                let mut req = req;
                req.extensions_mut().insert(ApiKeyUser {
                    user_id: key_info.user_id,
                    permissions: key_info.permissions,
                });
                
                return next.run(req).await;
            }
        }
    }
    
    Err(Error::Unauthorized("Invalid API key".to_string()))
}

#[derive(Clone)]
struct ApiKeyUser {
    user_id: String,
    permissions: Vec<String>,
}

// Protected endpoint for API key users
async fn api_protected_route(user: ApiKeyUser) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "message": "API access granted",
        "user_id": user.user_id,
        "permissions": user.permissions
    })))
}
```

## Password Hashing

Secure password handling with hashing:

```rust
use oxidite::prelude::*;

pub struct PasswordHasher;

impl PasswordHasher {
    pub fn hash(password: &str) -> Result<String> {
        // In a real app, use a proper hashing library like argon2 or bcrypt
        // This is a placeholder implementation
        use sha2::{Sha256, Digest};
        
        let salt = generate_salt();
        let mut hasher = Sha256::new();
        
        hasher.update(password.as_bytes());
        hasher.update(&salt);
        
        let hash = hasher.finalize();
        let hash_hex = format!("{:x}", hash);
        
        Ok(format!("sha256:{}:{}", base64::encode(&salt), hash_hex))
    }
    
    pub fn verify(password: &str, hashed: &str) -> Result<bool> {
        if !hashed.starts_with("sha256:") {
            return Err(Error::Server("Unsupported hash format".to_string()));
        }
        
        let parts: Vec<&str> = hashed.split(':').collect();
        if parts.len() != 3 {
            return Err(Error::Server("Invalid hash format".to_string()));
        }
        
        let salt = base64::decode(parts[1]).map_err(|_| Error::Server("Invalid salt".to_string()))?;
        
        let mut hasher = sha2::Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(&salt);
        let hash = hasher.finalize();
        let hash_hex = format!("{:x}", hash);
        
        Ok(hash_hex == parts[2])
    }
}

fn generate_salt() -> Vec<u8> {
    use rand::RngCore;
    let mut salt = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt);
    salt.to_vec()
}

// Example usage in user registration
#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}

async fn register_user(Json(registration): Json<RegisterRequest>) -> Result<Response> {
    // Hash the password
    let password_hash = PasswordHasher::hash(&registration.password)?;
    
    // Save user to database (simplified)
    let user = UserRegistration {
        username: registration.username,
        email: registration.email,
        password_hash,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    
    // In a real app, save to database
    save_user_to_db(user).await?;
    
    Ok(Response::json(serde_json::json!({
        "message": "User registered successfully"
    })))
}

#[derive(Clone)]
struct UserRegistration {
    username: String,
    email: String,
    password_hash: String,
    created_at: String,
}

async fn save_user_to_db(_user: UserRegistration) -> Result<()> {
    // Implementation would save to database
    Ok(())
}
```

## OAuth2 Integration

OAuth2 support for third-party authentication:

```rust
use oxidite::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct OAuthCallback {
    code: String,
    state: Option<String>,
}

// OAuth2 redirect for Google (example)
async fn google_oauth_redirect(_req: Request) -> Result<Response> {
    let client_id = std::env::var("GOOGLE_CLIENT_ID").unwrap();
    let redirect_uri = "http://localhost:3000/auth/google/callback";
    let scopes = "email profile";
    let state = generate_state();
    
    let auth_url = format!(
        "https://accounts.google.com/o/oauth2/auth?client_id={}&redirect_uri={}&scope={}&response_type=code&state={}",
        client_id, redirect_uri, scopes, state
    );
    
    // In a real app, redirect to the auth URL
    Ok(Response::json(serde_json::json!({
        "redirect_url": auth_url,
        "state": state
    })))
}

// OAuth2 callback handler
async fn google_oauth_callback(
    Query(params): Query<OAuthCallback>,
    State(session_store): State<Arc<SessionStore>>
) -> Result<Response> {
    // Verify state parameter (security measure)
    if let Some(expected_state) = params.state {
        if !validate_state(&expected_state).await {
            return Err(Error::BadRequest("Invalid state parameter".to_string()));
        }
    }
    
    // Exchange code for token
    let token_response = exchange_code_for_token(&params.code).await?;
    
    // Get user info from Google
    let user_info = get_google_user_info(&token_response.access_token).await?;
    
    // Create session for the user
    let session_id = session_store.create_session(
        user_info.id.clone(),
        "oauth_user".to_string()
    );
    
    // Return session info or redirect
    Ok(Response::json(serde_json::json!({
        "message": "Authentication successful",
        "session_id": session_id,
        "user": user_info
    })))
}

struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u32,
}

struct GoogleUserInfo {
    id: String,
    email: String,
    name: String,
    verified_email: bool,
}

async fn exchange_code_for_token(_code: &str) -> Result<TokenResponse> {
    // In a real app, make HTTP request to token endpoint
    Ok(TokenResponse {
        access_token: "fake_access_token".to_string(),
        token_type: "Bearer".to_string(),
        expires_in: 3600,
    })
}

async fn get_google_user_info(_access_token: &str) -> Result<GoogleUserInfo> {
    // In a real app, make HTTP request to userinfo endpoint
    Ok(GoogleUserInfo {
        id: "google_user_123".to_string(),
        email: "user@example.com".to_string(),
        name: "Google User".to_string(),
        verified_email: true,
    })
}

fn generate_state() -> String {
    use uuid::Uuid;
    Uuid::new_v4().to_string()
}

async fn validate_state(_state: &str) -> bool {
    // In a real app, verify against stored states
    true
}
```

## Role-Based Access Control (RBAC)

Implement role-based access control:

```rust
use oxidite::prelude::*;
use std::collections::HashSet;

#[derive(Clone)]
struct Permission {
    resource: String,
    action: String,
}

#[derive(Clone)]
struct Role {
    name: String,
    permissions: Vec<Permission>,
}

#[derive(Clone)]
struct UserRole {
    user_id: String,
    role: String,
}

#[derive(Clone)]
struct RbacStore {
    roles: Vec<Role>,
    user_roles: Vec<UserRole>,
}

impl RbacStore {
    fn new() -> Self {
        // Define roles and permissions
        let admin_role = Role {
            name: "admin".to_string(),
            permissions: vec![
                Permission { resource: "users".to_string(), action: "read".to_string() },
                Permission { resource: "users".to_string(), action: "write".to_string() },
                Permission { resource: "users".to_string(), action: "delete".to_string() },
                Permission { resource: "posts".to_string(), action: "read".to_string() },
                Permission { resource: "posts".to_string(), action: "write".to_string() },
            ],
        };
        
        let user_role = Role {
            name: "user".to_string(),
            permissions: vec![
                Permission { resource: "users".to_string(), action: "read".to_string() },
                Permission { resource: "posts".to_string(), action: "read".to_string() },
                Permission { resource: "posts".to_string(), action: "write".to_string() },
            ],
        };
        
        Self {
            roles: vec![admin_role, user_role],
            user_roles: vec![
                UserRole { user_id: "admin123".to_string(), role: "admin".to_string() },
                UserRole { user_id: "user456".to_string(), role: "user".to_string() },
            ],
        }
    }
    
    fn user_has_permission(&self, user_id: &str, resource: &str, action: &str) -> bool {
        // Get user's roles
        let user_roles: Vec<&str> = self.user_roles
            .iter()
            .filter(|ur| ur.user_id == user_id)
            .map(|ur| ur.role.as_str())
            .collect();
        
        // Check if any role grants the required permission
        for role_name in user_roles {
            if let Some(role) = self.roles.iter().find(|r| r.name == role_name) {
                if role.permissions.iter().any(|perm| {
                    perm.resource == resource && perm.action == action
                }) {
                    return true;
                }
            }
        }
        
        false
    }
}

// RBAC middleware
async fn rbac_middleware(
    req: Request,
    next: Next,
    State(rbac): State<Arc<RbacStore>>
) -> Result<Response> {
    // Get authenticated user from request extensions
    if let Some(user) = req.extensions().get::<AuthenticatedUser>() {
        // Extract resource and action from the request
        let resource = extract_resource_from_path(req.uri().path());
        let action = req.method().as_str().to_lowercase();
        
        if rbac.user_has_permission(&user.id, &resource, &action) {
            return next.run(req).await;
        }
        
        return Err(Error::Forbidden("Insufficient permissions".to_string()));
    }
    
    Err(Error::Unauthorized("User not authenticated".to_string()))
}

fn extract_resource_from_path(path: &str) -> String {
    // Simplified resource extraction
    // In a real app, you'd have more sophisticated routing
    path.split('/').nth(1).unwrap_or("").to_string()
}

// Example route with RBAC protection
async fn admin_only_route(user: AuthenticatedUser) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "message": "Admin access granted",
        "user_id": user.id
    })))
}
```

## Two-Factor Authentication (2FA)

Implement two-factor authentication:

```rust
use oxidite::prelude::*;
use qrcode::QrCode;
use base32;

#[derive(Clone)]
struct TotpSecret {
    secret: String,
    user_id: String,
    verified: bool,
}

#[derive(Clone)]
struct TotpStore {
    secrets: std::sync::Arc<Mutex<HashMap<String, TotpSecret>>>,
}

impl TotpStore {
    fn new() -> Self {
        Self {
            secrets: std::sync::Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    fn generate_secret(&self, user_id: &str) -> String {
        // Generate a random secret
        let secret: Vec<u8> = (0..32).map(|_| rand::random::<u8>()).collect();
        let secret_base32 = base32::encode(base32::Alphabet::RFC4648 { padding: false }, &secret);
        
        // Store the secret
        self.secrets.lock().unwrap().insert(
            user_id.to_string(),
            TotpSecret {
                secret: secret_base32.clone(),
                user_id: user_id.to_string(),
                verified: false,
            }
        );
        
        secret_base32
    }
    
    fn verify_token(&self, user_id: &str, token: &str) -> bool {
        // In a real app, verify the TOTP token using the stored secret
        // This is a simplified check
        token.len() == 6 && token.chars().all(|c| c.is_ascii_digit())
    }
    
    fn enable_2fa(&self, user_id: &str) {
        let mut secrets = self.secrets.lock().unwrap();
        if let Some(secret) = secrets.get_mut(user_id) {
            secret.verified = true;
        }
    }
}

// Generate 2FA setup
async fn generate_2fa_setup(
    user: AuthenticatedUser,
    State(totp_store): State<Arc<TotpStore>>
) -> Result<Response> {
    let secret = totp_store.generate_secret(&user.id);
    let issuer = "Oxidite App";
    let account = &user.id;
    
    // Generate QR code URL
    let otpauth_url = format!(
        "otpauth://totp/{}:{}?secret={}&issuer={}",
        issuer, account, secret, issuer
    );
    
    // Generate QR code
    let qr_code = QrCode::new(otpauth_url.as_bytes()).unwrap();
    let qr_string = qr_code
        .render::<unicode_art::Dense1x2>()
        .dark_color(unicode_art::Color::Ascii(' '))
        .light_color(unicode_art::Color::Ascii('█'))
        .build();
    
    Ok(Response::json(serde_json::json!({
        "secret": secret,
        "qr_code": qr_string,
        "manual_entry": format!("{} {}", issuer, account)
    })))
}

// Verify 2FA token
#[derive(Deserialize)]
struct Verify2faRequest {
    token: String,
}

async fn verify_2fa(
    user: AuthenticatedUser,
    Json(payload): Json<Verify2faRequest>,
    State(totp_store): State<Arc<TotpStore>>
) -> Result<Response> {
    if totp_store.verify_token(&user.id, &payload.token) {
        totp_store.enable_2fa(&user.id);
        
        Ok(Response::json(serde_json::json!({
            "message": "2FA enabled successfully"
        })))
    } else {
        Err(Error::BadRequest("Invalid 2FA token".to_string()))
    }
}

// 2FA middleware
async fn twofa_middleware(
    req: Request,
    next: Next,
    State(totp_store): State<Arc<TotpStore>>
) -> Result<Response> {
    // Check if user has 2FA enabled
    if let Some(user) = req.extensions().get::<AuthenticatedUser>() {
        let secrets = totp_store.secrets.lock().unwrap();
        if let Some(secret) = secrets.get(&user.id) {
            if !secret.verified {
                // 2FA not verified yet
                return Err(Error::Unauthorized("2FA verification required".to_string()));
            }
        }
    }
    
    next.run(req).await
}
```

## Security Best Practices

### 1. Secure Token Storage
```rust
// Store tokens securely, never in plain text
const TOKEN_LENGTH: usize = 32; // 256 bits

fn generate_secure_token() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; TOKEN_LENGTH];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}
```

### 2. Rate Limiting for Auth Attempts
```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone)]
struct RateLimiter {
    attempts: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_attempts: u32,
    window: Duration,
}

impl RateLimiter {
    fn new(max_attempts: u32, window_minutes: u64) -> Self {
        Self {
            attempts: Arc::new(Mutex::new(HashMap::new())),
            max_attempts,
            window: Duration::from_secs(window_minutes * 60),
        }
    }
    
    fn is_allowed(&self, identifier: &str) -> bool {
        let mut attempts = self.attempts.lock().unwrap();
        let now = Instant::now();
        let window_start = now - self.window;
        
        // Clean old attempts
        if let Some(times) = attempts.get_mut(identifier) {
            times.retain(|time| *time > window_start);
        }
        
        // Check limit
        let count = attempts.entry(identifier.to_string())
            .or_insert_with(Vec::new)
            .len();
        
        if count < self.max_attempts as usize {
            attempts.get_mut(identifier).unwrap().push(now);
            true
        } else {
            false
        }
    }
}
```

### 3. Session Regeneration
```rust
// Regenerate session ID after login to prevent session fixation
async fn regenerate_session(
    old_session_id: &str,
    new_user_info: AuthenticatedUser,
    session_store: &SessionStore
) -> String {
    session_store.destroy_session(old_session_id);
    // Create new session with new ID
    session_store.create_session(new_user_info.id, new_user_info.role)
}
```

## Summary

Authentication in Oxidite provides multiple secure methods:

- **JWT**: Stateless authentication with tokens
- **Sessions**: Server-side state management
- **API Keys**: Service-to-service authentication
- **Password Hashing**: Secure credential storage
- **OAuth2**: Third-party authentication integration
- **RBAC**: Role-based access control
- **2FA**: Two-factor authentication support

Key security practices include:
- Using strong password hashing
- Implementing rate limiting
- Securing token storage and transmission
- Validating all inputs
- Following authentication best practices

Choose the appropriate authentication method based on your application's requirements and security needs.