use oxidite::prelude::*;
use oxidite::db::Model;
use serde::{Deserialize, Serialize};

// Define a simple model
#[derive(Debug, Clone, Serialize, Deserialize, Model, sqlx::FromRow)]
#[model(table = "users")]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Oxidite Single File Example ===\n");

    // Load configuration
    let config = Config::from_env()
        .await
        .unwrap_or_else(|_| Config::default());

    println!("Server would run on: {}:{}", config.host, config.port);
    println!("Database URL: {}", config.database_url);
    println!();

    // Example: Create a simple HTTP server
    println!("Starting HTTP server...");
    
    let app = Router::new()
        .route("/", get_root)
        .route("/health", get_health)
        .route("/users", get_users);

    println!("Routes registered:");
    println!("  GET /         -> Root handler");
    println!("  GET /health   -> Health check");
    println!("  GET /users    -> List users\n");

    // Note: In standalone mode, we just demonstrate the setup
    // To actually start the server, uncomment the line below:
    // Server::new(app).bind(([127, 0, 0, 1], 3000)).await?;

    println!("Server setup complete (not binding in demo mode)");
    println!("\n=== Example completed successfully ===");

    Ok(())
}

// Route handlers
async fn get_root(_req: Request) -> Result<Response> {
    let body = serde_json::json!({
        "message": "Welcome to Oxidite!",
        "version": "2.3.2"
    });

    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))?)
}

async fn get_health(_req: Request) -> Result<Response> {
    let body = serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().timestamp()
    });

    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))?)
}

async fn get_users(_req: Request) -> Result<Response> {
    // In a real app, you would fetch from database:
    // let users = User::query().fetch_all(&db).await?;
    
    let users = vec![
        serde_json::json!({ "id": 1, "name": "Alice", "email": "alice@example.com" }),
        serde_json::json!({ "id": 2, "name": "Bob", "email": "bob@example.com" }),
    ];

    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&users).unwrap()))?)
}
