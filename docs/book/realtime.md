# Real-time Features

Real-time features enable live updates, bidirectional communication, and interactive experiences in your Oxidite applications. This chapter covers WebSocket support, Server-Sent Events (SSE), and pub/sub messaging.

## Overview

Real-time features in Oxidite include:
- WebSocket connections for bidirectional communication
- Server-Sent Events for unidirectional server-to-client updates
- Pub/Sub messaging for event distribution
- Live updates and notifications
- Real-time collaboration features

## WebSocket Support

WebSockets provide full-duplex communication channels over a single TCP connection:

```rust
use oxidite::prelude::*;
use oxidite_realtime::websocket::{WebSocket, Message, WebSocketHandler};

async fn websocket_handler(ws: WebSocket) -> Result<()> {
    // Set up message handler
    ws.on_message(|msg| async move {
        match msg {
            Message::Text(text) => {
                println!("Received text: {}", text);
                
                // Echo the message back
                Ok(Message::Text(format!("Echo: {}", text)))
            }
            Message::Binary(data) => {
                println!("Received binary: {} bytes", data.len());
                
                // Echo the binary data back
                Ok(Message::Binary(data))
            }
            Message::Ping(data) => {
                // Respond with pong
                Ok(Message::Pong(data))
            }
            Message::Pong(_) => {
                // Pong received, usually no action needed
                Ok(Message::Pong(vec![]))
            }
            Message::Close(frame) => {
                // Close frame received
                Ok(Message::Close(frame))
            }
        }
    }).await?;
    
    // Handle connection close
    ws.on_close(|reason| async move {
        println!("WebSocket closed: {:?}", reason);
    }).await?;
    
    Ok(())
}

// WebSocket upgrade endpoint
async fn websocket_upgrade(_req: Request) -> Result<Response> {
    // This would typically be handled by the framework
    // The actual WebSocket handler is registered separately
    Ok(Response::text("WebSocket endpoint".to_string()))
}
```

## WebSocket with State Management

Manage WebSocket connections with shared state:

```rust
use oxidite::prelude::*;
use oxidite_realtime::websocket::{WebSocket, Message};
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone)]
struct ChatState {
    users: Arc<tokio::sync::Mutex<std::collections::HashMap<String, WebSocket>>>,
    broadcast_tx: broadcast::Sender<String>,
}

async fn chat_websocket_handler(ws: WebSocket, state: ChatState) -> Result<()> {
    // Generate a unique user ID
    let user_id = uuid::Uuid::new_v4().to_string();
    
    // Add user to chat room
    {
        let mut users = state.users.lock().await;
        users.insert(user_id.clone(), ws.clone());
    }
    
    // Send welcome message
    ws.send(Message::Text(format!("Welcome to chat, {}!", user_id))).await?;
    
    // Listen for messages
    ws.on_message(move |msg| {
        let state = state.clone();
        let user_id = user_id.clone();
        
        async move {
            match msg {
                Message::Text(text) => {
                    // Broadcast message to all users
                    let message = format!("[{}] {}", user_id, text);
                    
                    if state.broadcast_tx.send(message.clone()).is_err() {
                        // Channel is closed, return error
                        return Err("Broadcast channel closed".to_string());
                    }
                    
                    Ok(Message::Text("Message sent".to_string()))
                }
                Message::Binary(_) => {
                    Ok(Message::Text("Binary messages not supported".to_string()))
                }
                _ => Ok(msg), // Return other messages as-is
            }
        }
    }).await?;
    
    // Listen for broadcasts
    let mut rx = state.broadcast_tx.subscribe();
    let ws_clone = ws.clone();
    
    tokio::spawn(async move {
        while let Ok(message) = rx.recv().await {
            if let Err(e) = ws_clone.send(Message::Text(message)).await {
                eprintln!("Failed to send broadcast: {}", e);
                break;
            }
        }
    });
    
    // Handle connection close
    ws.on_close({
        let state = state.clone();
        let user_id = user_id.clone();
        
        move |_| {
            let state = state.clone();
            let user_id = user_id.clone();
            
            async move {
                let mut users = state.users.lock().await;
                users.remove(&user_id);
                println!("User {} left chat", user_id);
            }
        }
    }).await?;
    
    Ok(())
}
```

## Server-Sent Events (SSE)

Server-Sent Events provide unidirectional server-to-client communication:

```rust
use oxidite::prelude::*;
use oxidite_realtime::sse::EventStream;

async fn sse_handler(_req: Request) -> Result<Response> {
    let mut stream = EventStream::new();
    
    // Send initial connection event
    stream.send("Connected", Some("connection"), None).await?;
    
    // Send periodic updates
    let stream_clone = stream.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
        
        loop {
            interval.tick().await;
            
            let timestamp = chrono::Utc::now().to_rfc3339();
            let data = format!("{{\"timestamp\": \"{}\", \"message\": \"Periodic update\"}}", timestamp);
            
            if let Err(e) = stream_clone.send(&data, Some("periodic"), None).await {
                eprintln!("Failed to send SSE: {}", e);
                break;
            }
        }
    });
    
    // Send live metrics
    let stream_clone = stream.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
        
        loop {
            interval.tick().await;
            
            // Simulate some metrics
            let metrics = serde_json::json!({
                "users_active": 42,
                "messages_sent": 1234,
                "server_uptime": "24h"
            });
            
            if let Err(e) = stream_clone.send(&metrics.to_string(), Some("metrics"), None).await {
                eprintln!("Failed to send metrics SSE: {}", e);
                break;
            }
        }
    });
    
    Ok(stream.response())
}

// SSE endpoint with authentication
async fn authenticated_sse_handler(
    _req: Request,
    _user: AuthenticatedUser  // Assume this comes from auth middleware
) -> Result<Response> {
    let mut stream = EventStream::new();
    
    // Send user-specific data
    stream.send(
        &format!("{{\"user_id\": \"{}\", \"message\": \"Welcome to personalized feed\"}}", _user.id),
        Some("welcome"),
        None
    ).await?;
    
    Ok(stream.response())
}

#[derive(Clone)]
struct AuthenticatedUser {
    id: String,
    role: String,
}
```

## Pub/Sub Messaging

Implement publish-subscribe messaging for event distribution:

```rust
use oxidite::prelude::*;
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct PubSub {
    channels: Arc<tokio::sync::RwLock<std::collections::HashMap<String, broadcast::Sender<Event>>>>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Event {
    pub topic: String,
    pub data: serde_json::Value,
    pub timestamp: String,
    pub sender: Option<String>,
}

impl PubSub {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }
    
    pub async fn subscribe(&self, topic: &str) -> broadcast::Receiver<Event> {
        let mut channels = self.channels.write().await;
        
        if !channels.contains_key(topic) {
            let (tx, _) = broadcast::channel(100); // Buffer 100 messages
            channels.insert(topic.to_string(), tx);
        }
        
        let tx = channels.get(topic).unwrap();
        tx.subscribe()
    }
    
    pub async fn publish(&self, event: Event) -> Result<()> {
        let channels = self.channels.read().await;
        
        if let Some(tx) = channels.get(&event.topic) {
            if tx.send(event).is_err() {
                // Receiver dropped, channel is empty
            }
        }
        
        Ok(())
    }
    
    pub async fn create_topic(&self, topic: &str) -> Result<()> {
        let mut channels = self.channels.write().await;
        
        if !channels.contains_key(topic) {
            let (tx, _) = broadcast::channel(100);
            channels.insert(topic.to_string(), tx);
        }
        
        Ok(())
    }
}

// Example usage in a handler
async fn pubsub_example(
    Json(payload): Json<serde_json::Value>,
    State(pubsub): State<Arc<PubSub>>
) -> Result<Response> {
    let event = Event {
        topic: "user_activity".to_string(),
        data: payload,
        timestamp: chrono::Utc::now().to_rfc3339(),
        sender: Some("api".to_string()),
    };
    
    pubsub.publish(event).await?;
    
    Ok(Response::json(serde_json::json!({ "status": "published" })))
}

// Subscribe to events in a WebSocket
async fn event_stream_websocket_handler(
    ws: WebSocket,
    State(pubsub): State<Arc<PubSub>>
) -> Result<()> {
    let mut receiver = pubsub.subscribe("notifications").await;
    
    // Forward events to WebSocket
    let ws_clone = ws.clone();
    tokio::spawn(async move {
        while let Ok(event) = receiver.recv().await {
            if let Err(e) = ws_clone.send(Message::Text(serde_json::to_string(&event).unwrap())).await {
                eprintln!("Failed to send event to WebSocket: {}", e);
                break;
            }
        }
    });
    
    Ok(())
}
```

## Real-time Notifications

Build a notification system with real-time delivery:

```rust
use oxidite::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct NotificationService {
    subscribers: Arc<tokio::sync::RwLock<HashMap<String, broadcast::Sender<Notification>>>>,
    pubsub: Arc<PubSub>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Notification {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub body: String,
    pub category: String,
    pub timestamp: String,
    pub read: bool,
}

impl NotificationService {
    pub fn new(pubsub: Arc<PubSub>) -> Self {
        Self {
            subscribers: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            pubsub,
        }
    }
    
    pub async fn subscribe_user(&self, user_id: &str) -> broadcast::Receiver<Notification> {
        let mut subscribers = self.subscribers.write().await;
        
        if !subscribers.contains_key(user_id) {
            let (tx, _) = broadcast::channel(50);
            subscribers.insert(user_id.to_string(), tx);
        }
        
        let tx = subscribers.get(user_id).unwrap();
        tx.subscribe()
    }
    
    pub async fn send_notification(&self, notification: Notification) -> Result<()> {
        // Publish to user-specific channel
        let user_event = Event {
            topic: format!("notifications:{}", notification.user_id),
            data: serde_json::to_value(&notification)?,
            timestamp: chrono::Utc::now().to_rfc3339(),
            sender: Some("notification_service".to_string()),
        };
        
        self.pubsub.publish(user_event).await?;
        
        // Also send to user's subscription if online
        if let Some(tx) = self.subscribers.read().await.get(&notification.user_id) {
            let _ = tx.send(notification.clone());
        }
        
        Ok(())
    }
    
    pub async fn get_user_notifications(&self, user_id: &str) -> Result<Vec<Notification>> {
        // In a real app, this would fetch from database
        Ok(vec![])
    }
    
    pub async fn mark_as_read(&self, user_id: &str, notification_id: &str) -> Result<()> {
        // In a real app, this would update database
        Ok(())
    }
}

// Notification WebSocket handler
async fn notification_websocket_handler(
    ws: WebSocket,
    State(notification_service): State<Arc<NotificationService>>,
    user: AuthenticatedUser
) -> Result<()> {
    let mut receiver = notification_service.subscribe_user(&user.id).await;
    
    // Send existing unread notifications
    let existing_notifications = notification_service.get_user_notifications(&user.id).await?;
    for notification in existing_notifications {
        ws.send(Message::Text(serde_json::to_string(&notification)?)).await?;
    }
    
    // Listen for new notifications
    let ws_clone = ws.clone();
    tokio::spawn(async move {
        while let Ok(notification) = receiver.recv().await {
            if let Err(e) = ws_clone.send(Message::Text(serde_json::to_string(&notification).unwrap())).await {
                eprintln!("Failed to send notification: {}", e);
                break;
            }
        }
    });
    
    Ok(())
}
```

## Real-time Analytics

Track real-time metrics and analytics:

```rust
use oxidite::prelude::*;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct AnalyticsService {
    event_sender: mpsc::UnboundedSender<AnalyticsEvent>,
    metrics: Arc<tokio::sync::RwLock<Metrics>>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct AnalyticsEvent {
    pub event_type: String,
    pub user_id: Option<String>,
    pub properties: std::collections::HashMap<String, serde_json::Value>,
    pub timestamp: String,
    pub session_id: Option<String>,
}

#[derive(Default, Clone)]
pub struct Metrics {
    pub page_views: u64,
    pub unique_visitors: std::collections::HashSet<String>,
    pub active_users: std::collections::HashMap<String, chrono::DateTime<chrono::Utc>>,
    pub event_counts: std::collections::HashMap<String, u64>,
}

impl AnalyticsService {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<AnalyticsEvent>) {
        let (sender, receiver) = mpsc::unbounded_channel();
        
        let service = Self {
            event_sender: sender,
            metrics: Arc::new(tokio::sync::RwLock::new(Metrics::default())),
        };
        
        (service, receiver)
    }
    
    pub fn track_event(&self, event: AnalyticsEvent) -> Result<()> {
        self.event_sender.send(event)
            .map_err(|e| Error::InternalServerError(format!("Failed to track event: {}", e)))?;
        Ok(())
    }
    
    pub async fn get_metrics(&self) -> Metrics {
        self.metrics.read().await.clone()
    }
    
    pub async fn start_processing(&self) {
        let metrics = self.metrics.clone();
        
        // Spawn a task to process events
        let mut event_receiver = {
            let (sender, receiver) = mpsc::unbounded_channel();
            // We'd need to clone the original sender to return the receiver
            // This is a simplified example
            receiver
        };
        
        tokio::spawn(async move {
            while let Ok(event) = event_receiver.recv().await {
                let mut metrics = metrics.write().await;
                
                // Update metrics based on event
                match event.event_type.as_str() {
                    "page_view" => metrics.page_views += 1,
                    "user_login" => {
                        if let Some(user_id) = event.user_id {
                            metrics.unique_visitors.insert(user_id);
                        }
                    }
                    _ => {
                        *metrics.event_counts.entry(event.event_type).or_insert(0) += 1;
                    }
                }
                
                // Track active users (within last 5 minutes)
                if let Some(user_id) = event.user_id {
                    metrics.active_users.insert(
                        user_id,
                        chrono::Utc::now()
                    );
                }
                
                // Clean up old active users periodically
                let now = chrono::Utc::now();
                metrics.active_users.retain(|_, timestamp| {
                    (now - *timestamp).num_minutes() < 5
                });
            }
        });
    }
}

// Analytics tracking endpoint
async fn track_analytics(
    Json(event): Json<AnalyticsEvent>,
    State(analytics): State<Arc<AnalyticsService>>
) -> Result<Response> {
    analytics.track_event(event)?;
    
    Ok(Response::json(serde_json::json!({ "status": "tracked" })))
}

// Real-time metrics endpoint
async fn real_time_metrics(State(analytics): State<Arc<AnalyticsService>>) -> Result<Response> {
    let metrics = analytics.get_metrics().await;
    
    Ok(Response::json(serde_json::json!({
        "page_views": metrics.page_views,
        "unique_visitors": metrics.unique_visitors.len(),
        "active_users": metrics.active_users.len(),
        "event_counts": metrics.event_counts,
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}
```

## Performance Optimization

Optimize real-time features for performance:

```rust
use oxidite::prelude::*;

pub struct RealTimeConfig {
    pub websocket_max_connections: usize,
    pub sse_buffer_size: usize,
    pub broadcast_channel_capacity: usize,
    pub heartbeat_interval: std::time::Duration,
    pub connection_timeout: std::time::Duration,
}

impl RealTimeConfig {
    pub fn default_production() -> Self {
        Self {
            websocket_max_connections: 10_000,
            sse_buffer_size: 100,
            broadcast_channel_capacity: 1000,
            heartbeat_interval: std::time::Duration::from_secs(30),
            connection_timeout: std::time::Duration::from_secs(60),
        }
    }
    
    pub fn default_development() -> Self {
        Self {
            websocket_max_connections: 100,
            sse_buffer_size: 10,
            broadcast_channel_capacity: 100,
            heartbeat_interval: std::time::Duration::from_secs(60),
            connection_timeout: std::time::Duration::from_secs(300),
        }
    }
}

// Connection pool for WebSockets
pub struct WebSocketPool {
    connections: std::collections::HashMap<String, WebSocket>,
    config: RealTimeConfig,
}

impl WebSocketPool {
    pub fn new(config: RealTimeConfig) -> Self {
        Self {
            connections: std::collections::HashMap::new(),
            config,
        }
    }
    
    pub fn add_connection(&mut self, id: String, ws: WebSocket) -> Result<()> {
        if self.connections.len() >= self.config.websocket_max_connections {
            return Err(Error::InternalServerError("Maximum connections reached".to_string()));
        }
        
        self.connections.insert(id, ws);
        Ok(())
    }
    
    pub fn remove_connection(&mut self, id: &str) -> Option<WebSocket> {
        self.connections.remove(id)
    }
    
    pub fn broadcast_to_all(&self, message: Message) -> Result<()> {
        for (_, ws) in &self.connections {
            // Note: This is simplified; in practice, you'd handle errors per connection
            let _ = ws.send(message.clone());
        }
        Ok(())
    }
}
```

## Security Considerations

Secure real-time features properly:

```rust
use oxidite::prelude::*;

// Secure WebSocket middleware
async fn secure_websocket_middleware(
    req: Request,
    next: Next,
    State(rate_limiter): State<Arc<RateLimiter>>
) -> Result<Response> {
    // Rate limiting for WebSocket connections
    let client_ip = req.headers()
        .get("x-forwarded-for")
        .and_then(|hv| hv.to_str().ok())
        .unwrap_or("unknown");
    
    if !rate_limiter.is_allowed(client_ip, "websocket").await {
        return Err(Error::TooManyRequests);
    }
    
    // Validate origin for WebSocket upgrades
    if let Some(origin) = req.headers().get("origin") {
        if !is_valid_origin(origin)? {
            return Err(Error::Forbidden("Invalid origin".to_string()));
        }
    }
    
    next.run(req).await
}

fn is_valid_origin(origin: &http::HeaderValue) -> Result<bool> {
    let origin_str = origin.to_str().map_err(|_| Error::BadRequest("Invalid origin header".to_string()))?;
    
    // In a real app, validate against allowed origins
    let allowed_origins = ["http://localhost:3000", "https://yourdomain.com"];
    Ok(allowed_origins.iter().any(|&allowed| origin_str.starts_with(allowed)))
}

// Rate limiter for real-time features
#[derive(Clone)]
pub struct RateLimiter {
    limits: Arc<tokio::sync::Mutex<std::collections::HashMap<String, ClientLimits>>>,
    max_messages_per_minute: u32,
}

#[derive(Default)]
struct ClientLimits {
    message_count: u32,
    last_reset: std::time::Instant,
}

impl RateLimiter {
    pub fn new(max_messages_per_minute: u32) -> Self {
        Self {
            limits: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
            max_messages_per_minute,
        }
    }
    
    pub async fn is_allowed(&self, client_id: &str, _feature: &str) -> bool {
        let mut limits = self.limits.lock().await;
        let now = std::time::Instant::now();
        
        let client_limit = limits.entry(client_id.to_string()).or_default();
        
        // Reset counter if more than a minute has passed
        if now.duration_since(client_limit.last_reset).as_secs() >= 60 {
            client_limit.message_count = 0;
            client_limit.last_reset = now;
        }
        
        if client_limit.message_count >= self.max_messages_per_minute {
            return false;
        }
        
        client_limit.message_count += 1;
        true
    }
}

// Message validation
pub struct MessageValidator;

impl MessageValidator {
    pub fn validate_websocket_message(&self, msg: &Message) -> Result<()> {
        match msg {
            Message::Text(text) => {
                // Check message size
                if text.len() > 64 * 1024 { // 64KB limit
                    return Err(Error::PayloadTooLarge);
                }
                
                // Check for malicious content
                if contains_malicious_content(text) {
                    return Err(Error::BadRequest("Malicious content detected".to_string()));
                }
                
                Ok(())
            }
            Message::Binary(data) => {
                // Check binary size
                if data.len() > 1024 * 1024 { // 1MB limit
                    return Err(Error::PayloadTooLarge);
                }
                
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

fn contains_malicious_content(text: &str) -> bool {
    // Simple check for potential malicious patterns
    let dangerous_patterns = ["<script", "javascript:", "vbscript:", "onload=", "onerror="];
    dangerous_patterns.iter().any(|pattern| text.to_lowercase().contains(pattern))
}
```

## Integration with Frontend

Provide frontend integration examples:

```rust
use oxidite::prelude::*;

// Endpoint to get WebSocket connection details
async fn websocket_config(_user: AuthenticatedUser) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "websocket_url": "ws://localhost:3000/ws",
        "heartbeat_interval": 30000,
        "reconnect_attempts": 5,
        "reconnect_delay": 1000
    })))
}

// Frontend JavaScript example (as documentation):
/*
// Connect to WebSocket
const wsUrl = 'ws://localhost:3000/ws';
const socket = new WebSocket(wsUrl);

socket.onopen = function(event) {
    console.log('Connected to WebSocket');
    // Send initial authentication
    socket.send(JSON.stringify({
        type: 'auth',
        token: localStorage.getItem('authToken')
    }));
};

socket.onmessage = function(event) {
    const data = JSON.parse(event.data);
    console.log('Received:', data);
    
    // Handle different message types
    switch(data.type) {
        case 'notification':
            showNotification(data.payload);
            break;
        case 'chat':
            displayChatMessage(data.payload);
            break;
        case 'analytics':
            updateAnalytics(data.payload);
            break;
    }
};

socket.onclose = function(event) {
    console.log('WebSocket closed:', event.code, event.reason);
    // Attempt to reconnect
    setTimeout(() => {
        // Reconnection logic here
    }, 1000);
};
*/

// SSE connection helper
async fn sse_connection_helper(_user: AuthenticatedUser) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "sse_url": "/api/events",
        "retry_interval": 3000,
        "supported_events": ["notifications", "chat", "analytics"]
    })))
}
```

## Summary

Real-time features in Oxidite provide:

- **WebSocket Support**: Full-duplex communication for interactive applications
- **Server-Sent Events**: Unidirectional server-to-client updates
- **Pub/Sub Messaging**: Event distribution system
- **Live Notifications**: Real-time alert delivery
- **Real-time Analytics**: Live metrics and tracking
- **Performance Optimization**: Connection pooling and rate limiting
- **Security**: Authentication, validation, and rate limiting
- **Frontend Integration**: Easy client-side implementation

These features enable building highly interactive and responsive web applications with real-time updates and bidirectional communication capabilities.