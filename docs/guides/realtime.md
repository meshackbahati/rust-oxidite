# Realtime Guide

## Overview

Oxidite provides realtime features through WebSockets, Server-Sent Events (SSE), and Pub/Sub messaging.

## WebSockets

### Basic Setup

```rust
use oxidite_realtime::{WebSocketManager, WebSocketConnection, WsMessage};
use std::sync::Arc;

let ws_manager = Arc::new(WebSocketManager::new());

// Create connection
let (conn, mut rx) = WebSocketConnection::new(Some("user123".to_string()));
let conn = Arc::new(conn);

// Add to manager
ws_manager.add_connection(conn.clone()).await;

// Send message
conn.send(WsMessage::text("Hello!"))?;
```

### Broadcasting

```rust
// Broadcast to all connections
ws_manager.broadcast(WsMessage::text("Server announcement")).await?;

// Send to specific user
ws_manager.send_to_user("user123", WsMessage::json(
    serde_json::json!({
        "type": "notification",
        "message": "You have a new message"
    })
)).await?;
```

### Rooms

Group connections into rooms for targeted messaging:

```rust
let room_manager = ws_manager.room_manager();

// Create room
room_manager.create_room("chat-general".to_string()).await?;

// Join room
room_manager.join_room("chat-general", conn.id.clone()).await?;

// Broadcast to room
room_manager.broadcast_to_room(
    "chat-general",
    WsMessage::text("Room message"),
    &ws_manager
).await?;

// Leave room
room_manager.leave_room("chat-general", &conn.id).await?;

// List rooms
let rooms = room_manager.list_rooms().await;
```

## Server-Sent Events (SSE)

SSE provides one-way communication from server to client:

```rust
use oxidite_realtime::{SseEvent, SseConfig};

// Create SSE event
let event = SseEvent::new()
    .event("message")
    .data("Hello from server!")
    .id("1")
    .retry(5000);

// Format for transmission
let formatted = event.format();

// Send to client
// (In a real handler, you'd stream these events)
```

### SSE Configuration

```rust
let config = SseConfig::new()
    .with_heartbeat_interval(30)
    .with_retry(5000);
```

## Pub/Sub

Internal pub/sub system for event notification:

```rust
use oxidite_realtime::{PubSub, Event, EventType};

let pubsub = PubSub::new();

// Create channel
let channel = pubsub.create_channel("notifications").await;

// Subscribe
let mut subscriber = pubsub.subscribe("notifications").await?;

// Publish event
let event = Event::message("user-joined", serde_json::json!({
    "user_id": "123",
    "username": "Alice"
}));

pubsub.publish("notifications", event).await?;

// Receive events
tokio::spawn(async move {
    while let Ok(event) = subscriber.receive().await {
        println!("Received: {:?}", event);
    }
});
```

### Event Types

```rust
use oxidite_realtime::{Event, EventType};

// Different event types
let msg = Event::message("topic", data);
let notif = Event::notification("topic", data);
let update = Event::update("topic", data);
let system = Event::system("topic", data);
let custom = Event::custom("topic", "custom_type", data);
```

## Integration Example

### Chat Application

```rust
use oxidite_realtime::{WebSocketManager, WsMessage};
use std::sync::Arc;

struct ChatServer {
    ws_manager: Arc<WebSocketManager>,
}

impl ChatServer {
    async fn handle_message(&self, conn_id: &str, message: WsMessage) {
        let room_manager = self.ws_manager.room_manager();
        
        // Broadcast to all users in the same room
        if let Some(room_name) = self.get_user_room(conn_id).await {
            room_manager.broadcast_to_room(
                &room_name,
                message,
                &self.ws_manager
            ).await.ok();
        }
    }
    
    async fn user_joined(&self, conn_id: &str, room: &str) {
        let room_manager = self.ws_manager.room_manager();
        room_manager.join_room(room, conn_id.to_string()).await.ok();
        
        // Notify others
        room_manager.broadcast_to_room(
            room,
            WsMessage::json(serde_json::json!({
                "type": "user_joined",
                "user": conn_id
            })),
            &self.ws_manager
        ).await.ok();
    }
}
```

## Best Practices

1. **Handle disconnections** - Clean up resources when clients disconnect
2. **Use rooms** - Group related connections for efficient messaging
3. **Authentication** - Verify user identity before allowing connections
4. **Rate limiting** - Prevent message flooding
5. **Heartbeats** - Keep connections alive with periodic pings

## Next Steps

- [Templating Guide](templating.md) - Server-side rendering
- [Deployment Guide](deployment.md) - Production deployment
