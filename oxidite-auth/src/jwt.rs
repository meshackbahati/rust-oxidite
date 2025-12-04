use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::{AuthError, Result};

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,  // Subject (user ID)
    pub exp: usize,   // Expiration time
    pub iat: usize,   // Issued at
    pub nbf: usize,   // Not before
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Vec<String>>,
}

impl Claims {
    pub fn new(user_id: String, expiry_secs: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        Self {
            sub: user_id,
            exp: now + expiry_secs as usize,
            iat: now,
            nbf: now,
            roles: None,
            permissions: None,
        }
    }

    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles = Some(roles);
        self
    }

    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = Some(permissions);
        self
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles
            .as_ref()
            .map(|roles| roles.iter().any(|r| r == role))
            .unwrap_or(false)
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions
            .as_ref()
            .map(|perms| perms.iter().any(|p| p == permission))
            .unwrap_or(false)
    }
}

/// JWT Token struct
pub struct JwtToken {
    secret: String,
}

impl JwtToken {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    pub fn create(&self, claims: Claims) -> Result<String> {
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )?;
        Ok(token)
    }

    pub fn verify(&self, token: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )?;
        Ok(token_data.claims)
    }
}

/// Create a JWT token
pub fn create_token(user_id: String, secret: &str, expiry_secs: u64) -> Result<String> {
    let claims = Claims::new(user_id, expiry_secs);
    let jwt = JwtToken::new(secret.to_string());
    jwt.create(claims)
}

/// Verify a JWT token
pub fn verify_token(token: &str, secret: &str) -> Result<Claims> {
    let jwt = JwtToken::new(secret.to_string());
    jwt.verify(token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_verify_token() {
        let secret = "test_secret_key";
        let user_id = "user123";
        
        let token = create_token(user_id.to_string(), secret, 3600).unwrap();
        let claims = verify_token(&token, secret).unwrap();
        
        assert_eq!(claims.sub, user_id);
    }

    #[test]
    fn test_claims_with_roles() {
        let claims = Claims::new("user123".to_string(), 3600)
            .with_roles(vec!["admin".to_string(), "user".to_string()]);
        
        assert!(claims.has_role("admin"));
        assert!(claims.has_role("user"));
        assert!(!claims.has_role("guest"));
    }
}
