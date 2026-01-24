# oxidite-core

Core HTTP server and routing for the Oxidite web framework.

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/oxidite-core.svg)](https://crates.io/crates/oxidite-core)
[![Docs.rs](https://docs.rs/oxidite-core/badge.svg)](https://docs.rs/oxidite-core)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

</div>

## Overview

`oxidite-core` provides the foundational building blocks for the Oxidite web framework, including the HTTP server, router, request/response types, and type-safe extractors. It serves as the base layer upon which all other Oxidite components are built.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
oxidite-core = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Features

- **HTTP/1.1 and HTTP/2 support** - Built on top of `hyper` for high-performance HTTP handling
- **WebSocket support** - Built-in WebSocket functionality
- **Type-safe extractors** - Safe and ergonomic request data extraction
- **Flexible routing** - Support for path parameters, wildcards, and complex routing patterns
- **Middleware support** - Integration with the `tower` ecosystem
- **Async-first** - Designed with Rust's async/await from the ground up

## Usage

### Basic Server

```rust
use oxidite_core::{Router, Server, Response};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut router = Router::new();
    
    router.get("/", |_req| async {
        Ok(Response::text("Hello, Oxidite!"))
    });
    
    Server::new(router)
        .listen("127.0.0.1:3000".parse().unwrap())
        .await
}
```

### Using Extractors

`oxidite-core` provides several type-safe extractors for handling different kinds of requests:

```rust
use oxidite_core::{Router, Server, Response, Error, extract::*};
use serde::{Serialize, Deserialize};

#[derive(Deserialize, Serialize)]
struct User {
    name: String,
    email: String,
}

// JSON extractor
async fn create_user(Json(user): Json<User>) -> Result<Response, Error> {
    Ok(Response::json(serde_json::json!({
        "message": "User created successfully",
        "user": user
    })))
}

// Query parameters extractor
async fn search_users(Query(params): Query<User>) -> Result<Response, Error> {
    Ok(Response::json(serde_json::json!(params)))
}

// Path parameters extractor
#[derive(Deserialize)]
struct UserId {
    id: u32,
}

async fn get_user(Path(params): Path<UserId>) -> Result<Response, Error> {
    Ok(Response::json(serde_json::json!({
        "id": params.id,
        "name": "Sample User"
    })))
}

// Form data extractor
async fn create_user_from_form(Form(user): Form<User>) -> Result<Response, Error> {
    Ok(Response::json(serde_json::json!({
        "message": "User created from form",
        "user": user
    })))
}

// Cookies extractor
async fn get_cookies(Cookies(cookies): Cookies) -> Result<Response, Error> {
    Ok(Response::json(serde_json::json!(cookies)))
}

// Raw body extractor
async fn handle_raw_body(Body(body): Body) -> Result<Response, Error> {
    Ok(Response::text(format!("Received {} characters", body.len())))
}
```

### Advanced Routing

```rust
use oxidite_core::{Router, Server, Response};

let mut router = Router::new();

// Basic routes
router.get("/", handler);
router.post("/users", create_user);
router.put("/users/:id", update_user);
router.delete("/users/:id", delete_user);

// Path parameters
router.get("/users/:user_id/posts/:post_id", get_post);

// Wildcard routes (should be registered last)
router.get("/static/*", serve_static);
```

### Response Utilities

`oxidite-core` provides convenient response utilities:

```rust
use oxidite_core::Response;

// Text response
let text_response = Response::text("Plain text");

// JSON response
let json_response = Response::json(serde_json::json!({
    "message": "Success",
    "data": [1, 2, 3]
}));

// HTML response
let html_response = Response::html("<h1>HTML Content</h1>");

// Custom status code
let custom_response = Response::builder()
    .status(418) // I'm a teapot
    .body("Teapot response");
```

## Error Handling

The framework provides comprehensive error handling with appropriate HTTP status codes:

```rust
use oxidite_core::Error;

// Various error types
let not_found_error = Error::NotFound("Resource not found".to_string());
let bad_request_error = Error::BadRequest("Invalid request".to_string());
let forbidden_error = Error::Forbidden("Access denied".to_string());
let conflict_error = Error::Conflict("Resource conflict".to_string());
```

## License

MIT
