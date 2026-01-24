mod db;
mod auth;
mod queue;
mod realtime;

use oxidite::prelude::*;


#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("Starting Comprehensive Test App...");
    
    // Initialize components
    let db = db::init_db().await?;
    let auth = auth::init_auth().await?;
    let queue = queue::init_queue().await?;
    let realtime = realtime::init_realtime().await?;
    
    // Create router
    let mut router = Router::new();
    
    // Register routes
    auth::auth_routes(&mut router);
    queue::queue_routes(&mut router);
    realtime::realtime_routes(&mut router);
    
    // Basic health check
    router.get("/health", |_: Request| async {
        Ok(Response::json(serde_json::json!({ "status": "ok" })))
    });
    
    // Start server
    let addr: std::net::SocketAddr = "127.0.0.1:3000".parse().unwrap();
    println!("Listening on http://{}", addr);
    
    Server::new(router).listen(addr).await
}
