// Simple WebSocket chat example
// Run with: cargo run --example websocket_chat

use oxidite_websocket::{Message, WebSocketConnection, WebSocketManager, Room};
use std::sync::Arc;
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() {
    println!("🚀 WebSocket Chat Example");
    println!("==========================\n");

    // Create WebSocket manager
    let manager = Arc::new(WebSocketManager::new());
    let room_manager = manager.room_manager();

    // Create a chat room
    room_manager.create_room("general".to_string()).await.unwrap();
    println!("✅ Created room: general\n");

    // Simulate user connections
    let (user1, mut rx1) = WebSocketConnection::new(Some("Alice".to_string()));
    let (user2, mut rx2) = WebSocketConnection::new(Some("Bob".to_string()));
    let (user3, mut rx3) = WebSocketConnection::new(Some("Charlie".to_string()));

    let user1 = Arc::new(user1);
    let user2 = Arc::new(user2);
    let user3 = Arc::new(user3);

    // Add connections to manager
    manager.add_connection(user1.clone()).await;
    manager.add_connection(user2.clone()).await;
    manager.add_connection(user3.clone()).await;

    println!("👤 Alice connected (ID: {})", user1.id);
    println!("👤 Bob connected (ID: {})", user2.id);
    println!("👤 Charlie connected (ID: {})\n", user3.id);

    // Join room
    room_manager.join_room("general", user1.id.clone()).await.unwrap();
    room_manager.join_room("general", user2.id.clone()).await.unwrap();
    room_manager.join_room("general", user3.id.clone()).await.unwrap();

    println!("✅ All users joined 'general' room\n");

    // Spawn message receivers
    let user1_id = user1.user_id.clone().unwrap();
    tokio::spawn(async move {
        while let Ok(msg) = rx1.recv().await {
            if let Message::Text { content } = msg {
                println!("  [Alice received]: {}", content);
            }
        }
    });

    let user2_id = user2.user_id.clone().unwrap();
    tokio::spawn(async move {
        while let Ok(msg) = rx2.recv().await {
            if let Message::Text { content } = msg {
                println!("  [Bob received]: {}", content);
            }
        }
    });

    let user3_id = user3.user_id.clone().unwrap();
    tokio::spawn(async move {
        while let Ok(msg) = rx3.recv().await {
            if let Message::Text { content } = msg {
                println!("  [Charlie received]: {}", content);
            }
        }
    });

    sleep(Duration::from_millis(100)).await;

    // Send messages
    println!("💬 Alice sends: Hello everyone!");
    manager.broadcast(Message::text("Alice: Hello everyone!")).await.unwrap();
    sleep(Duration::from_millis(100)).await;

    println!("\n💬 Bob sends: Hi Alice!");
    manager.broadcast(Message::text("Bob: Hi Alice!")).await.unwrap();
    sleep(Duration::from_millis(100)).await;

    println!("\n💬 Charlie sends: Hey team!");
    manager.broadcast(Message::text("Charlie: Hey team!")).await.unwrap();
    sleep(Duration::from_millis(100)).await;

    // Show room members
    let members = room_manager.get_room_members("general").await.unwrap();
    println!("\n👥 Room 'general' has {} members", members.len());

    // Cleanup
    sleep(Duration::from_millis(200)).await;
    println!("\n✅ Chat example completed!");
}
