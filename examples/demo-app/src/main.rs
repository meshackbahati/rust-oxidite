//! Oxidite Demo Application
//! 
//! This demo showcases all features of the Oxidite web framework:
//! - RESTful API with versioning
//! - Authentication (JWT & OAuth2)
//! - Database operations with transactions
//! - Template rendering
//! - Real-time WebSockets & SSE
//! - Background job processing
//! - File uploads & storage
//! - Email sending
//! - Caching

use oxidite_core::{Router, Server, Request, Response, Error};
use oxidite_auth::{JwtManager, SessionManager};
use oxidite_db::{DbPool, Database};
use oxidite_middleware::{LoggerLayer, CorsLayer};
use oxidite_template::{TemplateEngine, serve_static};
use oxidite_config::Config;
use std::sync::Arc;

mod routes;
mod models;
mod services;

/// Application state shared across handlers
pub struct AppState {
    db: DbPool,
    jwt: JwtManager,
    sessions: SessionManager,
    templates: TemplateEngine,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Oxidite Demo Application...\n");
    
    // Load configuration
    let config = Config::load()?;
    
    // Initialize database
    println!("🔌 Connecting to database...");
    let db_pool = DbPool::connect("sqlite:demo.db?mode=rwc").await?;
    
    // Run migrations
    println!("🔄 Running database migrations...");
    use std::fs;
    let migration_sql = fs::read_to_string("migrations/001_initial_schema.sql")?;
    for statement in migration_sql.split(';').filter(|s| !s.trim().is_empty()) {
        db_pool.execute(statement.trim()).await?;
    }
    println!("✅ Database initialized");
    
    // Initialize auth
    println!("🔐 Setting up authentication...");
    let jwt = JwtManager::new("demo-secret-key-change-in-production".to_string());
    let sessions = SessionManager::new_memory();
    
    // Initialize templates
    println!("🎨 Loading templates...");
    let mut templates = TemplateEngine::new();
    templates.load_dir("templates")?;
    println!("✅ Templates loaded");
    
    // Create shared state
    let state = Arc::new(AppState {
        db: db_pool,
        jwt,
        sessions,
        templates,
    });
    
    // Setup router with versioning
    println!("🛣️  Configuring routes...");
    let router = setup_router(state.clone());
    
    // Create service with state injection
    let service = oxidite_middleware::tower::ServiceBuilder::new()
        .layer(oxidite_middleware::tower_http::add_extension::AddExtensionLayer::new(state))
        .service(router);
    
    // Start server
    let addr = "127.0.0.1:8080";
    println!("\n✅ Server running on http://{}", addr);
    println!("📖 Visit http://{}/docs for API documentation\n", addr);
    println!("👉 Try V1: curl http://localhost:8080/api/users");
    println!("👉 Try V2: curl http://localhost:8080/api/users -H 'Accept: application/vnd.api+json;version=2'\n");
    
    let server = Server::new(service);
    server.listen(addr.parse()?).await?;
    
    Ok(())
}

fn setup_router(_state: Arc<AppState>) -> Router {
    // use oxidite_core::versioning::{VersionedRouter, ApiVersion};
    
    // V1 Router
    let mut v1_router = Router::new();
    v1_router.get("/api/users", routes::api::list_users);
    v1_router.post("/api/users", routes::api::create_user);
    
    // V2 Router
    let mut v2_router = Router::new();
    v2_router.get("/api/users", routes::api_v2::list_users_v2);
    
    // Main Router (Common routes)
    let mut router = Router::new();
    
    // Static files
    router.get("/public/*", serve_static);
    
    // Web routes (HTML pages)
    router.get("/", routes::index);
    router.get("/favicon.ico", routes::favicon);
    router.get("/docs", routes::api_docs);
    router.get("/api/openapi.json", routes::openapi_spec);
    router.get("/users", routes::web::list_users);
    router.get("/users/new", routes::web::new_user_form);
    router.get("/users/*/posts", routes::user_posts::user_posts);
    router.get("/posts", routes::posts::list_posts);
    router.get("/posts/new", routes::posts::new_post_form);
    router.get("/posts/*", routes::posts::show_post);
    
    // Auth routes
    router.post("/auth/register", routes::auth::register);
    router.post("/auth/login", routes::auth::login);
    router.post("/auth/oauth/google", routes::auth::oauth_google);
    
    // Real-time routes
    router.get("/ws", routes::realtime::websocket_handler);
    router.get("/sse", routes::realtime::sse_handler);
    
    // API routes with versioning
    router.get("/api/v1/users", routes::api::list_users);
    router.post("/api/v1/users", routes::api::create_user);
    router.get("/api/v1/posts", routes::posts::api_list_posts);
    router.post("/api/v1/posts", routes::posts::api_create_post);
    router.get("/api/v2/users", routes::api_v2::list_users_v2);
    
    // 404 handler (catch-all, must be last)
    // Note: In a real implementation, this would be handled by middleware
    // For now, any unmatched routes will return NotFound error
    
    router
}
