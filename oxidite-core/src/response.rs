use crate::types::OxiditeResponse;
use crate::error::Error;
use http_body_util::{Full, BodyExt};
use bytes::Bytes;
use hyper::Response;
use hyper::header::{HeaderValue, CONTENT_TYPE, SERVER};
use http::StatusCode;

const SERVER_HEADER_VALUE: &str = concat!("Oxidite/", env!("CARGO_PKG_VERSION"));

impl OxiditeResponse {
    /// Create a JSON response
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
