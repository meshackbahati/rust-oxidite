// Example: Full-featured API with Oxidite
// Demonstrates routing, extractors, middleware, and more

use oxidite_core::{Router, Server, OxiditeRequest, OxiditeResponse, Result, Path, Query, Json};
use oxidite_middleware::{ServiceBuilder, LoggerLayer};
use serde::{Deserialize, Serialize};
use http_body_util::Full;
use bytes::Bytes;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
}

#[derive(Debug, Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

#[derive(Debug, Deserialize)]
struct ListQuery {
    page: Option<u32>,
    limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct UserId {
    id: u64,
}

// GET / - Hello world
async fn index(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    Ok(hyper::Response::new(Full::new(Bytes::from("Welcome to Oxidite API!"))))
}

// GET /users - List users with pagination
async fn list_users(req: OxiditeRequest) -> Result<OxiditeResponse> {
    // In a real app, this would query the database
    let users = vec![
        User {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        },
        User {
            id: 2,
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
        },
    ];

    let body = serde_json::to_vec(&users)
        .map_err(|e| oxidite_core::Error::Server(format!("Serialization error: {}", e)))?;

    Ok(hyper::Response::builder()
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(body)))
        .unwrap())
}

// GET /users/:id - Get single user
async fn get_user(req: OxiditeRequest) -> Result<OxiditeResponse> {
    // Extract path parameter
    // In a real app, use Path extractor after router enhancement
    
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    let body = serde_json::to_vec(&user)
        .map_err(|e| oxidite_core::Error::Server(format!("Serialization error: {}", e)))?;

    Ok(hyper::Response::builder()
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(body)))
        .unwrap())
}

// POST /users - Create user
async fn create_user(req: OxiditeRequest) -> Result<OxiditeResponse> {
    // In a real app, extract JSON body and create in database
    
    let user = User {
        id: 3,
        name: "Charlie".to_string(),
        email: "charlie@example.com".to_string(),
    };

    let body = serde_json::to_vec(&user)
        .map_err(|e| oxidite_core::Error::Server(format!("Serialization error: {}", e)))?;

    Ok(hyper::Response::builder()
        .status(201)
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(body)))
        .unwrap())
}

// GET /health - Health check
async fn health(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    let health = serde_json::json!({
        "status": "ok",
        "uptime": 12345,
        "version": env!("CARGO_PKG_VERSION"),
    });

    let body = serde_json::to_vec(&health)
        .map_err(|e| oxidite_core::Error::Server(format!("Serialization error: {}", e)))?;

    Ok(hyper::Response::builder()
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(body)))
        .unwrap())
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting Oxidite Example API");
    println!("📍 Listening on http://127.0.0.1:3000");
    println!("📝 Available endpoints:");
    println!("   GET  /         - Welcome message");
    println!("   GET  /users    - List all users");
    println!("   GET  /users/1  - Get user by ID");
    println!("   POST /users    - Create new user");
    println!("   GET  /health   - Health check");
    println!();

    let mut router = Router::new();
    
    // Register routes
    router.get("/", index);
    router.get("/users", list_users);
    router.get("/users/1", get_user);
    router.post("/users", create_user);
    router.get("/health", health);

    // Compose middleware stack
    let service = ServiceBuilder::new()
        .layer(LoggerLayer)
        .service(router);

    // Start server
    let server = Server::new(service);
    server.listen("127.0.0.1:3000".parse().unwrap()).await?;

    Ok(())
}
