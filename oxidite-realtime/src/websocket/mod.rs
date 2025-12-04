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
        for conn in connections.values() {
            let _ = conn.send(message.clone());
        }
        Ok(())
    }

    pub async fn send_to_user(&self, user_id: &str, message: Message) -> Result<()> {
        let connections = self.connections.read().await;
        for conn in connections.values() {
            if conn.user_id.as_deref() == Some(user_id) {
                conn.send(message.clone())?;
            }
        }
        Ok(())
    }

    pub fn room_manager(&self) -> Arc<RoomManager> {
        self.room_manager.clone()
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
    
    #[error("Invalid message format")]
    InvalidMessage,
    
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Room not found")]
    RoomNotFound,
}

pub type Result<T> = std::result::Result<T, WebSocketError>;
