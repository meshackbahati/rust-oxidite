use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Internal server error: {0}")]
    InternalServerError(String),
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
    #[error("Method not allowed: {0}")]
    MethodNotAllowed(String),
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
    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
}

/// A specialized Result type for Oxidite applications
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> hyper::StatusCode {
        match self {
            Error::NotFound(_) => hyper::StatusCode::NOT_FOUND,
            Error::BadRequest(_) | Error::SerdeJson(_) | Error::SerdeUrlEncoded(_) | Error::Utf8(_) => hyper::StatusCode::BAD_REQUEST,
            Error::Unauthorized(_) => hyper::StatusCode::UNAUTHORIZED,
            Error::Forbidden(_) => hyper::StatusCode::FORBIDDEN,
            Error::Conflict(_) => hyper::StatusCode::CONFLICT,
            Error::Validation(_) => hyper::StatusCode::UNPROCESSABLE_ENTITY,
            Error::RateLimited(_) => hyper::StatusCode::TOO_MANY_REQUESTS,
            Error::ServiceUnavailable(_) => hyper::StatusCode::SERVICE_UNAVAILABLE,
            Error::MethodNotAllowed(_) => hyper::StatusCode::METHOD_NOT_ALLOWED,
            Error::InternalServerError(_) | Error::Hyper(_) | Error::Io(_) | Error::Http(_) => hyper::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Check if this is a client error (4xx status code)
    /// These errors are expected and should be logged at debug/trace level
    pub fn is_client_error(&self) -> bool {
        let status = self.status_code();
        status.is_client_error()
    }

    /// Check if this is a server error (5xx status code)
    /// These errors are unexpected and should be logged at error level
    pub fn is_server_error(&self) -> bool {
        let status = self.status_code();
        status.is_server_error()
    }
}
