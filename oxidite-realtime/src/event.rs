//! Event types for realtime messaging

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Event type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventType {
    /// A message event
    Message,
    /// A notification event
    Notification,
    /// A data update event
    Update,
    /// A system event
    System,
    /// Custom event type
    Custom(String),
}

/// Generic event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Unique event ID
    pub id: String,
    /// Event type
    #[serde(rename = "type")]
    pub event_type: EventType,
    /// Event channel/topic
    pub channel: String,
    /// Event payload
    pub data: serde_json::Value,
    /// Timestamp
    pub timestamp: u64,
}

impl Event {
    /// Create a new event
    pub fn new(event_type: EventType, channel: impl Into<String>, data: serde_json::Value) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_type,
            channel: channel.into(),
            data,
            timestamp,
        }
    }

    /// Create a message event
    pub fn message(channel: impl Into<String>, data: serde_json::Value) -> Self {
        Self::new(EventType::Message, channel, data)
    }

    /// Create a notification event
    pub fn notification(channel: impl Into<String>, data: serde_json::Value) -> Self {
        Self::new(EventType::Notification, channel, data)
    }

    /// Create an update event
    pub fn update(channel: impl Into<String>, data: serde_json::Value) -> Self {
        Self::new(EventType::Update, channel, data)
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = Event::message("test-channel", serde_json::json!({"msg": "hello"}));
        
        assert_eq!(event.channel, "test-channel");
        assert_eq!(event.event_type, EventType::Message);
        assert!(event.timestamp > 0);
    }
}
