// Comprehensive example showcasing all Oxidite features
// This demonstrates: routing, middleware, auth, caching, queues, and database

use oxidite_core::{Router, Server, OxiditeRequest, OxiditeResponse, Result as CoreResult, Path, Json};
use oxidite_middleware::{ServiceBuilder, LoggerLayer};
use oxidite_config::Config;
use oxidite_auth::{hash_password, verify_password, create_token, JwtToken, Claims};
use oxidite_cache::{Cache, MemoryCache};
use oxidite_queue::{Queue, Job, JobWrapper, Worker};
use serde::{Deserialize, Serialize};
use http_body_util::Full;
use bytes::Bytes;
use std::sync::Arc;
use std::time::Duration;

// ============================================================================
// Data Models
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
struct User {
    id: u64,
    name: String,
    email: String,
    password_hash: String,
}

#[derive(Debug, Deserialize)]
struct RegisterRequest {
    name: String,
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct AuthResponse {
    token: String,
    user: UserResponse,
}

#[derive(Debug, Serialize, Clone)]
struct UserResponse {
    id: u64,
    name: String,
    email: String,
}

// ============================================================================
// Background Jobs
// ============================================================================

#[derive(Serialize, Deserialize)]
struct SendEmailJob {
    to: String,
    subject: String,
    body: String,
}

#[async_trait::async_trait]
impl Job for SendEmailJob {
    async fn perform(&self) -> oxidite_queue::Result<()> {
        println!("📧 Sending email to: {}", self.to);
        println!("   Subject: {}", self.subject);
        
        // Simulate sending email
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        println!("✅ Email sent successfully");
        Ok(())
    }

    fn name(&self) -> &'static str {
        "SendEmailJob"
    }
}

// ============================================================================
// API Handlers
// ============================================================================

async fn index(_req: OxiditeRequest) -> CoreResult<OxiditeResponse> {
    let response = serde_json::json!({
        "message": "Welcome to Oxidite Comprehensive Example!",
        "version": env!("CARGO_PKG_VERSION"),
        "endpoints": {
            "health": "GET /health",
            "users": {
                "register": "POST /auth/register",
                "login": "POST /auth/login",
                "profile": "GET /users/:id"
            },
            "cache": {
                "demo": "GET /cache/demo"
            }
        }
    });

    let json = serde_json::to_vec(&response)
        .map_err(|e| oxidite_core::Error::Server(e.to_string()))?;

    Ok(hyper::Response::builder()
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(json)))
        .unwrap())
}

async fn health(_req: OxiditeRequest) -> CoreResult<OxiditeResponse> {
    let health = serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "services": {
            "database": "ok",
            "cache": "ok",
            "queue": "ok"
        }
    });

    let json = serde_json::to_vec(&health)
        .map_err(|e| oxidite_core::Error::Server(e.to_string()))?;

    Ok(hyper::Response::builder()
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(json)))
        .unwrap())
}

async fn register(_req: OxiditeRequest) -> CoreResult<OxiditeResponse> {
    // In real app, parse JSON body
    let user_data = RegisterRequest {
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
        password: "securepassword123".to_string(),
    };

    // Hash password
    let password_hash = hash_password(&user_data.password)
        .map_err(|e| oxidite_core::Error::Server(e.to_string()))?;

    let user = User {
        id: 1,
        name: user_data.name.clone(),
        email: user_data.email.clone(),
        password_hash,
    };

    // Create JWT token
    let token = create_token(user.id.to_string(), "secret_key", 3600)
        .map_err(|e| oxidite_core::Error::Server(e.to_string()))?;

    let response = AuthResponse {
        token,
        user: UserResponse {
            id: user.id,
            name: user.name,
            email: user.email,
        },
    };

    let json = serde_json::to_vec(&response)
        .map_err(|e| oxidite_core::Error::Server(e.to_string()))?;

    Ok(hyper::Response::builder()
        .status(201)
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(json)))
        .unwrap())
}

async fn cache_demo(_req: OxiditeRequest) -> CoreResult<OxiditeResponse> {
    let cache = MemoryCache::new();

    // Demonstrate caching
    let data = cache.remember("expensive_operation", Duration::from_secs(60), || async {
        println!("🔄 Computing expensive operation...");
        tokio::time::sleep(Duration::from_secs(1)).await;
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>("Expensive result".to_string())
    }).await.map_err(|e| oxidite_core::Error::Server(e.to_string()))?;

    let response = serde_json::json!({
        "cached_value": data,
        "message": "This was cached for 60 seconds"
    });

    let json = serde_json::to_vec(&response)
        .map_err(|e| oxidite_core::Error::Server(e.to_string()))?;

    Ok(hyper::Response::builder()
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(json)))
        .unwrap())
}

// ============================================================================
// Main Application
// ============================================================================

#[tokio::main]
async fn main() -> CoreResult<()> {
    println!("🚀 Oxidite Comprehensive Example");
    println!("==================================");
    println!();

    // Load configuration
    let config = Config::load().unwrap_or_default();
    println!("✅ Configuration loaded");
    println!("   Server: {}:{}", config.server.host, config.server.port);
    println!("   Environment: {}", config.app.environment);
    println!();

    // Initialize cache
    let cache = Arc::new(MemoryCache::new());
    println!("✅ Cache initialized");

    // Initialize queue
    let queue = Arc::new(Queue::memory());
    println!("✅ Queue initialized");

    // Enqueue a sample job
    let email_job = SendEmailJob {
        to: "user@example.com".to_string(),
        subject: "Welcome to Oxidite!".to_string(),
        body: "Thank you for using Oxidite framework.".to_string(),
    };
    let job_wrapper = JobWrapper::new(&email_job)
        .map_err(|e| oxidite_core::Error::Server(e.to_string()))?;
    queue.enqueue(job_wrapper).await
        .map_err(|e| oxidite_core::Error::Server(e.to_string()))?;
    println!("✅ Sample job enqueued");
    println!();

    // Start queue worker in background
    let queue_clone = queue.clone();
    tokio::spawn(async move {
        Worker::new(queue_clone)
            .worker_count(2)
            .start()
            .await;
    });

    // Setup router
    let mut router = Router::new();
    
    router.get("/", index);
    router.get("/health", health);
    router.post("/auth/register", register);
    router.get("/cache/demo", cache_demo);

    // Build middleware stack
    let service = ServiceBuilder::new()
        .layer(LoggerLayer)
        .service(router);

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .unwrap();

    println!("📍 Server starting on http://{}", addr);
    println!();
    println!("Available endpoints:");
    println!("  GET  /            - API overview");
    println!("  GET  /health      - Health check");
    println!("  POST /auth/register - Register user");
    println!("  GET  /cache/demo  - Cache demonstration");
    println!();
    println!("Example requests:");
    println!("  curl http://{}/", addr);
    println!("  curl http://{}/health", addr);
    println!("  curl -X POST http://{}/auth/register", addr);
    println!("  curl http://{}/cache/demo", addr);
    println!();

    let server = Server::new(service);
    server.listen(addr).await
}
