//! # Oxidite Realtime
//!
//! Realtime features for the Oxidite framework including Server-Sent Events (SSE),
//! pub/sub messaging, and event broadcasting.

pub mod sse;
pub mod pubsub;
pub mod event;
pub mod websocket;

pub use sse::{SseEvent, SseStream, SseConfig};
pub use pubsub::{PubSub, Subscriber, Channel};
pub use event::{Event, EventType};
pub use websocket::{WebSocketConnection, WebSocketManager, Message as WsMessage, WebSocketError};

use thiserror::Error;

/// Realtime errors
#[derive(Error, Debug)]
pub enum RealtimeError {
    #[error("Channel not found: {0}")]
    ChannelNotFound(String),
    
    #[error("Subscriber disconnected")]
    Disconnected,
    
    #[error("Failed to send event: {0}")]
    SendError(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, RealtimeError>;
