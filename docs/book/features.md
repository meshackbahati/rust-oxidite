# Features

This chapter consolidates all the features of the Oxidite framework into a single comprehensive overview, as requested in the documentation consolidation rule.

## Core Features

### 1. High-Performance Web Server
- Built on top of Hyper and Tokio for async/await support
- Supports HTTP/1.1, HTTP/2, and HTTP/3 protocols
- Zero-copy transfers for optimal performance
- Concurrent request handling with async runtime

```rust
use oxidite::prelude::*;

async fn hello_world(_req: Request) -> Result<Response> {
    Ok(Response::text("Hello, World!"))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    router.get("/", hello_world);
    
    Server::new(router)
        .listen("127.0.0.1:3000".parse()?)
        .await
}
```

### 2. Type-Safe Request Handling
- Strongly typed request extractors
- Compile-time validation of route parameters
- Automatic serialization/deserialization with Serde
- Error handling with custom error types

```rust
use oxidite::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct QueryParams {
    page: Option<u32>,
    limit: Option<u32>,
}

async fn api_handler(
    Path(user_id): Path<u32>,
    Query(params): Query<QueryParams>,
    Json(payload): Json<serde_json::Value>
) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "user_id": user_id,
        "query_params": params,
        "payload": payload
    })))
}
```

### 3. Comprehensive Response System
- Multiple response types (JSON, HTML, text, etc.)
- Consistent API with `Response::method()` pattern
- Template engine integration
- Proper HTTP status codes

```rust
use oxidite::prelude::*;

async fn various_responses(_req: Request) -> Result<Response> {
    // JSON response
    let json_resp = Response::json(serde_json::json!({ "type": "json" }));
    
    // HTML response
    let html_resp = Response::html("<h1>Hello HTML</h1>");
    
    // Text response
    let text_resp = Response::text("Plain text");
    
    // Empty responses
    let ok_resp = Response::ok();
    let no_content_resp = Response::no_content();
    
    // Return one of them
    Ok(json_resp)
}
```

## Advanced Features

### 4. Database ORM
- Model definitions with derive macros
- Type-safe database operations
- Relationship management
- Migrations and schema management
- Validation and hooks

```rust
use oxidite::prelude::*;
use serde::Deserialize;

#[derive(Model, Deserialize)]
#[model(table = "users")]
pub struct User {
    #[model(primary_key)]
    pub id: i32,
    #[model(unique, not_null)]
    pub email: String,
    #[model(not_null)]
    pub name: String,
    #[model(default = "now")]
    pub created_at: String,
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

async fn user_operations() -> Result<()> {
    // Create
    let user = User {
        id: 0,
        email: "john@example.com".to_string(),
        name: "John Doe".to_string(),
        created_at: now(),
    };
    let saved_user = user.save().await?;
    
    // Read
    let users = User::find_all().await?;
    
    // Update
    let mut user = saved_user;
    user.name = "John Updated".to_string();
    user.save().await?;
    
    // Delete
    user.delete().await?;
    
    Ok(())
}
```

### 5. Authentication & Authorization
- JWT token support
- Session-based authentication
- API key authentication
- OAuth2 integration
- Role-based access control (RBAC)
- Two-factor authentication (2FA)

```rust
use oxidite::prelude::*;

// JWT authentication middleware
async fn jwt_auth_middleware(req: Request, next: Next) -> Result<Response> {
    // Extract and validate JWT token
    let auth_header = req.headers()
        .get("authorization")
        .and_then(|hv| hv.to_str().ok());
    
    match auth_header {
        Some(auth) if auth.starts_with("Bearer ") => {
            let token = auth.trim_start_matches("Bearer ").trim();
            if verify_jwt(token).await.is_ok() {
                next.run(req).await
            } else {
                Err(Error::Unauthorized("Invalid token".to_string()))
            }
        }
        _ => Err(Error::Unauthorized("Missing token".to_string())),
    }
}

async fn verify_jwt(_token: &str) -> Result<()> {
    // Implementation would verify the JWT
    Ok(())
}
```

### 6. Middleware System
- Global and route-specific middleware
- Request/response modification
- Cross-cutting concerns
- Built-in middleware for common tasks

```rust
use oxidite::prelude::*;

async fn logging_middleware(req: Request, next: Next) -> Result<Response> {
    println!("Request: {} {}", req.method(), req.uri());
    let response = next.run(req).await?;
    println!("Response: {}", response.status());
    Ok(response)
}

async fn cors_middleware(req: Request, next: Next) -> Result<Response> {
    let mut response = if req.method() == http::Method::OPTIONS {
        Response::ok()
    } else {
        next.run(req).await?
    };
    
    // Add CORS headers
    use http::header::{HeaderName, HeaderValue};
    response.headers_mut().insert(
        HeaderName::from_static("access-control-allow-origin"),
        HeaderValue::from_static("*")
    );
    
    Ok(response)
}
```

### 7. Template Engine
- Server-side template rendering
- Template inheritance and composition
- Context variable binding
- Direct integration with Response system

```rust
use oxidite::prelude::*;
use oxidite_template::{TemplateEngine, Context};

async fn template_example(_req: Request) -> Result<Response> {
    let mut template_engine = TemplateEngine::new();
    
    // Add a template
    template_engine.add_template("welcome", r#"
        <html>
        <head><title>{{ title }}</title></head>
        <body>
            <h1>{{ greeting }}</h1>
            <p>Welcome, {{ name }}!</p>
            <ul>
            {% for item in items %}
                <li>{{ item }}</li>
            {% endfor %}
            </ul>
        </body>
        </html>
    "#)?;
    
    // Create context
    let mut context = Context::new();
    context.set("title", "Welcome Page");
    context.set("greeting", "Hello!");
    context.set("name", "User");
    context.set("items", vec!["Feature 1", "Feature 2", "Feature 3"]);
    
    // Render as response
    let response = template_engine.render_response("welcome", &context)
        .map_err(|e| Error::Server(e.to_string()))?;
    
    Ok(response)
}
```

### 8. Background Jobs & Queues
- Asynchronous job processing
- Multiple backend support (Redis, PostgreSQL, memory)
- Job scheduling and retries
- Worker management

```rust
use oxidite_queue::{Job, Queue, Worker};

// Define a job
#[derive(serde::Serialize, serde::Deserialize)]
struct EmailJob {
    recipient: String,
    subject: String,
    body: String,
}

impl Job for EmailJob {
    type Output = Result<(), String>;
    
    async fn execute(self) -> Self::Output {
        // Send email logic here
        println!("Sending email to {} with subject: {}", 
                 self.recipient, self.subject);
        Ok(())
    }
}

async fn queue_example() -> Result<()> {
    // Create a queue
    let queue = Queue::new("default");
    
    // Enqueue a job
    let email_job = EmailJob {
        recipient: "user@example.com".to_string(),
        subject: "Welcome!".to_string(),
        body: "Thank you for joining.".to_string(),
    };
    
    queue.enqueue(email_job).await?;
    
    // Start a worker
    let worker = Worker::new(queue.clone());
    worker.start().await?;
    
    Ok(())
}
```

### 9. Real-time Features
- WebSocket support
- Server-Sent Events (SSE)
- Pub/Sub messaging
- Live updates and notifications

```rust
use oxidite::prelude::*;
use oxidite_realtime::websocket::{WebSocket, Message};

async fn websocket_handler(ws: WebSocket) -> Result<()> {
    ws.on_message(|msg| async move {
        match msg {
            Message::Text(text) => {
                println!("Received: {}", text);
                // Echo back
                Ok(Message::Text(format!("Echo: {}", text)))
            }
            Message::Binary(data) => {
                println!("Received binary: {} bytes", data.len());
                Ok(Message::Binary(data))
            }
        }
    }).await?;
    
    Ok(())
}

async fn sse_example(_req: Request) -> Result<Response> {
    use oxidite_realtime::sse::EventStream;
    
    let mut stream = EventStream::new();
    stream.send("Connected", Some("connection"), None).await?;
    
    // Send periodic updates
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            stream.send("Update", Some("data"), None).await.ok();
        }
    });
    
    Ok(stream.response())
}
```

### 10. File Upload & Storage
- Multipart form handling
- File validation and sanitization
- Multiple storage backends (local, S3, etc.)
- Streaming uploads for large files

```rust
use oxidite::prelude::*;

async fn upload_handler(_req: Request) -> Result<Response> {
    // In a real implementation, handle multipart form data
    // and save files to configured storage backend
    
    Ok(Response::json(serde_json::json!({
        "status": "uploaded",
        "files": []
    })))
}
```

### 11. Security Features
- Rate limiting
- CSRF protection
- XSS prevention
- SQL injection prevention
- Input validation
- Secure headers

```rust
use oxidite::prelude::*;

async fn security_middleware(req: Request, next: Next) -> Result<Response> {
    // Rate limiting
    if !is_request_allowed(&req).await {
        return Err(Error::RateLimited);
    }
    
    // Add security headers
    let mut response = next.run(req).await?;
    
    use http::header::{HeaderName, HeaderValue};
    response.headers_mut().insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff")
    );
    
    response.headers_mut().insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY")
    );
    
    Ok(response)
}

async fn is_request_allowed(_req: &Request) -> bool {
    // Implementation would check rate limits
    true
}
```

## Enterprise Features

### 12. Configuration Management
- Environment-based configuration
- Multiple configuration sources
- Type-safe configuration loading
- Hot reloading support

```rust
use oxidite_config::Config;

#[derive(serde::Deserialize)]
struct AppConfig {
    database_url: String,
    server_port: u16,
    jwt_secret: String,
    #[serde(default)]
    debug: bool,
}

async fn load_config() -> Result<AppConfig> {
    let config = Config::builder()
        .add_source(ConfigSource::Env)
        .add_source(ConfigSource::File("config.json"))
        .build()
        .await?;
    
    let app_config: AppConfig = config.try_deserialize()?;
    Ok(app_config)
}

enum ConfigSource {
    Env,
    File(String),
}
```

### 13. CLI Tools
- Project scaffolding
- Code generation
- Database migrations
- Development server with hot reload

```bash
# Create a new project
oxidite new my-app

# Generate a model
oxidite generate model User email:string name:string

# Run migrations
oxidite migrate

# Start development server
oxidite dev
```

### 14. Testing Utilities
- Built-in test utilities
- Mock request/response objects
- Test server for integration tests
- Fixture management

```rust
use oxidite::prelude::*;
use oxidite_testing::{TestServer, RequestBuilder};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_hello_world() {
        let server = TestServer::new(|router| {
            router.get("/", hello_world);
        }).await;
        
        let response = server.get("/").send().await;
        assert_eq!(response.status(), 200);
        
        let body = response.text().await;
        assert_eq!(body, "Hello, World!");
    }
}
```

### 15. OpenAPI Integration
- Automatic API documentation generation
- Schema inference from types
- Interactive API explorer
- Validation against OpenAPI spec

```rust
use oxidite::prelude::*;
use oxidite_openapi::OpenApi;

#[derive(oxidite_macros::RouteInfo)]
#[openapi(path = "/users/{id}", method = "GET")]
async fn get_user(Path(id): Path<u32>) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "id": id,
        "name": format!("User {}", id)
    })))
}

async fn setup_openapi_docs() -> Result<()> {
    let mut openapi = OpenApi::new();
    openapi.add_route(get_user).await?;
    
    // Serve documentation at /docs
    // Implementation would serve the OpenAPI JSON and UI
    
    Ok(())
}
```

### 16. Plugin System
- Extensible architecture
- Hooks and lifecycle events
- Third-party integrations
- Custom middleware and handlers

```rust
use oxidite::prelude::*;

trait Plugin {
    fn name(&self) -> &str;
    fn initialize(&self, _router: &mut Router) -> Result<()>;
    fn on_request(&self, _req: &mut Request) -> Result<()>;
    fn on_response(&self, _resp: &mut Response) -> Result<()>;
}

struct LoggingPlugin;

impl Plugin for LoggingPlugin {
    fn name(&self) -> &str { "logging" }
    
    fn initialize(&self, _router: &mut Router) -> Result<()> {
        println!("Logging plugin initialized");
        Ok(())
    }
    
    fn on_request(&self, req: &mut Request) -> Result<()> {
        println!("Processing request: {} {}", req.method(), req.uri());
        Ok(())
    }
    
    fn on_response(&self, resp: &mut Response) -> Result<()> {
        println!("Sending response: {}", resp.status());
        Ok(())
    }
}
```

## Summary

Oxidite is a comprehensive web framework that combines:

- **Performance**: Built on async/await with Hyper and Tokio
- **Safety**: Type-safe request handling with compile-time validation
- **Flexibility**: Extensible architecture with middleware and plugins
- **Security**: Built-in security features and best practices
- **Productivity**: Rich ecosystem with ORM, authentication, etc.
- **Scalability**: Designed for high-concurrency applications

The framework provides everything needed to build modern web applications, from basic routing to enterprise-level features like authentication, real-time communication, and background jobs.