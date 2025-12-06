use rand::Rng;
use oxidite_db::sqlx::Row;

/// Email verification module
pub mod email_verification {
    use rand::Rng;
    
    /// Generate email verification token
    pub fn generate_token() -> String {
        let mut rng = rand::thread_rng();
        let random_bytes: Vec<u8> = (0..32).map(|_| rng.gen::<u8>()).collect();
        hex::encode(random_bytes)
    }
    
    /// Store verification token for user
    pub async fn create_token<D: oxidite_db::Database + ?Sized>(
        db: &D,
        user_id: i64,
    ) -> oxidite_db::Result<String> {
        let token = generate_token();
        
        let query = format!(
            "UPDATE users SET verification_token = '{}' WHERE id = {}",
            token, user_id
        );
        db.execute(&query).await?;
        
        Ok(token)
    }
    
    /// Verify email with token
    pub async fn verify_email<D: oxidite_db::Database + ?Sized>(
        db: &D,
        token: &str,
    ) -> oxidite_db::Result<bool> {
        let query = format!(
            "UPDATE users SET email_verified = 1, verification_token = NULL 
             WHERE verification_token = '{}'",
            token
        );
        let rows = db.execute(&query).await?;
        Ok(rows > 0)
    }
    
    /// Check if user email is verified
    pub async fn is_verified<D: oxidite_db::Database + ?Sized>(
        db: &D,
        user_id: i64,
    ) -> oxidite_db::Result<bool> {
        let query = format!(
            "SELECT email_verified FROM users WHERE id = {}",
            user_id
        );
        let row = db.query_one(&query).await?;
        
        if let Some(row) = row {
            let verified: i64 = row.try_get("email_verified").unwrap_or(0);
            Ok(verified == 1)
        } else {
            Ok(false)
        }
    }
}

/// Password reset module
pub mod password_reset {
    use rand::Rng;
    
    /// Generate password reset token
    pub fn generate_token() -> String {
        let mut rng = rand::thread_rng();
        let random_bytes: Vec<u8> = (0..32).map(|_| rng.gen::<u8>()).collect();
        hex::encode(random_bytes)
    }
    
    /// Create password reset token (valid for 1 hour)
    pub async fn create_token<D: oxidite_db::Database + ?Sized>(
        db: &D,
        user_id: i64,
    ) -> oxidite_db::Result<String> {
        let token = generate_token();
        let now = chrono::Utc::now().timestamp();
        let expires_at = now + 3600; // 1 hour
        
        let query = format!(
            "INSERT INTO password_reset_tokens (user_id, token, expires_at, created_at)
             VALUES ({}, '{}', {}, {})",
            user_id, token, expires_at, now
        );
        db.execute(&query).await?;
        
        Ok(token)
    }
    
    /// Verify reset token and return user_id
    pub async fn verify_token<D: oxidite_db::Database + ?Sized>(
        db: &D,
        token: &str,
    ) -> oxidite_db::Result<Option<i64>> {
        let now = chrono::Utc::now().timestamp();
        
        let query = format!(
            "SELECT user_id FROM password_reset_tokens 
             WHERE token = '{}' AND expires_at > {}",
            token, now
        );
        
        let row = db.query_one(&query).await?;
        
        if let Some(row) = row {
            let user_id: i64 = row.try_get("user_id").unwrap_or(0);
            Ok(Some(user_id))
        } else {
            Ok(None)
        }
    }
    
    /// Consume (delete) reset token
    pub async fn consume_token<D: oxidite_db::Database + ?Sized>(
        db: &D,
        token: &str,
    ) -> oxidite_db::Result<()> {
        let query = format!(
            "DELETE FROM password_reset_tokens WHERE token = '{}'",
            token
        );
        db.execute(&query).await?;
        Ok(())
    }
    
    /// Clean up expired tokens
    pub async fn cleanup_expired<D: oxidite_db::Database + ?Sized>(
        db: &D,
    ) -> oxidite_db::Result<()> {
        let now = chrono::Utc::now().timestamp();
        let query = format!(
            "DELETE FROM password_reset_tokens WHERE expires_at < {}",
            now
        );
        db.execute(&query).await?;
        Ok(())
    }
}

/// Two-Factor Authentication (TOTP) module
pub mod two_factor {
    use totp_rs::{TOTP, Algorithm, Secret};
    use oxidite_db::sqlx::Row;
    
    /// Generate 2FA secret for user
    pub fn generate_secret() -> String {
        use base64::Engine;
        let mut rng = rand::thread_rng();
        let random_bytes: Vec<u8> = (0..20).map(|_| rng.gen::<u8>()).collect();
        base64::engine::general_purpose::STANDARD.encode(random_bytes)
    }
    
    /// Enable 2FA for user
    pub async fn enable<D: oxidite_db::Database + ?Sized>(
        db: &D,
        user_id: i64,
        secret: &str,
    ) -> oxidite_db::Result<()> {
        let query = format!(
            "UPDATE users SET two_factor_secret = '{}', two_factor_enabled = 1 
             WHERE id = {}",
            secret, user_id
        );
        db.execute(&query).await?;
        Ok(())
    }
    
    /// Disable 2FA for user
    pub async fn disable<D: oxidite_db::Database + ?Sized>(
        db: &D,
        user_id: i64,
    ) -> oxidite_db::Result<()> {
        let query = format!(
            "UPDATE users SET two_factor_secret = NULL, two_factor_enabled = 0 
             WHERE id = {}",
            user_id
        );
        db.execute(&query).await?;
        Ok(())
    }
    
    /// Verify TOTP code
    pub fn verify_code(secret: &str, code: &str) -> bool {
        use base64::Engine;
        let secret_bytes = match base64::engine::general_purpose::STANDARD.decode(secret) {
            Ok(bytes) => bytes,
            Err(_) => return false,
        };
        
        let totp = match TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret_bytes,
        ) {
            Ok(t) => t,
            Err(_) => return false,
        };
        
        totp.check_current(code).unwrap_or(false)
    }
    
    /// Get user's 2FA secret
    pub async fn get_secret<D: oxidite_db::Database + ?Sized>(
        db: &D,
        user_id: i64,
    ) -> oxidite_db::Result<Option<String>> {
        let query = format!(
            "SELECT two_factor_secret, two_factor_enabled FROM users WHERE id = {}",
            user_id
        );
        
        let row = db.query_one(&query).await?;
        
        if let Some(row) = row {
            let enabled: i64 = row.try_get("two_factor_enabled").unwrap_or(0);
            if enabled == 1 {
                let secret: String = row.try_get("two_factor_secret").unwrap_or_default();
                if !secret.is_empty() {
                    return Ok(Some(secret));
                }
            }
        }
        
        Ok(None)
    }
    
    /// Generate provisioning URI for TOTP setup (for QR code)
    pub fn generate_provisioning_uri(secret: &str, account: &str, issuer: &str) -> String {
        // Format: otpauth://totp/issuer:account?secret=SECRET&issuer=ISSUER
        format!(
            "otpauth://totp/{}:{}?secret={}&issuer={}",
            urlencoding::encode(issuer),
            urlencoding::encode(account),
            secret,
            urlencoding::encode(issuer)
        )
    }
}
