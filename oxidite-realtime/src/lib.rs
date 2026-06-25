//! # Oxidite Realtime
//!
//! Realtime features for the Oxidite framework including Server-Sent Events (SSE),
//! pub/sub messaging, and event broadcasting.

/// Server-Sent Events support.
pub mod sse;
/// Publish/subscribe messaging.
pub mod pubsub;
/// Event types for realtime messaging.
pub mod event;
/// WebSocket support.
pub mod websocket;

/// Re-export of SSE types.
pub use sse::{SseEvent, SseStream, SseConfig};
/// Re-export of pub/sub types.
pub use pubsub::{PubSub, Subscriber, Channel};
/// Re-export of event types.
pub use event::{Event, EventType};
/// Re-export of WebSocket types.
pub use websocket::{WebSocketConnection, WebSocketManager, Message, WebSocketError};
/// Re-export of the WebSocket stream type.
pub use tokio_tungstenite::WebSocketStream;

use thiserror::Error;

/// Realtime errors
#[derive(Error, Debug)]
pub enum RealtimeError {
    #[error("Channel not found: {0}")]
    ChannelNotFound(String),
    
    #[error("Subscriber disconnected")]
    Disconnected,

    #[error("Subscriber lagged behind and missed {0} message(s)")]
    Lagged(u64),
    
    #[error("Failed to send event: {0}")]
    SendError(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Convenience alias for realtime operation results.
pub type Result<T> = std::result::Result<T, RealtimeError>;
