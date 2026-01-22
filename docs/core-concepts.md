# Core Concepts

This document explains the fundamental concepts of the Oxidite web framework.

## Architecture Overview

Oxidite follows a modular architecture with the following main components:

- **oxidite-core**: HTTP server, routing, and basic request/response handling
- **oxidite-db**: Database ORM and migrations
- **oxidite-auth**: Authentication and authorization
- **oxidite-middleware**: Middleware components
- **oxidite-template**: Template engine
- **oxidite-cli**: Command-line tools

## Request-Response Lifecycle

The typical lifecycle of a request in Oxidite:

1. **Incoming Request**: HTTP request arrives at the server
2. **Routing**: Router matches the path to a handler function
3. **Middleware Processing**: Request passes through configured middleware layers
4. **Handler Execution**: Handler function processes the request
5. **Response Creation**: Handler returns a response
6. **Middleware Processing**: Response passes back through middleware
7. **Response Sent**: Server sends response back to client

## Router

The Router is responsible for mapping incoming HTTP requests to handler functions.

### Creating a Router

```rust
use oxidite::prelude::*;

let mut router = Router::new();
```

### Adding Routes

```rust
// Different HTTP methods
router.get("/users", list_users);
router.post("/users", create_user);
router.put("/users/:id", update_user);
router.delete("/users/:id", delete_user);
router.patch("/users/:id", partial_update);

// Path parameters (captured and available in handlers)
router.get("/users/:id", get_user);
router.get("/users/:user_id/posts/:post_id", get_post);

// Wildcards (match any path)
router.get("/static/*", serve_static);
```

### Route Matching Priority

Routes are matched in the order they are registered. More specific routes should be registered before general ones:

```rust
// More specific route first
router.get("/users/me", get_current_user);
// More general route after
router.get("/users/:id", get_user);
```

## Handlers

Handlers are async functions that process requests and return responses.

### Handler Signature

```rust
use oxidite::prelude::*;

async fn handler(request: OxiditeRequest) -> Result<OxiditeResponse> {
    // Process request
    Ok(response::text("Hello, World!"))
}
```

### Handler with Extractors

```rust
use oxidite::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct UserId {
    id: u64,
}

async fn get_user(
    Path(params): Path<UserId>
) -> Result<OxiditeResponse> {
    Ok(response::json(serde_json::json!({
        "id": params.id,
        "name": "User Name"
    })))
}
```

## Request and Response Types

### OxiditeRequest

Represents an incoming HTTP request with:

- HTTP method
- URI (path and query)
- Headers
- Body
- Extensions (for storing additional data like path params, state)

### OxiditeResponse

Represents an outgoing HTTP response with:

- Status code
- Headers
- Body

## Response Utilities

Oxidite provides utility functions to create common response types:

```rust
use oxidite::response;

// JSON response
let json_resp = response::json(serde_json::json!({"key": "value"}));

// HTML response
let html_resp = response::html("<h1>Hello</h1>");

// Text response
let text_resp = response::text("Plain text");
```

## Extractors

Extractors are types that implement the `FromRequest` trait to extract data from requests.

### Available Extractors

- `Json<T>`: Extracts and deserializes JSON from request body
- `Query<T>`: Extracts and deserializes query parameters
- `Path<T>`: Extracts and deserializes path parameters
- `State<T>`: Extracts application state from request extensions

### Json Extractor

```rust
use oxidite::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

async fn create_user(Json(data): Json<CreateUser>) -> Result<OxiditeResponse> {
    Ok(response::json(serde_json::json!({
        "id": 1,
        "name": data.name,
        "email": data.email
    })))
}
```

### Query Extractor

```rust
use oxidite::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    limit: Option<u32>,
}

async fn list_items(Query(params): Query<Pagination>) -> Result<OxiditeResponse> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(10);
    
    Ok(response::json(serde_json::json!({
        "page": page,
        "limit": limit
    })))
}
```

### Path Extractor

```rust
use oxidite::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct UserId {
    id: u64,
}

async fn get_user(Path(params): Path<UserId>) -> Result<OxiditeResponse> {
    Ok(response::json(serde_json::json!({
        "id": params.id
    })))
}
```

### State Extractor

```rust
use oxidite::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    app_name: String,
}

async fn handler(State(state): State<Arc<AppState>>) -> Result<OxiditeResponse> {
    Ok(response::json(serde_json::json!({
        "app_name": state.app_name
    })))
}
```

## Application State

To share state across handlers, use the State extractor:

```rust
use oxidite::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    db_pool: DbPool,
    config: Config,
}

// When setting up your service, add state to request extensions
let state = Arc::new(AppState { /* ... */ });

let service = ServiceBuilder::new()
    .layer(AddExtensionLayer::new(state))
    .service(router);
```

Then extract it in handlers:

```rust
async fn handler(State(state): State<Arc<AppState>>) -> Result<OxiditeResponse> {
    // Use state.db_pool, state.config, etc.
    Ok(response::text("Success"))
}
```

## Error Handling

Oxidite uses a Result-based error handling system:

```rust
use oxidite::prelude::*;

// Error variants in oxidite-core
enum Error {
    Server(String),
    NotFound,
    BadRequest(String),
    Unauthorized(String),
    Hyper(hyper::Error),
    Io(std::io::Error),
}

type Result<T> = std::result::Result<T, Error>;

// In handlers
async fn handler(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    // This will return a Server error if it fails
    let data = some_operation_that_might_fail()?;
    
    Ok(response::json(serde_json::json!(data)))
}
```

## Server Configuration

The Server component listens on a socket and handles incoming connections:

```rust
use oxidite::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    router.get("/", handler);
    
    Server::new(router)
        .listen("127.0.0.1:3000".parse().unwrap())
        .await
}
```

The server creates a new task for each incoming connection, enabling concurrent request handling.