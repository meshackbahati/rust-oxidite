use http::{Method, HeaderMap, HeaderName, HeaderValue};
use http_body_util::Full;
use bytes::Bytes;
use serde::Serialize;
use oxidite_core::types::OxiditeRequest;
use http_body_util::BodyExt;

/// Error type for test request construction.
#[derive(Debug, thiserror::Error)]
pub enum TestRequestError {
    #[error("invalid header name: {0}")]
    InvalidHeaderName(#[from] http::header::InvalidHeaderName),
    #[error("invalid header value: {0}")]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),
    #[error("failed to serialize JSON body: {0}")]
    JsonSerialize(#[from] serde_json::Error),
    #[error("failed to build request: {0}")]
    Build(#[from] http::Error),
}

/// Test request builder
pub struct TestRequest {
    method: Method,
    uri: String,
    headers: HeaderMap,
    body: Vec<u8>,
}

impl TestRequest {
    /// Create a new test request builder
    pub fn new(method: Method, uri: impl Into<String>) -> Self {
        Self {
            method,
            uri: uri.into(),
            headers: HeaderMap::new(),
            body: Vec::new(),
        }
    }

    /// Create a GET request
    pub fn get(uri: impl Into<String>) -> Self {
        Self::new(Method::GET, uri)
    }

    /// Create a POST request
    pub fn post(uri: impl Into<String>) -> Self {
        Self::new(Method::POST, uri)
    }

    /// Create a PUT request
    pub fn put(uri: impl Into<String>) -> Self {
        Self::new(Method::PUT, uri)
    }

    /// Create a DELETE request
    pub fn delete(uri: impl Into<String>) -> Self {
        Self::new(Method::DELETE, uri)
    }

    /// Add a header
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        let name = HeaderName::from_bytes(name.into().as_bytes())
            .expect("invalid header name in TestRequest::header");
        let value = HeaderValue::from_str(&value.into())
            .expect("invalid header value in TestRequest::header");
        self.headers.insert(name, value);
        self
    }

    /// Add a header without panicking on invalid header input.
    pub fn try_header(
        mut self,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, TestRequestError> {
        let name = HeaderName::from_bytes(name.into().as_bytes())?;
        let value = HeaderValue::from_str(&value.into())?;
        self.headers.insert(name, value);
        Ok(self)
    }

    /// Set JSON body
    pub fn json<T: Serialize>(mut self, body: &T) -> Self {
        self.body = serde_json::to_vec(body).expect("failed to serialize JSON body in TestRequest::json");
        self = self.header("content-type", "application/json");
        self
    }

    /// Set JSON body without panicking.
    pub fn try_json<T: Serialize>(mut self, body: &T) -> Result<Self, TestRequestError> {
        self.body = serde_json::to_vec(body)?;
        self.try_header("content-type", "application/json")
    }

    /// Set raw body
    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }

    /// Build the request
    pub fn build(self) -> http::Request<Full<Bytes>> {
        let mut builder = http::Request::builder()
            .method(self.method)
            .uri(self.uri);

        for (name, value) in self.headers.iter() {
            builder = builder.header(name, value);
        }

        builder
            .body(Full::new(Bytes::from(self.body)))
            .expect("failed to build http::Request in TestRequest::build")
    }

    /// Build the request without panicking.
    pub fn try_build(self) -> Result<http::Request<Full<Bytes>>, TestRequestError> {
        let mut builder = http::Request::builder()
            .method(self.method)
            .uri(self.uri);

        for (name, value) in &self.headers {
            builder = builder.header(name, value);
        }

        Ok(builder.body(Full::new(Bytes::from(self.body)))?)
    }

    /// Build an Oxidite request body for direct handler/router invocation.
    pub fn build_oxidite(self) -> OxiditeRequest {
        let request = self.build();
        let (parts, body) = request.into_parts();
        let body = body.map_err(|e| match e {}).boxed();
        http::Request::from_parts(parts, body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Serialize, Deserialize)]
    struct TestData {
        name: String,
    }

    #[test]
    fn test_request_builder() {
        let data = TestData {
            name: "test".to_string(),
        };

        let request = TestRequest::post("/api/test")
            .json(&data)
            .header("x-custom", "value")
            .build();

        assert_eq!(request.method(), Method::POST);
        assert_eq!(request.uri(), "/api/test");
        assert!(request.headers().contains_key("content-type"));
    }

    #[test]
    fn test_try_header_invalid_name() {
        let result = TestRequest::get("/").try_header("bad header", "value");
        assert!(result.is_err());
    }

    #[test]
    fn test_build_oxidite() {
        let request = TestRequest::get("/health").build_oxidite();
        assert_eq!(request.uri(), "/health");
    }
}
