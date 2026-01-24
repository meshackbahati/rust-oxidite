use crate::error::{Error, Result};
use crate::types::OxiditeRequest;
use http_body_util::BodyExt;

/// Extension trait for Request to provide helper methods
pub trait RequestExt {
    /// Read the entire body as a String
    fn body_string(&mut self) -> impl std::future::Future<Output = Result<String>> + Send;
    
    /// Read the entire body as Bytes
    fn body_bytes(&mut self) -> impl std::future::Future<Output = Result<bytes::Bytes>> + Send;
}

impl RequestExt for OxiditeRequest {
    async fn body_string(&mut self) -> Result<String> {
        let bytes = self.body_bytes().await?;
        String::from_utf8(bytes.to_vec())
            .map_err(|e| Error::BadRequest(format!("Invalid UTF-8: {}", e)))
    }

    async fn body_bytes(&mut self) -> Result<bytes::Bytes> {
        let body = self.body_mut();
        let collected = body.collect().await
            .map_err(|e| Error::InternalServerError(format!("Failed to read body: {}", e)))?;
        Ok(collected.to_bytes())
    }
}
