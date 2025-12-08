# Real-time & WebSocket Guide

Build real-time applications with WebSockets in Oxidite.

## Installation

```toml
[dependencies]
oxidite = { version = "1.0", features = ["realtime"] }
```

## Quick Start

```rust
use oxidite::prelude::*;
use oxidite_realtime::WebSocketManager;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    ws_manager: Arc<WebSocketManager>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let ws_manager = Arc::new(WebSocketManager::new());
    let state = AppState { ws_manager };
    
    let mut app = Router::new();
    
    // WebSocket endpoint
    app.get("/ws", |State(state): State<AppState>, req: OxiditeRequest| async {
        state.ws_manager.handle_upgrade(req).await
    });
    
    let app = app.with_state(state);

    Server::new(app).listen("127.0.0.1:3000".parse()?).await
}
```

## Broadcasting

```rust
// Broadcast to all clients
state.ws_manager.broadcast("Hello everyone!").await;

// Broadcast to room
state.ws_manager.broadcast_to("room1", "Room message").await;
```

## Direct Messaging

```rust
// Send to specific client
state.ws_manager.send_to(client_id, "Direct message").await;
```

## Room Management

```rust
// Join room
state.ws_manager.join(client_id, "chat-room").await;

// Leave room
state.ws_manager.leave(client_id, "chat-room").await;

// List clients in room
let clients = state.ws_manager.clients_in_room("chat-room").await;
```

## Complete Chat Example

```rust
use oxidite::prelude::*;
use oxidite_realtime::WebSocketManager;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    user: String,
    message: String,
    room: String,
}

#[derive(Clone)]
struct AppState {
    ws_manager: Arc<WebSocketManager>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let ws_manager = Arc::new(WebSocketManager::new());
    let state = AppState { ws_manager };
    
    let mut app = Router::new();
    
    // WebSocket connection
    app.get("/ws", |State(state): State<AppState>, req: OxiditeRequest| async {
        state.ws_manager.handle_upgrade(req).await
    });
    
    // Send message to room
    app.post("/chat/send", |State(state): State<AppState>, Json(msg): Json<ChatMessage>| async move {
        state.ws_manager.broadcast_to(&msg.room, &serde_json::to_string(&msg)?).await;
        Ok(Json(json!({ "status": "sent" })))
    });
    
    let app = app.with_state(state);

    Server::new(app).listen("127.0.0.1:3000".parse()?).await
}
```

## Client (JavaScript)

```javascript
const ws = new WebSocket('ws://localhost:3000/ws');

ws.onopen = () => {
    console.log('Connected');
    ws.send(JSON.stringify({
        type: 'join',
        room: 'general'
    }));
};

ws.onmessage = (event) => {
    const data = JSON.parse(event.data);
    console.log('Received:', data);
};

// Send message via HTTP POST to /chat/send
async function sendMessage(room, user, message) {
    await fetch('/chat/send', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ room, user, message })
    });
}
```
