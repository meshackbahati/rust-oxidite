use crate::types::OxiditeResponse;
use crate::error::Error;
use http_body_util::{Full, BodyExt};
use bytes::Bytes;
use hyper::Response;
use hyper::header::{HeaderValue, CONTENT_TYPE, SERVER};
use http::StatusCode;

pub(crate) const SERVER_HEADER_VALUE: &str = concat!("Oxidite/", env!("CARGO_PKG_VERSION"));

impl OxiditeResponse {
    /// Create a JSON response from any serializable value.
    ///
    /// Handy with `serde_json::json!()` for inline JSON without defining a struct:
    ///
    /// ```rust,ignore
    /// use oxidite::prelude::*;
    ///
    /// async fn handler() -> Result<Response> {
    ///     Ok(Response::json(json!({"status": "ok", "count": 42})))
    /// }
    /// ```
    pub fn json<T: serde::Serialize>(data: T) -> Self {
        match serde_json::to_vec(&data) {
            Ok(json_bytes) => {
                let res = Response::builder()
                    .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
                    .header(SERVER, HeaderValue::from_static(SERVER_HEADER_VALUE))
                    .body(Full::new(Bytes::from(json_bytes)).map_err(|e| match e {}).boxed())
                    .unwrap();
                Self(res)
            },
            Err(_) => {
                let res = Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
                    .header(SERVER, HeaderValue::from_static(SERVER_HEADER_VALUE))
                    .body(Full::new(Bytes::from("{\"error\":\"Internal Server Error\"}".as_bytes().to_vec())).map_err(|e| match e {}).boxed())
                    .unwrap();
                Self(res)
            },
        }
    }

    /// Create a JSON response from a pre-built `serde_json::Value`.
    ///
    /// This is a convenience that works directly with `serde_json::json!()`
    /// output, letting you write inline JSON without any serde import:
    ///
    /// ```rust,ignore
    /// use oxidite::prelude::*;
    ///
    /// async fn handler() -> Result<Response> {
    ///     Ok(Response::json_val(json!({"message": "Hello!"})))
    /// }
    /// ```
    #[inline]
    pub fn json_val(value: serde_json::Value) -> Self {
        Self::json(value)
    }

    /// Create an HTML response
    pub fn html(body: impl Into<String>) -> Self {
        let res = Response::builder()
            .header(CONTENT_TYPE, HeaderValue::from_static("text/html"))
            .header(SERVER, HeaderValue::from_static(SERVER_HEADER_VALUE))
            .body(Full::new(Bytes::from(body.into())).map_err(|e| match e {}).boxed())
            .unwrap();
        Self(res)
    }

    /// Create a plain text response
    pub fn text(body: impl Into<String>) -> Self {
        let res = Response::builder()
            .header(CONTENT_TYPE, HeaderValue::from_static("text/plain"))
            .header(SERVER, HeaderValue::from_static(SERVER_HEADER_VALUE))
            .body(Full::new(Bytes::from(body.into())).map_err(|e| match e {}).boxed())
            .unwrap();
        Self(res)
    }

    /// Create an empty response with 200 OK status
    pub fn ok() -> Self {
        let res = Response::builder()
            .status(StatusCode::OK)
            .header(SERVER, HeaderValue::from_static(SERVER_HEADER_VALUE))
            .body(Full::new(Bytes::new()).map_err(|e| match e {}).boxed())
            .unwrap();
        Self(res)
    }

    /// Create an empty response with 204 No Content status
    pub fn no_content() -> Self {
        let res = Response::builder()
            .status(StatusCode::NO_CONTENT)
            .header(SERVER, HeaderValue::from_static(SERVER_HEADER_VALUE))
            .body(Full::new(Bytes::new()).map_err(|e| match e {}).boxed())
            .unwrap();
        Self(res)
    }
}

impl From<Error> for OxiditeResponse {
    fn from(error: Error) -> Self {
        let status = error.status_code();
        let body = serde_json::json!({
            "error": error.to_string()
        });

        let fallback = format!(r#"{{"error":"{}"}}"#, status);
        let json_bytes = serde_json::to_vec(&body).unwrap_or_else(|_| fallback.into_bytes());

        let res = Response::builder()
            .status(status)
            .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
            .header(SERVER, HeaderValue::from_static(SERVER_HEADER_VALUE))
            .body(Full::new(Bytes::from(json_bytes)).map_err(|e| match e {}).boxed())
            .unwrap();
        Self(res)
    }
}

/// Helper functions for creating responses.
///
/// Use these for cleaner response creation without needing to call
/// `Response::json(serde_json::json!(...))` directly.
///
/// # Examples
///
/// ```rust,ignore
/// use oxidite_core::response::helpers::{json, text, html};
///
/// // Create a JSON response with inline data
/// async fn handler() -> Result<OxiditeResponse> {
///     Ok(json!({
///         "status": "ok",
///         "count": 42
///     }))
/// }
///
/// // Create a text response
/// async fn text_handler() -> Result<OxiditeResponse> {
///     Ok(text("Hello, world!"))
/// }
///
/// // Create an HTML response
/// async fn html_handler() -> Result<OxiditeResponse> {
///     Ok(html("<h1>Hello</h1>"))
/// }
/// ```
pub mod helpers {
    use crate::types::OxiditeResponse;

    /// Create a JSON response from any serializable value.
    ///
    /// This is a convenience wrapper around `Response::json()` that also
    /// accepts `serde_json::json!()` macro output for inline JSON.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // With inline JSON (most common)
    /// use oxidite_core::response::helpers::json;
    /// async fn handler() -> OxiditeResponse {
    ///     json!({
    ///         "message": "Hello",
    ///         "count": 42
    ///     })
    /// }
    ///
    /// // With a struct
    /// use oxidite_core::response::helpers::json;
    /// #[derive(serde::Serialize)]
    /// struct MyData { message: String }
    /// async fn handler2() -> OxiditeResponse {
    ///     json(MyData { message: "Hello".into() })
    /// }
    /// ```
    #[inline]
    pub fn json<T: serde::Serialize>(data: T) -> OxiditeResponse {
        OxiditeResponse::json(data)
    }

    /// Create a plain text response.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use oxidite_core::response::helpers::text;
    /// async fn handler() -> OxiditeResponse {
    ///     text("Hello, world!")
    /// }
    /// ```
    #[inline]
    pub fn text(body: impl Into<String>) -> OxiditeResponse {
        OxiditeResponse::text(body)
    }

    /// Create an HTML response.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use oxidite_core::response::helpers::html;
    /// async fn handler() -> OxiditeResponse {
    ///     html("<h1>Hello</h1>")
    /// }
    /// ```
    #[inline]
    pub fn html(body: impl Into<String>) -> OxiditeResponse {
        OxiditeResponse::html(body)
    }

    /// Create an empty 200 OK response.
    #[inline]
    pub fn ok() -> OxiditeResponse {
        OxiditeResponse::ok()
    }

    /// Create an empty 204 No Content response.
    #[inline]
    pub fn no_content() -> OxiditeResponse {
        OxiditeResponse::no_content()
    }

    /// Create a JSON response from a pre-built `serde_json::Value`.
    ///
    /// See [`OxiditeResponse::json_val`] for details.
    #[inline]
    pub fn json_val(value: serde_json::Value) -> OxiditeResponse {
        OxiditeResponse::json_val(value)
    }
}
