use serde::de::DeserializeOwned;
use http::StatusCode;
use http_body_util::BodyExt;

/// Test response wrapper
pub struct TestResponse {
    status: StatusCode,
    body: Vec<u8>,
}

impl TestResponse {
    /// Create a new test response
    pub fn new(status: StatusCode, body: Vec<u8>) -> Self {
        Self { status, body }
    }

    /// Get status code
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Get body as bytes
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Get body as string
    pub fn text(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.clone())
    }

    /// Get body as a lossy UTF-8 string.
    pub fn text_lossy(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }

    /// Deserialize JSON body
    pub fn json<T: DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.body)
    }

    /// Check if response is successful (2xx)
    pub fn is_success(&self) -> bool {
        self.status.is_success()
    }

    /// Assert response status code.
    pub fn assert_status(&self, expected: StatusCode) {
        assert_eq!(
            self.status, expected,
            "expected status {}, got {}",
            expected, self.status
        );
    }

    /// Assert response status is successful (2xx).
    pub fn assert_success(&self) {
        assert!(
            self.is_success(),
            "expected successful status, got {}",
            self.status
        );
    }

    /// Convert from an Oxidite response for test assertions.
    pub async fn from_oxidite_response(response: oxidite_core::OxiditeResponse) -> Self {
        let response = response.into_inner();
        let status = response.status();
        let body = response.into_body();
        let bytes = body
            .collect()
            .await
            .expect("failed to collect response body")
            .to_bytes();
        Self::new(status, bytes.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::TestResponse;
    use http::StatusCode;

    #[test]
    fn text_lossy_handles_binary() {
        let response = TestResponse::new(StatusCode::OK, vec![0xff, 0xfe, b'A']);
        let text = response.text_lossy();
        assert!(text.contains('A'));
    }

    #[tokio::test]
    async fn from_oxidite_response_collects_body() {
        let response = oxidite_core::OxiditeResponse::text("hello");
        let test_response = TestResponse::from_oxidite_response(response).await;
        assert_eq!(test_response.status(), StatusCode::OK);
        assert_eq!(test_response.text().expect("text"), "hello");
    }
}
