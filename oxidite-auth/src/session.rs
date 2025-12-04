use async_trait::async_trait;
use cookie::{Cookie, SameSite};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use uuid::Uuid;
use redis::{Client, AsyncCommands};
use crate::{AuthError, Result};

/// Session data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub data: HashMap<String, serde_json::Value>,
}

impl Session {
    pub fn new(user_id: String, ttl_secs: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            created_at: now,
            expires_at: now + ttl_secs,
            data: HashMap::new(),
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now >= self.expires_at
    }

    pub fn renew(&mut self, ttl_secs: u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.expires_at = now + ttl_secs;
    }

    pub fn set_data(&mut self, key: String, value: serde_json::Value) {
        self.data.insert(key, value);
    }

    pub fn get_data(&self, key: &str) -> Option<&serde_json::Value> {
        self.data.get(key)
    }
}

/// Session storage trait
#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn create(&self, session: Session) -> Result<String>;
    async fn get(&self, session_id: &str) -> Result<Option<Session>>;
    async fn update(&self, session: Session) -> Result<()>;
    async fn delete(&self, session_id: &str) -> Result<()>;
    async fn cleanup_expired(&self) -> Result<usize>;
}

/// In-memory session store
pub struct InMemorySessionStore {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
}

impl InMemorySessionStore {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemorySessionStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SessionStore for InMemorySessionStore {
    async fn create(&self, session: Session) -> Result<String> {
        let session_id = session.id.clone();
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);
        Ok(session_id)
    }

    async fn get(&self, session_id: &str) -> Result<Option<Session>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(session_id).cloned())
    }

    async fn update(&self, session: Session) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id.clone(), session);
        Ok(())
    }

    async fn delete(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
        Ok(())
    }

    async fn cleanup_expired(&self) -> Result<usize> {
        let mut sessions = self.sessions.write().await;
        let initial_count = sessions.len();
        sessions.retain(|_, session| !session.is_expired());
        Ok(initial_count - sessions.len())
    }
}

/// Redis session store
pub struct RedisSessionStore {
    client: Client,
    prefix: String,
}

impl RedisSessionStore {
    pub fn new(url: &str, prefix: &str) -> Result<Self> {
        let client = Client::open(url)
            .map_err(|e| AuthError::HashError(e.to_string()))?;
        
        Ok(Self {
            client,
            prefix: prefix.to_string(),
        })
    }

    fn session_key(&self, session_id: &str) -> String {
        format!("{}:{}", self.prefix, session_id)
    }
}

#[async_trait]
impl SessionStore for RedisSessionStore {
    async fn create(&self, session: Session) -> Result<String> {
        let session_id = session.id.clone();
        let key = self.session_key(&session_id);
        let ttl = session.expires_at - session.created_at;
        
        let mut conn = self.client.get_multiplexed_async_connection()
            .await
            .map_err(|e| AuthError::HashError(e.to_string()))?;
        
        let data = serde_json::to_string(&session)
            .map_err(|e| AuthError::HashError(e.to_string()))?;
        
        let _: () = conn.set_ex(&key, data, ttl)
            .await
            .map_err(|e| AuthError::HashError(e.to_string()))?;
        
        Ok(session_id)
    }

    async fn get(&self, session_id: &str) -> Result<Option<Session>> {
        let key = self.session_key(session_id);
        
        let mut conn = self.client.get_multiplexed_async_connection()
            .await
            .map_err(|e| AuthError::HashError(e.to_string()))?;
        
        let result: Option<String> = conn.get(&key)
            .await
            .map_err(|e| AuthError::HashError(e.to_string()))?;
        
        if let Some(data) = result {
            let session: Session = serde_json::from_str(&data)
                .map_err(|e| AuthError::HashError(e.to_string()))?;
            
            if session.is_expired() {
                self.delete(session_id).await?;
                return Ok(None);
            }
            
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    async fn update(&self, session: Session) -> Result<()> {
        self.create(session).await?;
        Ok(())
    }

    async fn delete(&self, session_id: &str) -> Result<()> {
        let key = self.session_key(session_id);
        
        let mut conn = self.client.get_multiplexed_async_connection()
            .await
            .map_err(|e| AuthError::HashError(e.to_string()))?;
        
        let _: () = conn.del(&key)
            .await
            .map_err(|e| AuthError::HashError(e.to_string()))?;
        
        Ok(())
    }

    async fn cleanup_expired(&self) -> Result<usize> {
        // Redis automatically expires keys with TTL, so no cleanup needed
        Ok(0)
    }
}
