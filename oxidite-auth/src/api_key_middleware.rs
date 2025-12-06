use oxidite_core::{OxiditeRequest, OxiditeResponse, Result as OxiditeResult, Error};
use oxidite_db::Database;
use std::sync::Arc;
use crate::api_key::ApiKey;

/// Middleware to authenticate requests using API keys
pub struct ApiKeyMiddleware {
    db: Arc<dyn Database>,
}

impl ApiKeyMiddleware {
    pub fn new(db: Arc<dyn Database>) -> Self {
        Self { db }
    }
    
    /// Extract and verify API key from request
    pub async fn authenticate(&self, req: &mut OxiditeRequest) -> OxiditeResult<i64> {
        // Extract API key from Authorization header or query parameter
        let key = self.extract_key(req)?;
        
        // Verify the key
        let api_key = ApiKey::verify_key(&*self.db, &key).await
            .map_err(|_| Error::Server("Database error".to_string()))?
            .ok_or_else(|| Error::Unauthorized("Invalid or expired API key".to_string()))?;
        
        // Store user_id in request extensions
        req.extensions_mut().insert(api_key.user_id);
        
        Ok(api_key.user_id)
    }
    
    /// Extract API key from request headers or query string
    fn extract_key(&self, req: &OxiditeRequest) -> OxiditeResult<String> {
        // Try Authorization header first (Bearer token style)
        if let Some(auth_header) = req.headers().get("authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                if let Some(key) = auth_str.strip_prefix("Bearer ") {
                    return Ok(key.to_string());
                }
            }
        }
        
        // Try X-API-Key header
        if let Some(api_key_header) = req.headers().get("x-api-key") {
            if let Ok(key) = api_key_header.to_str() {
                return Ok(key.to_string());
            }
        }
        
        // Try query parameter
        if let Some(query) = req.uri().query() {
            for param in query.split('&') {
                if let Some((k, v)) = param.split_once('=') {
                    if k == "api_key" {
                        return Ok(v.to_string());
                    }
                }
            }
        }
        
        Err(Error::Unauthorized("API key required".to_string()))
    }
}
