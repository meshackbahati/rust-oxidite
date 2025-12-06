pub mod hasher;
pub mod jwt;
pub mod middleware;
pub mod rbac;

pub use hasher::{PasswordHasher, hash_password, verify_password};
pub use jwt::{JwtManager, create_token, verify_token, Claims};
pub use middleware::AuthMiddleware;
pub use rbac::{Role, Permission};

pub mod session;
pub mod session_middleware;

pub use session::{Session, SessionStore, InMemorySessionStore, RedisSessionStore, SessionManager};
pub use session_middleware::{SessionMiddleware, SessionLayer};

pub mod oauth2;
pub use oauth2::{OAuth2Client, OAuth2Config, ProviderConfig, OAuth2Provider};

pub mod authorization;
pub use authorization::{RequireRole, RequirePermission, AuthorizationService};

pub mod api_key;
pub mod api_key_middleware;
pub use api_key::ApiKey;
pub use api_key_middleware::ApiKeyMiddleware;

pub mod security;
pub use security::{email_verification, password_reset, two_factor};

use thiserror::Error;

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
    
    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
}

pub type Result<T> = std::result::Result<T, AuthError>;
