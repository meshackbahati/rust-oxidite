use crate::types::OxiditeResponse;
use http_body_util::Full;
use bytes::Bytes;
use hyper::Response;
use hyper::header::{HeaderValue, CONTENT_TYPE, SERVER};
use http::StatusCode;

/// Create a JSON response
pub fn json<T: serde::Serialize>(data: T) -> OxiditeResponse {
    match serde_json::to_vec(&data) {
        Ok(json_bytes) => {
            let mut response = Response::builder()
                .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
                .header(SERVER, HeaderValue::from_static("Oxidite/0.1.0"))
                .body(Full::new(Bytes::from(json_bytes)))
                .unwrap();
            response
        },
        Err(e) => {
            let mut response = Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header(SERVER, HeaderValue::from_static("Oxidite/0.1.0"))
                .body(Full::new(Bytes::from(format!("Internal Server Error: {}", e))))
                .unwrap();
            response
        },
    }
}

/// Create an HTML response
pub fn html(body: impl Into<String>) -> OxiditeResponse {
    let mut response = Response::builder()
        .header(CONTENT_TYPE, HeaderValue::from_static("text/html"))
        .header(SERVER, HeaderValue::from_static("Oxidite/0.1.0"))
        .body(Full::new(Bytes::from(body.into())))
        .unwrap();
    response
}

/// Create a plain text response
pub fn text(body: impl Into<String>) -> OxiditeResponse {
    Response::builder()
        .header(CONTENT_TYPE, HeaderValue::from_static("text/plain"))
        .header(SERVER, HeaderValue::from_static("Oxidite/0.1.0"))
        .body(Full::new(Bytes::from(body.into())))
        .unwrap()
}
