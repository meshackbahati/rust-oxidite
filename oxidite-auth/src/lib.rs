pub mod hasher;
pub mod jwt;
pub mod middleware;

pub use hasher::{PasswordHasher, hash_password, verify_password};
pub use jwt::{JwtToken, create_token, verify_token, Claims};
pub use middleware::AuthMiddleware;
pub mod session;
pub mod session_middleware;

pub use session::{Session, SessionStore, InMemorySessionStore, RedisSessionStore};
pub use session_middleware::{SessionMiddleware, SessionLayer};

pub mod oauth2;
pub use oauth2::{OAuth2Client, OAuth2Config, ProviderConfig, OAuth2Provider};

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
