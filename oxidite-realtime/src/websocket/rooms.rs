use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use super::{Message, Result, WebSocketError};

/// A room/channel for grouping WebSocket connections
pub struct Room {
    pub name: String,
    members: HashSet<String>, // Connection IDs
}

impl Room {
    pub fn new(name: String) -> Self {
        Self {
            name,
            members: HashSet::new(),
        }
    }

    pub fn add_member(&mut self, conn_id: String) {
        self.members.insert(conn_id);
    }

    pub fn remove_member(&mut self, conn_id: &str) {
        self.members.remove(conn_id);
    }

    pub fn members(&self) -> &HashSet<String> {
        &self.members
    }

    pub fn member_count(&self) -> usize {
        self.members.len()
    }
}

/// Room manager for handling multiple rooms
pub struct RoomManager {
    rooms: Arc<RwLock<HashMap<String, Room>>>,
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_room(&self, name: String) -> Result<()> {
        let mut rooms = self.rooms.write().await;
        if !rooms.contains_key(&name) {
            rooms.insert(name.clone(), Room::new(name));
        }
        Ok(())
    }

    pub async fn join_room(&self, room_name: &str, conn_id: String) -> Result<()> {
        let mut rooms = self.rooms.write().await;
        
        // Create room if it doesn't exist
        let room = rooms.entry(room_name.to_string())
            .or_insert_with(|| Room::new(room_name.to_string()));
        
        room.add_member(conn_id);
        Ok(())
    }

    pub async fn leave_room(&self, room_name: &str, conn_id: &str) -> Result<()> {
        let mut rooms = self.rooms.write().await;
        
        if let Some(room) = rooms.get_mut(room_name) {
            room.remove_member(conn_id);
            
            // Remove empty rooms
            if room.member_count() == 0 {
                rooms.remove(room_name);
            }
        }
        
        Ok(())
    }

    pub async fn remove_from_all_rooms(&self, conn_id: &str) {
        let mut rooms = self.rooms.write().await;
        
        // Remove from all rooms
        rooms.retain(|_, room| {
            room.remove_member(conn_id);
            room.member_count() > 0
        });
    }

    pub async fn broadcast_to_room(&self, room_name: &str, message: Message, manager: &super::WebSocketManager) -> Result<()> {
        let rooms = self.rooms.read().await;
        
        if let Some(room) = rooms.get(room_name) {
            for conn_id in room.members() {
                // Send to each member
                let connections = manager.connections.read().await;
                if let Some(conn) = connections.get(conn_id) {
                    let _ = conn.send(message.clone());
                }
            }
            Ok(())
        } else {
            Err(WebSocketError::RoomNotFound)
        }
    }

    pub async fn get_room_members(&self, room_name: &str) -> Result<Vec<String>> {
        let rooms = self.rooms.read().await;
        
        if let Some(room) = rooms.get(room_name) {
            Ok(room.members().iter().cloned().collect())
        } else {
            Err(WebSocketError::RoomNotFound)
        }
    }

    pub async fn list_rooms(&self) -> Vec<String> {
        let rooms = self.rooms.read().await;
        rooms.keys().cloned().collect()
    }

    /// Get number of members in a room.
    pub async fn room_member_count(&self, room_name: &str) -> usize {
        let rooms = self.rooms.read().await;
        rooms.get(room_name).map(|r| r.member_count()).unwrap_or(0)
    }

    /// Get total number of members across all rooms.
    pub async fn total_members(&self) -> usize {
        let rooms = self.rooms.read().await;
        rooms.values().map(Room::member_count).sum()
    }
}

impl Default for RoomManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::RoomManager;

    #[tokio::test]
    async fn room_metrics_track_members() {
        let rooms = RoomManager::new();
        rooms.join_room("r1", "c1".to_string()).await.expect("join");
        rooms.join_room("r1", "c2".to_string()).await.expect("join");
        rooms.join_room("r2", "c3".to_string()).await.expect("join");

        assert_eq!(rooms.room_member_count("r1").await, 2);
        assert_eq!(rooms.room_member_count("missing").await, 0);
        assert_eq!(rooms.total_members().await, 3);
    }
}
