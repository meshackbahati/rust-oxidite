# oxidite-realtime

WebSocket and real-time features for Oxidite.

## Installation

```toml
[dependencies]
oxidite-realtime = "2.1.0"
```

## Usage

```rust
use oxidite_realtime::{Event, Message, PubSub, WebSocketManager};

#[tokio::main]
async fn main() {
    // Pub/Sub
    let pubsub = PubSub::new();
    let mut subscriber = pubsub.subscribe("news").await;
    let event = Event::message("news", serde_json::json!({"headline": "hello"}));
    let _ = pubsub.publish("news", event).await;
    let _ = subscriber.recv().await;

    // WebSocket manager
    let ws = WebSocketManager::new();
    let _ = ws.broadcast(Message::text("system broadcast")).await;
}
```

## Features

- WebSocket support
- Room management
- Pub/sub messaging
- Direct messaging
- SSE event formatting helpers

## License

MIT
