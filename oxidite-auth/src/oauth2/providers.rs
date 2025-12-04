use crate::oauth2::client::OAuth2Config;

/// Preconfigured OAuth2 provider
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub name: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: Option<String>,
    pub default_scopes: Vec<String>,
}

impl ProviderConfig {
    /// Google OAuth2 provider
    pub fn google() -> Self {
        Self {
            name: "Google".to_string(),
            authorization_endpoint: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_endpoint: "https://oauth2.googleapis.com/token".to_string(),
            userinfo_endpoint: Some("https://www.googleapis.com/oauth2/v2/userinfo".to_string()),
            default_scopes: vec!["openid".to_string(), "email".to_string(), "profile".to_string()],
        }
    }

    /// GitHub OAuth2 provider
    pub fn github() -> Self {
        Self {
            name: "GitHub".to_string(),
            authorization_endpoint: "https://github.com/login/oauth/authorize".to_string(),
            token_endpoint: "https://github.com/login/oauth/access_token".to_string(),
            userinfo_endpoint: Some("https://api.github.com/user".to_string()),
            default_scopes: vec!["user:email".to_string()],
        }
    }

    /// Microsoft OAuth2 provider
    pub fn microsoft() -> Self {
        Self {
            name: "Microsoft".to_string(),
            authorization_endpoint: "https://login.microsoftonline.com/common/oauth2/v2.0/authorize".to_string(),
            token_endpoint: "https://login.microsoftonline.com/common/oauth2/v2.0/token".to_string(),
            userinfo_endpoint: Some("https://graph.microsoft.com/v1.0/me".to_string()),
           default_scopes: vec!["openid".to_string(), "profile".to_string(), "email".to_string()],
        }
    }

    /// Convert to OAuth2Config
    pub fn to_config(&self, client_id: String, client_secret: String, redirect_uri: String) -> OAuth2Config {
        OAuth2Config {
            client_id,
            client_secret,
            redirect_uri,
            authorization_endpoint: self.authorization_endpoint.clone(),
            token_endpoint: self.token_endpoint.clone(),
            scopes: self.default_scopes.clone(),
        }
    }
}
