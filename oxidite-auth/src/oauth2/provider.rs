use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::{AuthError, Result};
use crate::oauth2::grants::AuthorizationCodeGrant;

/// Authorization request
#[derive(Debug, Clone, Deserialize)]
pub struct AuthorizationRequest {
    pub client_id: String,
    pub redirect_uri: String,
    pub response_type: String,
    pub scope: Option<String>,
    pub state: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
}

/// Token request
#[derive(Debug, Clone, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub code: Option<String>,
    pub redirect_uri: Option<String>,
    pub client_id: String,
    pub client_secret: String,
    pub code_verifier: Option<String>,
    pub refresh_token: Option<String>,
}

/// Token response
#[derive(Debug, Clone, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

/// OAuth2 provider
pub struct OAuth2Provider {
    codes: Arc<RwLock<HashMap<String, AuthorizationCodeGrant>>>,
    clients: Arc<RwLock<HashMap<String, ClientConfig>>>,
}

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uris: Vec<String>,
}

impl OAuth2Provider {
    pub fn new() -> Self {
        Self {
            codes: Arc::new(RwLock::new(HashMap::new())),
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a client
    pub async fn register_client(&self, config: ClientConfig) -> Result<()> {
        let mut clients = self.clients.write().await;
        clients.insert(config.client_id.clone(), config);
        Ok(())
    }

    /// Handle authorization request
    pub async fn authorize(&self, req: AuthorizationRequest, user_id: String) -> Result<String> {
        // Validate client
        let clients = self.clients.read().await;
        let client = clients.get(&req.client_id)
            .ok_or(AuthError::InvalidCredentials)?;

        // Validate redirect URI
        if !client.redirect_uris.contains(&req.redirect_uri) {
            return Err(AuthError::InvalidCredentials);
        }

        // Generate authorization code
        let mut grant = AuthorizationCodeGrant::new(
            req.client_id.clone(),
            req.redirect_uri.clone(),
            600, // 10 minutes
        );

        if let Some(challenge) = req.code_challenge {
            grant = grant.with_pkce(challenge);
        }

        let code = grant.code.clone();
        let mut codes = self.codes.write().await;
        codes.insert(code.clone(), grant);

        Ok(code)
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code(&self, req: TokenRequest) -> Result<TokenResponse> {
        let code = req.code.ok_or(AuthError::InvalidToken)?;

        // Get and remove authorization code
        let mut codes = self.codes.write().await;
        let grant = codes.remove(&code).ok_or(AuthError::InvalidToken)?;

        // Validate client
        let clients = self.clients.read().await;
        let client = clients.get(&req.client_id)
            .ok_or(AuthError::InvalidCredentials)?;

        if client.client_secret != req.client_secret {
            return Err(AuthError::InvalidCredentials);
        }

        // Validate redirect URI
        if let Some(redirect_uri) = req.redirect_uri {
            if grant.redirect_uri != redirect_uri {
                return Err(AuthError::InvalidCredentials);
            }
        }

        // Check expiration
        if grant.is_expired() {
            return Err(AuthError::TokenExpired);
        }

        // Validate PKCE if used
        if let Some(challenge) = grant.code_challenge {
            let verifier = req.code_verifier.ok_or(AuthError::InvalidToken)?;
            // TODO: Verify PKCE challenge
        }

        // Generate access token
        let access_token = Uuid::new_v4().to_string();
        let refresh_token = Uuid::new_v4().to_string();

        Ok(TokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            refresh_token: Some(refresh_token),
            scope: None,
        })
    }
}

impl Default for OAuth2Provider {
    fn default() -> Self {
        Self::new()
    }
}
