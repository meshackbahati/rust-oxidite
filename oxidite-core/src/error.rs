use thiserror::Error;

/// Error type for Oxidite applications.
///
/// Covers common HTTP error conditions as well as low-level
/// errors from hyper, serde, I/O, and other dependencies.
/// Use [`Error::internal`] to wrap arbitrary errors while preserving
/// the original source chain.
#[derive(Error, Debug)]
pub enum Error {
    /// A generic internal server error with a message string.
    ///
    /// Prefer [`Error::internal`] when you have access to the original
    /// source error so its type information and backtrace are preserved.
    #[error("Internal server error: {0}")]
    InternalServerError(String),
    /// Internal server error that preserves the original source error for debugging.
    ///
    /// Prefer this variant when you want to chain the original error without losing
    /// its type information, source chain, or backtrace.
    #[error("Internal server error: {message}")]
    InternalServerErrorWithSource {
        message: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// The requested resource could not be found.
    ///
    /// Maps to HTTP 404.
    #[error("Resource not found: {0}")]
    NotFound(String),
    /// The request was malformed or contains invalid parameters.
    ///
    /// Maps to HTTP 400.
    #[error("Bad request: {0}")]
    BadRequest(String),
    /// Authentication is required and has failed or not been provided.
    ///
    /// Maps to HTTP 401.
    #[error("Unauthorized access: {0}")]
    Unauthorized(String),
    /// The server understood the request but refuses to authorize it.
    ///
    /// Maps to HTTP 403.
    #[error("Forbidden: {0}")]
    Forbidden(String),
    /// The request conflicts with the current state of the resource.
    ///
    /// Maps to HTTP 409.
    #[error("Resource conflict: {0}")]
    Conflict(String),
    /// Input validation failed.
    ///
    /// Maps to HTTP 422.
    #[error("Validation failed: {0}")]
    Validation(String),
    /// Too many requests have been made in a given time window.
    ///
    /// Maps to HTTP 429.
    #[error("Rate limit exceeded: {0}")]
    RateLimited(String),
    /// The server is temporarily unable to handle the request.
    ///
    /// Maps to HTTP 503.
    #[error("Service temporarily unavailable: {0}")]
    ServiceUnavailable(String),
    /// The HTTP method is not allowed for the requested resource.
    ///
    /// Maps to HTTP 405.
    #[error("Method not allowed: {0}")]
    MethodNotAllowed(String),
    /// An error from the underlying hyper HTTP library.
    #[error(transparent)]
    Hyper(#[from] hyper::Error),
    /// An I/O error (e.g. file not found, connection reset).
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// JSON serialization or deserialization error.
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    /// URL-encoded form data deserialization error.
    #[error(transparent)]
    SerdeUrlEncoded(#[from] serde_urlencoded::de::Error),
    /// An error from the `http` crate (e.g. invalid header or status code).
    #[error(transparent)]
    Http(#[from] http::Error),
    /// A string could not be decoded as valid UTF-8.
    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
}

/// A specialized Result type for Oxidite applications
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Create a new `InternalServerError` while preserving the original source error.
    ///
    /// This is the recommended way to wrap external errors in handler code — the
    /// original error's type, source chain, and backtrace are retained for debugging
    /// while the `Display` message is still user-facing.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use oxidite::prelude::*;
    ///
    /// async fn handler() -> Result<Response> {
    ///     let data = std::fs::read_to_string("config.toml")
    ///         .map_err(Error::internal)?;
    ///     Ok(Response::text(data))
    /// }
    /// ```
    pub fn internal<E: std::error::Error + Send + Sync + 'static>(err: E) -> Self {
        Error::InternalServerErrorWithSource {
            message: err.to_string(),
            source: Box::new(err),
        }
    }

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
            Error::InternalServerError(_)
            | Error::InternalServerErrorWithSource { .. }
            | Error::Hyper(_)
            | Error::Io(_)
            | Error::Http(_) => hyper::StatusCode::INTERNAL_SERVER_ERROR,
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

/// Render an Ignition-style HTML error page
pub fn render_ignition_error(error: &Error) -> String {
    let error_type = match error {
        Error::InternalServerError(_) | Error::InternalServerErrorWithSource { .. } => "Internal Server Error",
        Error::NotFound(_) => "Not Found",
        Error::BadRequest(_) => "Bad Request",
        Error::Unauthorized(_) => "Unauthorized",
        Error::Forbidden(_) => "Forbidden",
        Error::Conflict(_) => "Conflict",
        Error::Validation(_) => "Validation Error",
        Error::RateLimited(_) => "Rate Limited",
        Error::ServiceUnavailable(_) => "Service Unavailable",
        Error::MethodNotAllowed(_) => "Method Not Allowed",
        Error::Hyper(_) => "Hyper Protocol Error",
        Error::Io(_) => "I/O Error",
        Error::SerdeJson(_) => "JSON Serialization Error",
        Error::SerdeUrlEncoded(_) => "URL Encoded Error",
        Error::Http(_) => "HTTP Error",
        Error::Utf8(_) => "UTF-8 Encoding Error",
    };

    let error_message = error.to_string();

    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Oxidite Exception: {}</title>
    <style>
        :root {{
            --bg-color: #0f1115;
            --surface: #1e2128;
            --primary: #f05033;
            --text: #e2e8f0;
            --text-muted: #94a3b8;
            --border: #334155;
            --danger: #ef4444;
        }}
        body {{
            font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
            background-color: var(--bg-color);
            color: var(--text);
            margin: 0;
            padding: 0;
            display: flex;
            justify-content: center;
        }}
        .container {{
            max-width: 1200px;
            width: 100%;
            margin: 40px;
            background-color: var(--surface);
            border-radius: 12px;
            box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
            overflow: hidden;
            border: 1px solid var(--border);
        }}
        .header {{
            background: linear-gradient(135deg, #2a0808 0%, var(--surface) 100%);
            padding: 40px;
            border-bottom: 1px solid var(--border);
        }}
        .logo {{
            font-size: 24px;
            font-weight: 800;
            color: var(--primary);
            margin-bottom: 20px;
            display: flex;
            align-items: center;
            gap: 10px;
        }}
        .logo svg {{
            width: 32px;
            height: 32px;
        }}
        h1 {{
            margin: 0;
            font-size: 32px;
            font-weight: 700;
            color: white;
            line-height: 1.2;
        }}
        .exception-type {{
            display: inline-block;
            background-color: rgba(239, 68, 68, 0.1);
            color: var(--danger);
            padding: 4px 12px;
            border-radius: 9999px;
            font-size: 14px;
            font-weight: 600;
            margin-bottom: 16px;
        }}
        .content {{
            padding: 40px;
        }}
        .stack-trace {{
            background-color: #0d1117;
            border-radius: 8px;
            padding: 20px;
            font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
            font-size: 14px;
            line-height: 1.6;
            overflow-x: auto;
            border: 1px solid var(--border);
        }}
        .footer {{
            padding: 20px 40px;
            background-color: #15181e;
            border-top: 1px solid var(--border);
            font-size: 14px;
            color: var(--text-muted);
            display: flex;
            justify-content: space-between;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="logo">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"/>
                </svg>
                Oxidite
            </div>
            <div class="exception-type">{}</div>
            <h1>{}</h1>
        </div>
        <div class="content">
            <h3>Stack Trace</h3>
            <div class="stack-trace">
                <div>Environment: development</div>
                <div style="color: var(--text-muted); margin-top: 10px;">Backtrace captured at crash site:</div>
                <div style="color: var(--primary); margin-top: 10px;">{}</div>
            </div>
        </div>
        <div class="footer">
            <span>Oxidite Framework v2.3.3-1</span>
            <span>Running in Development Mode</span>
        </div>
    </div>
</body>
</html>"#, error_type, error_type, error_message, error_message)
}
