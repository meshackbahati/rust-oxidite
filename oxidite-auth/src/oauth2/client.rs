use serde::{Deserialize, Serialize};
use url::Url;
use reqwest::Client;
use base64::{Engine as _, engine::general_purpose};
use crate::{AuthError, Result};

/// OAuth2 client configuration
#[derive(Clone, Debug)]
pub struct OAuth2Config {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: Option<String>,
    pub scopes: Vec<String>,
}

/// OAuth2 client
pub struct OAuth2Client {
    config: OAuth2Config,
    http_client: Client,
}

impl OAuth2Client {
    /// Create a new `OAuth2Client` from the given configuration.
    pub fn new(config: OAuth2Config) -> Self {
        Self {
            config,
            http_client: Client::new(),
        }
    }

    /// Generate authorization URL with PKCE
    pub fn authorization_url(&self, state: &str, code_challenge: Option<&str>) -> Result<String> {
        let mut url = Url::parse(&self.config.authorization_endpoint)
            .map_err(|e| AuthError::HashError(e.to_string()))?;

        url.query_pairs_mut()
            .append_pair("client_id", &self.config.client_id)
            .append_pair("redirect_uri", &self.config.redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("state", state)
            .append_pair("scope", &self.config.scopes.join(" "));

        if let Some(challenge) = code_challenge {
            url.query_pairs_mut()
                .append_pair("code_challenge", challenge)
                .append_pair("code_challenge_method", "S256");
        }

        Ok(url.to_string())
    }

    /// Exchange authorization code for access token with state validation
    pub async fn exchange_code_with_state(&self, code: &str, state: &str, expected_state: &str, code_verifier: Option<&str>) -> Result<TokenResponse> {
        if state != expected_state {
            return Err(AuthError::TokenError("Invalid OAuth2 state".to_string()));
        }
        self.exchange_code(code, code_verifier).await
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code(&self, code: &str, code_verifier: Option<&str>) -> Result<TokenResponse> {
        let mut params = vec![
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &self.config.redirect_uri),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
        ];

        if let Some(verifier) = code_verifier {
            params.push(("code_verifier", verifier));
        }

        let response = self.http_client
            .post(&self.config.token_endpoint)
            .form(&params)
            .send()
            .await
            .map_err(|e| AuthError::TokenError(e.to_string()))?;

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| AuthError::TokenError(e.to_string()))?;

        Ok(token_response)
    }

    /// Get user info from the provider's userinfo endpoint
    pub async fn get_userinfo(&self, access_token: &str) -> Result<serde_json::Value> {
        let endpoint = self.config.userinfo_endpoint.as_ref()
            .ok_or_else(|| AuthError::TokenError("Userinfo endpoint not configured".to_string()))?;

        let response = self.http_client
            .get(endpoint)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| AuthError::TokenError(e.to_string()))?;

        let userinfo = response
            .json::<serde_json::Value>()
            .await
            .map_err(|e| AuthError::TokenError(e.to_string()))?;

        Ok(userinfo)
    }

    /// Refresh access token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse> {
        let params = vec![
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
        ];

        let response = self.http_client
            .post(&self.config.token_endpoint)
            .form(&params)
            .send()
            .await
            .map_err(|e| AuthError::TokenError(e.to_string()))?;

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| AuthError::TokenError(e.to_string()))?;

        Ok(token_response)
    }
}

/// Response from an OAuth2 token endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

/// Generate PKCE code verifier and challenge
pub fn generate_pkce() -> (String, String) {
    use rand::{Rng, distr::{Alphanumeric}};
    
    let verifier: String = rand::rng()
        .sample_iter(Alphanumeric)
        .take(128)
        .map(char::from)
        .collect();

    let challenge = general_purpose::URL_SAFE_NO_PAD.encode(
        ring::digest::digest(&ring::digest::SHA256, verifier.as_bytes()).as_ref()
    );

    (verifier, challenge)
}
