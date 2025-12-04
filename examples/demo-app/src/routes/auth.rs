use oxidite_core::{Request, Response, Error, State, FromRequest, RequestExt, json};
use serde::Deserialize;
use crate::AppState;
use std::sync::Arc;
use std::collections::BTreeMap;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Register a new user
pub async fn register(mut req: Request) -> Result<Response, Error> {
    let _body = req.body_string().await?;
    
    // In a real app: hash password, save to database
    
    Ok(json(serde_json::json!({
        "success": true,
        "message": "User registered successfully"
    })))
}

// ... (structs remain same)

/// Login and receive JWT token
pub async fn login(mut req: Request) -> Result<Response, Error> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    let body = req.body_string().await?;
    let login_req: LoginRequest = serde_json::from_str(&body)
        .map_err(|e| Error::BadRequest(format!("Invalid JSON: {}", e)))?;
    
    // In a real app: verify credentials against DB
    if login_req.password != "secret" {
        return Err(Error::Unauthorized("Invalid credentials".to_string()));
    }
    
    // Generate real JWT
    let mut claims = BTreeMap::new();
    claims.insert("sub".to_string(), login_req.email.clone());
    claims.insert("name".to_string(), "John Doe".to_string());
    
    let token = state.jwt.generate_token(&claims)
        .map_err(|e| Error::Server(format!("Failed to generate token: {}", e)))?;
    
    Ok(json(serde_json::json!({
        "success": true,
        "token": token,
        "user": {
            "email": login_req.email,
            "name": "John Doe"
        }
    })))
}

/// OAuth2 Google authentication
/// 
/// # Example
/// 
/// ```bash
/// # Redirect user to Google auth
/// curl http://localhost:8080/auth/oauth/google
/// ```
pub async fn oauth_google(_req: Request) -> Result<Response, Error> {
    // In a real app: use OAuth2Client to generate auth URL
    let auth_url = "https://accounts.google.com/o/oauth2/v2/auth?client_id=...";
    
    Ok(json(serde_json::json!({
        "auth_url": auth_url,
        "message": "Redirect user to this URL"
    })))
}
