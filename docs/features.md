# Oxidite v2 Features

This document provides a comprehensive overview of all features available in Oxidite v2.

## Core Features

### Request Handling
Oxidite provides flexible request handling with multiple extractor types:

#### JSON Extractor
The `Json<T>` extractor handles JSON request bodies:

```rust
use oxidite::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

async fn create_user(Json(payload): Json<CreateUser>) -> Result<Response> {
    // payload contains deserialized JSON
    Ok(response::json(serde_json::json!(payload)))
}
```

#### Query Parameters Extractor
The `Query<T>` extractor handles URL query parameters:

```rust
use oxidite::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    limit: Option<u32>,
}

async fn list_users(Query(params): Query<Pagination>) -> Result<Response> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(10);
    
    Ok(response::json(serde_json::json!({ "page": page, "limit": limit })))
}
```

#### Path Parameters Extractor
The `Path<T>` extractor handles URL path parameters:

```rust
use oxidite::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct UserId {
    id: u64,
}

async fn get_user(Path(params): Path<UserId>) -> Result<Response> {
    Ok(response::json(serde_json::json!({ "id": params.id })))
}
```

#### Form Data Extractor
The `Form<T>` extractor handles `application/x-www-form-urlencoded` data:

```rust
use oxidite::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

async fn login_handler(Form(login_data): Form<LoginForm>) -> Result<Response> {
    // login_data contains the deserialized form data
    Ok(response::json(serde_json::json!({
        "message": "Login successful",
        "username": login_data.username
    })))
}
```

#### Cookies Extractor
The `Cookies` extractor provides access to request cookies:

```rust
use oxidite::prelude::*;

async fn handler(cookies: Cookies) -> Result<Response> {
    if let Some(session_id) = cookies.get("session_id") {
        // use the session id
        println!("session id: {}", session_id);
    }
    
    // check if cookie exists
    if cookies.contains("remember_me") {
        println!("remember me cookie present");
    }
    
    // iterate through all cookies
    for (name, value) in cookies.iter() {
        println!("{}: {}", name, value);
    }
    
    Ok(response::json(serde_json::json!({ "status": "ok" })))
}
```

#### Raw Body Extractor
The `Body<T>` extractor allows access to raw request body data:

```rust
use oxidite::prelude::*;

// extract as string
async fn webhook_handler(Body(raw_body): Body<String>) -> Result<Response> {
    println!("received webhook with {} characters", raw_body.len());
    Ok(response::text("webhook received"))
}

// extract as bytes
async fn binary_handler(Body(bytes): Body<Vec<u8>>) -> Result<Response> {
    println!("received {} bytes", bytes.len());
    Ok(response::text("binary data received"))
}
```

### State Management
The `State<T>` extractor manages application state:

```rust
use oxidite::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    db_url: String,
}

async fn handler(State(state): State<Arc<AppState>>) -> Result<Response> {
    Ok(response::json(serde_json::json!({ "db_url": state.db_url })))
}
```

## Advanced Features

### API Versioning
Oxidite supports multiple approaches to API versioning to maintain backward compatibility while evolving your API.

#### URL-based Versioning
```rust
use oxidite::prelude::*;

let mut router = Router::new();

// version 1 api
router.get("/api/v1/users", list_users_v1);
router.post("/api/v1/users", create_user_v1);

// version 2 api
router.get("/api/v2/users", list_users_v2);
router.post("/api/v2/users", create_user_v2);
```

#### Header-based Versioning
```rust
use oxidite::prelude::*;

async fn versioned_handler(req: Request) -> Result<Response> {
    let version = req.headers()
        .get("accept")
        .and_then(|hv| hv.to_str().ok())
        .and_then(|s| {
            if s.contains("version=2") {
                Some(2)
            } else {
                Some(1) // default version
            }
        })
        .unwrap_or(1);
    
    match version {
        1 => handle_v1_api(req).await,
        2 => handle_v2_api(req).await,
        _ => Err(Error::BadRequest("unsupported api version".to_string())),
    }
}
```

#### Query Parameter Versioning
```rust
use oxidite::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct ApiVersionQuery {
    version: Option<u32>,
}

async fn query_versioned_handler(
    req: Request,
    Query(params): Query<ApiVersionQuery>
) -> Result<Response> {
    let version = params.version.unwrap_or(1);
    
    match version {
        1 => handle_v1_api(req).await,
        2 => handle_v2_api(req).await,
        _ => Err(Error::BadRequest("unsupported api version".to_string())),
    }
}
```

### Background Jobs
Oxidite includes a robust background job system for processing tasks asynchronously.

#### Job Definition
```rust
use oxidite::queue::{Job, Queue};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct EmailJob {
    recipient: String,
    subject: String,
    body: String,
}

impl Job for EmailJob {
    async fn execute(self) -> Result<(), Box<dyn std::error::Error>> {
        // send email logic here
        send_email(&self.recipient, &self.subject, &self.body).await?;
        println!("email sent to: {}", self.recipient);
        Ok(())
    }
}
```

#### Queue Usage
```rust
use oxidite::queue::{Queue, Job};

async fn example_job_queue() -> Result<(), Box<dyn std::error::Error>> {
    // create a queue
    let queue = Queue::new_memory(); // or Queue::new_redis("redis://127.0.0.1:6379").await?;
    
    // create a job
    let email_job = EmailJob {
        recipient: "user@example.com".to_string(),
        subject: "welcome!".to_string(),
        body: "thank you for joining us.".to_string(),
    };
    
    // enqueue the job
    queue.enqueue(email_job).await?;
    
    // process jobs
    queue.process().await?;
    
    Ok(())
}
```

#### Scheduled Jobs (Cron)
```rust
use oxidite::queue::{Job, Queue};
use cron::Schedule;
use chrono::Utc;

#[derive(Serialize, Deserialize)]
struct DailyReportJob;

impl Job for DailyReportJob {
    async fn execute(self) -> Result<(), Box<dyn std::error::Error>> {
        // generate daily report
        generate_daily_report().await?;
        Ok(())
    }
}

async fn schedule_cron_jobs() -> Result<(), Box<dyn std::error::Error>> {
    let queue = Queue::new_memory();
    
    // schedule a job to run daily at midnight
    let schedule = Schedule::from_str("0 0 * * * *")?; // every day at 00:00
    
    // add to scheduler (conceptual - actual implementation may vary)
    // scheduler.schedule(schedule, DailyReportJob).await?;
    
    Ok(())
}
```

### Real-time Features
Oxidite provides real-time communication capabilities through WebSockets and Server-Sent Events (SSE).

#### WebSocket Support
```rust
use oxidite::prelude::*;
use oxidite::realtime::{WebSocketManager, WebSocketConnection};

async fn websocket_handler(
    mut req: Request,
    ws_manager: &WebSocketManager
) -> Result<Response> {
    // upgrade to websocket connection
    if let Some(upgrade) = req.extensions().get::<hyper::upgrade::OnUpgrade>() {
        tokio::spawn(async move {
            if let Ok(upgraded) = upgrade.await {
                let mut ws_conn = WebSocketConnection::new(upgraded);
                
                // handle websocket messages
                while let Some(msg) = ws_conn.recv().await {
                    match msg {
                        Ok(text) => {
                            // echo the message back
                            ws_conn.send(&format!("echo: {}", text)).await.unwrap();
                            
                            // broadcast to all connected clients
                            ws_manager.broadcast(&text).await;
                        }
                        Err(e) => {
                            println!("websocket error: {}", e);
                            break;
                        }
                    }
                }
            }
        });
        
        Ok(response::text("websocket upgrade initiated"))
    } else {
        Err(Error::BadRequest("expected websocket upgrade".to_string()))
    }
}
```

#### Server-Sent Events (SSE)
```rust
use oxidite::prelude::*;
use futures::stream::StreamExt;

async fn sse_handler(_req: Request) -> Result<Response> {
    use futures::stream::iter;
    use tokio_stream::Stream;
    
    let stream = tokio_stream::iter(vec![
        Ok::<_, hyper::Error>(format!("data: {}\n\n", "connected")),
        Ok::<_, hyper::Error>(format!("data: {}\n\n", "message 1")),
        Ok::<_, hyper::Error>(format!("data: {}\n\n", "message 2")),
    ]);
    
    let body = http_body_util::BodyStream::new(stream.map(|item| {
        Ok::<_, hyper::Error>(hyper::body::Frame::data(item.unwrap().into_bytes()))
    }));
    
    let mut response = hyper::Response::builder()
        .header("content-type", "text/event-stream")
        .header("cache-control", "no-cache")
        .header("connection", "keep-alive")
        .body(body.boxed())
        .map_err(|e| Error::Server(e.to_string()))?;
    
    Ok(response)
}
```

## Error Handling

### Enhanced Error Types
The error system has been expanded with more specific error types and better HTTP status code mapping:

#### New Error Types
- `Error::Forbidden(String)` - 403 Forbidden
- `Error::Conflict(String)` - 409 Conflict  
- `Error::Validation(String)` - 422 Unprocessable Entity
- `Error::RateLimited` - 429 Too Many Requests
- `Error::ServiceUnavailable(String)` - 503 Service Unavailable

#### HTTP Status Code Mapping
Each error type now maps to the appropriate HTTP status code:

```rust
use oxidite::prelude::*;

impl Error {
    pub fn status_code(&self) -> hyper::StatusCode {
        match self {
            Error::NotFound => hyper::StatusCode::NOT_FOUND,
            Error::BadRequest(_) => hyper::StatusCode::BAD_REQUEST,
            Error::Unauthorized(_) => hyper::StatusCode::UNAUTHORIZED,
            Error::Forbidden(_) => hyper::StatusCode::FORBIDDEN,
            Error::Conflict(_) => hyper::StatusCode::CONFLICT,
            Error::Validation(_) => hyper::StatusCode::UNPROCESSABLE_ENTITY,
            Error::RateLimited => hyper::StatusCode::TOO_MANY_REQUESTS,
            Error::ServiceUnavailable(_) => hyper::StatusCode::SERVICE_UNAVAILABLE,
            Error::Server(_) | Error::Hyper(_) | Error::Io(_) | Error::SerdeJson(_) | 
            Error::SerdeUrlEncoded(_) | Error::Http(_) => hyper::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
```

#### Usage Example
```rust
use oxidite::prelude::*;

async fn create_user(Json(payload): Json<CreateUserRequest>) -> Result<OxiditeResponse> {
    // validate input
    if payload.name.trim().is_empty() {
        return Err(Error::Validation("name cannot be empty".to_string()));
    }
    
    // check for duplicates
    if user_exists(&payload.email) {
        return Err(Error::Conflict("email already exists".to_string()));
    }
    
    // create user...
    Ok(response::json(serde_json::json!(user)))
}
```

## Enterprise Features

### Security & Compliance
Oxidite provides enterprise-grade security features:

#### Enterprise Security Features
- Advanced authentication mechanisms (JWT, OAuth2, API Keys)
- Multi-factor authentication (MFA)
- Role-based access control (RBAC)
- Rate limiting with sliding window
- CSRF protection
- XSS sanitization
- Password hashing
- Two-factor authentication

#### Compliance Features
- Audit logging for compliance requirements
- Data retention policies
- GDPR compliance tools

### High Availability & Scalability
#### Clustering
- Multi-node cluster support
- Automatic failover mechanisms
- Load distribution across nodes
- Cluster-wide configuration management

#### Performance Optimization
- Horizontal scaling capabilities
- Auto-scaling based on demand
- Caching layers with invalidation
- Database read/write splitting
- CDN integration

### API Management
#### API Gateway Features
- Rate limiting and throttling
- API versioning strategies
- Request/response transformation
- Security enforcement
- Traffic analytics

#### Developer Portal
- Interactive API documentation
- SDK generation for multiple languages
- API testing tools
- Usage analytics for developers

### Integration Capabilities
#### Enterprise Integrations
- Message queue systems (PostgreSQL, Redis)
- Third-party service integrations
- Custom adapter framework
- Data synchronization tools

#### Data Management
- ETL pipeline capabilities
- Real-time data streaming
- Batch processing support
- Data quality checks
- Master data management

### Advanced Security
#### Security Automation
- Automated vulnerability scanning
- Security policy enforcement
- Threat detection and mitigation
- Intrusion prevention systems
- Security event correlation

#### Identity & Access Management
- Identity federation
- Privileged access management
- Identity governance
- Access certification
- Segregation of duties

### Deployment & DevOps
#### CI/CD Integration
- Automated testing pipelines
- Blue-green deployment strategies
- Canary release capabilities
- Infrastructure as code support
- Deployment automation

#### Container Orchestration
- Kubernetes native support
- Docker compatibility
- Service mesh integration
- Auto-healing capabilities
- Resource optimization