use oxidite::prelude::*;
use oxidite_realtime::WebSocketManager;
use std::sync::Arc;

pub async fn init_realtime() -> Result<Arc<WebSocketManager>> {
    println!("Initializing realtime...");
    
    let realtime = WebSocketManager::new();
    
    println!("Realtime initialized.");
    Ok(Arc::new(realtime))
}

pub fn realtime_routes(router: &mut Router) {
    router.get("/ws", |_req: Request| async {
        // In a real app, upgrade to WebSocket
        Ok(Response::text("WebSocket endpoint"))
    });
}
