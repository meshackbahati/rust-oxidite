use serde::{Deserialize, Serialize};

/// OAuth2 grant types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GrantType {
    AuthorizationCode,
    ClientCredentials,
    RefreshToken,
}

/// Authorization code grant
#[derive(Debug, Clone)]
pub struct AuthorizationCodeGrant {
    pub code: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub code_challenge: Option<String>,
    pub expires_at: u64,
}

/// Client credentials grant
#[derive(Debug, Clone)]
pub struct ClientCredentialsGrant {
    pub client_id: String,
    pub client_secret: String,
    pub scope: Option<String>,
}

impl AuthorizationCodeGrant {
    pub fn new(client_id: String, redirect_uri: String, ttl_secs: u64) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        use uuid::Uuid;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            code: Uuid::new_v4().to_string(),
            client_id,
            redirect_uri,
            code_challenge: None,
            expires_at: now + ttl_secs,
        }
    }

    pub fn with_pkce(mut self, code_challenge: String) -> Self {
        self.code_challenge = Some(code_challenge);
        self
    }

    pub fn is_expired(&self) -> bool {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        now >= self.expires_at
    }
}
