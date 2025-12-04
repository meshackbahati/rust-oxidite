//! Pub/Sub messaging system

use crate::{Event, Result, RealtimeError};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// A pub/sub channel
pub struct Channel {
    name: String,
    sender: broadcast::Sender<Event>,
}

impl Channel {
    /// Create a new channel
    pub fn new(name: impl Into<String>, capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            name: name.into(),
            sender,
        }
    }

    /// Get the channel name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Publish an event to the channel
    pub fn publish(&self, event: Event) -> Result<usize> {
        self.sender
            .send(event)
            .map_err(|_| RealtimeError::SendError("No subscribers".to_string()))
    }

    /// Subscribe to the channel
    pub fn subscribe(&self) -> Subscriber {
        Subscriber {
            receiver: self.sender.subscribe(),
        }
    }

    /// Get the number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

/// A channel subscriber
pub struct Subscriber {
    receiver: broadcast::Receiver<Event>,
}

impl Subscriber {
    /// Receive the next event
    pub async fn recv(&mut self) -> Result<Event> {
        self.receiver
            .recv()
            .await
            .map_err(|_| RealtimeError::Disconnected)
    }
}

/// Pub/Sub manager
pub struct PubSub {
    channels: Arc<RwLock<HashMap<String, Arc<Channel>>>>,
    default_capacity: usize,
}

impl PubSub {
    /// Create a new pub/sub manager
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            default_capacity: 100,
        }
    }

    /// Set default channel capacity
    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.default_capacity = capacity;
        self
    }

    /// Create or get a channel
    pub async fn channel(&self, name: &str) -> Arc<Channel> {
        let channels = self.channels.read().await;
        if let Some(channel) = channels.get(name) {
            return channel.clone();
        }
        drop(channels);

        let mut channels = self.channels.write().await;
        channels
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(Channel::new(name, self.default_capacity)))
            .clone()
    }

    /// Publish an event to a channel
    pub async fn publish(&self, channel_name: &str, event: Event) -> Result<usize> {
        let channel = self.channel(channel_name).await;
        channel.publish(event)
    }

    /// Subscribe to a channel
    pub async fn subscribe(&self, channel_name: &str) -> Subscriber {
        let channel = self.channel(channel_name).await;
        channel.subscribe()
    }

    /// Remove a channel
    pub async fn remove_channel(&self, name: &str) {
        let mut channels = self.channels.write().await;
        channels.remove(name);
    }

    /// List all channel names
    pub async fn channels(&self) -> Vec<String> {
        let channels = self.channels.read().await;
        channels.keys().cloned().collect()
    }
}

impl Default for PubSub {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EventType;

    #[tokio::test]
    async fn test_pubsub() {
        let pubsub = PubSub::new();
        
        // Subscribe first
        let mut sub = pubsub.subscribe("test").await;
        
        // Publish
        let event = Event::new(
            EventType::Message,
            "test",
            serde_json::json!({"hello": "world"}),
        );
        
        let count = pubsub.publish("test", event.clone()).await.unwrap();
        assert_eq!(count, 1);
        
        // Receive
        let received = sub.recv().await.unwrap();
        assert_eq!(received.id, event.id);
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let pubsub = PubSub::new();
        
        let mut sub1 = pubsub.subscribe("test").await;
        let mut sub2 = pubsub.subscribe("test").await;
        
        let event = Event::message("test", serde_json::json!({}));
        let count = pubsub.publish("test", event).await.unwrap();
        
        assert_eq!(count, 2);
        
        let r1 = sub1.recv().await;
        let r2 = sub2.recv().await;
        
        assert!(r1.is_ok());
        assert!(r2.is_ok());
    }
}
