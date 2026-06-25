/// Password hashing and verification using Argon2id.
pub mod hasher;
/// JSON Web Token (JWT) creation and verification.
pub mod jwt;
/// Auth middleware for validating JWT tokens in HTTP requests.
pub mod middleware;
/// Role-based access control models (roles and permissions).
pub mod rbac;

/// Re-exports from the `hasher` module.
pub use hasher::{PasswordHasher, hash_password, verify_password};
/// Re-exports from the `jwt` module.
pub use jwt::{JwtManager, create_token, verify_token, Claims};
/// Re-export of the JWT auth middleware.
pub use middleware::AuthMiddleware;
/// Re-exports from the `rbac` module.
pub use rbac::{Role, Permission};

/// Session types and storage backends.
pub mod session;
/// Tower middleware and layer for session cookie management.
pub mod session_middleware;

/// Re-exports from the `session` module.
pub use session::{Session, SessionStore, InMemorySessionStore, RedisSessionStore, SessionManager};
/// Re-exports from the `session_middleware` module.
pub use session_middleware::{SessionMiddleware, SessionLayer};

/// OAuth2 client and server-side provider implementation.
pub mod oauth2;
/// Re-exports from the `oauth2` module.
pub use oauth2::{OAuth2Client, OAuth2Config, ProviderConfig, OAuth2Provider};

/// Authorization middleware and service for role/permission checks.
pub mod authorization;
/// Re-exports from the `authorization` module.
pub use authorization::{RequireRole, RequirePermission, AuthorizationService};

/// API key generation, hashing, and verification.
pub mod api_key;
/// Middleware to authenticate requests via API keys.
pub mod api_key_middleware;
/// Re-export of the `ApiKey` type.
pub use api_key::ApiKey;
/// Re-export of the `ApiKeyMiddleware` type.
pub use api_key_middleware::ApiKeyMiddleware;

/// Security utilities: email verification, password reset, and 2FA.
pub mod security;
/// Re-exports from the `security` module.
pub use security::{email_verification, password_reset, two_factor};

use thiserror::Error;

/// Authentication errors for the oxidite-auth crate.
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("Token expired")]
    TokenExpired,
    
    #[error("Invalid token")]
    InvalidToken,
    
    #[error("Missing authorization header")]
    MissingHeader,
    
    #[error("Permission denied")]
    PermissionDenied,
    
    #[error("Hash error: {0}")]
    HashError(String),

    #[error("Token error: {0}")]
    TokenError(String),
    
    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
}

/// Crate-level result alias using [`AuthError`].
pub type Result<T> = std::result::Result<T, AuthError>;
