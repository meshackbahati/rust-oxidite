use oxidite_db::sqlx::{self, FromRow};
use sha2::{Sha256, Digest};
use rand::Rng;
use base64::Engine;

#[derive(FromRow, Clone, Debug)]
pub struct ApiKey {
    pub id: i64,
    pub user_id: i64,
    pub key_hash: String,
    pub name: String,
    pub last_used_at: Option<i64>,
    pub expires_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl ApiKey {
    /// Generate a new API key with prefix
    pub fn generate_key() -> String {
        let mut rng = rand::thread_rng();
        let random_bytes: Vec<u8> = (0..32).map(|_| rng.random()).collect();
        let key = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(&random_bytes);
        format!("ox_{}", key)
    }
    
    /// Hash an API key for storage
    pub fn hash_key(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    /// Create a new API key for a user
    pub async fn create_for_user<D: oxidite_db::Database>(
        db: &D,
        user_id: i64,
        name: &str,
        expires_at: Option<i64>,
    ) -> oxidite_db::Result<(ApiKey, String)> {
        let key = Self::generate_key();
        let key_hash = Self::hash_key(&key);
        let now = chrono::Utc::now().timestamp();
        
        let query = format!(
            "INSERT INTO api_keys (user_id, key_hash, name, expires_at, created_at, updated_at) 
             VALUES ({}, '{}', '{}', {}, {}, {})",
            user_id, key_hash, name,
            expires_at.map(|e| e.to_string()).unwrap_or("NULL".to_string()),
            now, now
        );
        
        db.execute(&query).await?;
        
        // Retrieve the created key
        let get_query = format!(
            "SELECT * FROM api_keys WHERE key_hash = '{}'",
            key_hash
        );
        let row = db.query_one(&get_query).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;
        
        let api_key = ApiKey::from_row(&row)?;
        Ok((api_key, key))
    }
    
    /// Find API key by key string and verify it's valid
    pub async fn verify_key<D: oxidite_db::Database + ?Sized>(
        db: &D,
        key: &str,
    ) -> oxidite_db::Result<Option<ApiKey>> {
        let key_hash = Self::hash_key(key);
        let now = chrono::Utc::now().timestamp();
        
        let query = format!(
            "SELECT * FROM api_keys 
             WHERE key_hash = '{}' 
             AND (expires_at IS NULL OR expires_at > {})",
            key_hash, now
        );
        
        let row = db.query_one(&query).await?;
        
        match row {
            Some(row) => {
                let mut api_key = ApiKey::from_row(&row)?;
                
                // Update last_used_at
                let update_query = format!(
                    "UPDATE api_keys SET last_used_at = {} WHERE id = {}",
                    now, api_key.id
                );
                let _ = db.execute(&update_query).await;
                api_key.last_used_at = Some(now);
                
                Ok(Some(api_key))
            }
            None => Ok(None),
        }
    }
    
    /// Revoke (delete) an API key
    pub async fn revoke<D: oxidite_db::Database>(
        db: &D,
        key_id: i64,
        user_id: i64,
    ) -> oxidite_db::Result<bool> {
        let query = format!(
            "DELETE FROM api_keys WHERE id = {} AND user_id = {}",
            key_id, user_id
        );
        let rows = db.execute(&query).await?;
        Ok(rows > 0)
    }
    
    /// Get all API keys for a user
    pub async fn get_user_keys<D: oxidite_db::Database>(
        db: &D,
        user_id: i64,
    ) -> oxidite_db::Result<Vec<ApiKey>> {
        let query = format!(
            "SELECT * FROM api_keys WHERE user_id = {} ORDER BY created_at DESC",
            user_id
        );
        
        let rows = db.query(&query).await?;
        let mut keys = Vec::new();
        
        for row in rows {
            keys.push(ApiKey::from_row(&row)?);
        }
        
        Ok(keys)
    }
}
