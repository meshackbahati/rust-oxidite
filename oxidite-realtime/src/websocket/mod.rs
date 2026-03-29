use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use uuid::Uuid;

pub mod rooms;

pub use rooms::{Room, RoomManager};

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    /// Text message
    Text { content: String },
    /// JSON message
    Json { data: serde_json::Value },
    /// Binary message
    Binary { data: Vec<u8> },
    /// Ping
    Ping,
    /// Pong  
    Pong,
    /// Close connection
    Close,
}

impl Message {
    pub fn text(content: impl Into<String>) -> Self {
        Self::Text { content: content.into() }
    }

    pub fn json(data: serde_json::Value) -> Self {
        Self::Json { data }
    }

    pub fn to_ws_message(&self) -> Result<WsMessage> {
        match self {
            Message::Text { content } => Ok(WsMessage::Text(content.clone())),
            Message::Json { data } => {
                let json_str = serde_json::to_string(data)?;
                Ok(WsMessage::Text(json_str))
            }
            Message::Binary { data } => Ok(WsMessage::Binary(data.clone())),
            Message::Ping => Ok(WsMessage::Ping(vec![])),
            Message::Pong => Ok(WsMessage::Pong(vec![])),
            Message::Close => Ok(WsMessage::Close(None)),
        }
    }

    pub fn from_ws_message(msg: WsMessage) -> Result<Self> {
        match msg {
            WsMessage::Text(text) => {
                // Try to parse as JSON first
                if let Ok(data) = serde_json::from_str(&text) {
                    Ok(Message::Json { data })
                } else {
                    Ok(Message::Text { content: text })
                }
            }
            WsMessage::Binary(data) => Ok(Message::Binary { data }),
            WsMessage::Ping(_) => Ok(Message::Ping),
            WsMessage::Pong(_) => Ok(Message::Pong),
            WsMessage::Close(_) => Ok(Message::Close),
            _ => Err(WebSocketError::InvalidMessage),
        }
    }
}

/// WebSocket connection
pub struct WebSocketConnection {
    pub id: String,
    pub user_id: Option<String>,
    tx: broadcast::Sender<Message>,
}

impl WebSocketConnection {
    pub fn new(user_id: Option<String>) -> (Self, broadcast::Receiver<Message>) {
        let (tx, rx) = broadcast::channel(100);
        let id = Uuid::new_v4().to_string();
        
        (
            Self {
                id,
                user_id,
                tx,
            },
            rx,
        )
    }

    pub fn send(&self, message: Message) -> Result<()> {
        self.tx.send(message)
            .map_err(|_| WebSocketError::SendError)?;
        Ok(())
    }
}

/// WebSocket manager for handling multiple connections
pub struct WebSocketManager {
    connections: Arc<RwLock<HashMap<String, Arc<WebSocketConnection>>>>,
    room_manager: Arc<RoomManager>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            room_manager: Arc::new(RoomManager::new()),
        }
    }

    pub async fn add_connection(&self, conn: Arc<WebSocketConnection>) {
        let mut connections = self.connections.write().await;
        connections.insert(conn.id.clone(), conn);
    }

    pub async fn remove_connection(&self, conn_id: &str) {
        let mut connections = self.connections.write().await;
        connections.remove(conn_id);
        
        // Remove from all rooms
        self.room_manager.remove_from_all_rooms(conn_id).await;
    }

    pub async fn broadcast(&self, message: Message) -> Result<()> {
        let connections = self.connections.read().await;
        let mut failed = 0usize;
        for conn in connections.values() {
            if conn.send(message.clone()).is_err() {
                failed += 1;
            }
        }
        if failed > 0 {
            return Err(WebSocketError::PartialSend { failed });
        }
        Ok(())
    }

    pub async fn send_to_user(&self, user_id: &str, message: Message) -> Result<()> {
        let connections = self.connections.read().await;
        let mut matched = 0usize;
        for conn in connections.values() {
            if conn.user_id.as_deref() == Some(user_id) {
                matched += 1;
                conn.send(message.clone())?;
            }
        }
        if matched == 0 {
            return Err(WebSocketError::UserNotConnected(user_id.to_string()));
        }
        Ok(())
    }

    pub fn room_manager(&self) -> Arc<RoomManager> {
        self.room_manager.clone()
    }

    /// Number of active websocket connections.
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }
}

impl Default for WebSocketManager {
    fn default() -> Self {
        Self::new()
    }
}

/// WebSocket errors
#[derive(Debug, thiserror::Error)]
pub enum WebSocketError {
    #[error("Failed to send message")]
    SendError,

    #[error("Failed to send message to {failed} connection(s)")]
    PartialSend { failed: usize },
    
    #[error("Invalid message format")]
    InvalidMessage,
    
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Room not found")]
    RoomNotFound,

    #[error("No active connection found for user `{0}`")]
    UserNotConnected(String),
}

pub type Result<T> = std::result::Result<T, WebSocketError>;

#[cfg(test)]
mod tests {
    use super::{Message, WebSocketConnection, WebSocketManager, WebSocketError};

    #[test]
    fn message_roundtrip_json() {
        let message = Message::json(serde_json::json!({"k": "v"}));
        let ws = message.to_ws_message().expect("to ws");
        let parsed = Message::from_ws_message(ws).expect("from ws");
        match parsed {
            Message::Json { data } => assert_eq!(data["k"], "v"),
            other => panic!("expected json message, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn send_to_unknown_user_returns_error() {
        let manager = WebSocketManager::new();
        let err = manager
            .send_to_user("missing-user", Message::text("hello"))
            .await
            .expect_err("expected user missing error");
        match err {
            WebSocketError::UserNotConnected(user) => assert_eq!(user, "missing-user"),
            other => panic!("unexpected error: {other}"),
        }
    }

    #[tokio::test]
    async fn connection_count_tracks_lifecycle() {
        let manager = WebSocketManager::new();
        let (conn, _rx) = WebSocketConnection::new(Some("u1".to_string()));
        let conn = std::sync::Arc::new(conn);
        let id = conn.id.clone();

        manager.add_connection(conn).await;
        assert_eq!(manager.connection_count().await, 1);

        manager.remove_connection(&id).await;
        assert_eq!(manager.connection_count().await, 0);
    }
}
