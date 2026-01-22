use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Internal server error: {0}")]
    InternalServerError(String),
    #[error("Server error: {0}")]
    Server(String),
    #[error("Resource not found: {0}")]
    NotFound(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Unauthorized access: {0}")]
    Unauthorized(String),
    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Resource conflict: {0}")]
    Conflict(String),
    #[error("Validation failed: {0}")]
    Validation(String),
    #[error("Rate limit exceeded: {0}")]
    RateLimited(String),
    #[error("Service temporarily unavailable: {0}")]
    ServiceUnavailable(String),
    #[error(transparent)]
    Hyper(#[from] hyper::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    SerdeUrlEncoded(#[from] serde_urlencoded::de::Error),
    #[error(transparent)]
    Http(#[from] http::Error),
}

/// A specialized Result type for Oxidite applications
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> hyper::StatusCode {
        match self {
            Error::NotFound(_) => hyper::StatusCode::NOT_FOUND,
            Error::BadRequest(_) => hyper::StatusCode::BAD_REQUEST,
            Error::Unauthorized(_) => hyper::StatusCode::UNAUTHORIZED,
            Error::Forbidden(_) => hyper::StatusCode::FORBIDDEN,
            Error::Conflict(_) => hyper::StatusCode::CONFLICT,
            Error::Validation(_) => hyper::StatusCode::UNPROCESSABLE_ENTITY,
            Error::RateLimited(_) => hyper::StatusCode::TOO_MANY_REQUESTS,
            Error::ServiceUnavailable(_) => hyper::StatusCode::SERVICE_UNAVAILABLE,
            Error::InternalServerError(_) | Error::Server(_) | Error::Hyper(_) | Error::Io(_) | Error::SerdeJson(_) | 
            Error::SerdeUrlEncoded(_) | Error::Http(_) => hyper::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
