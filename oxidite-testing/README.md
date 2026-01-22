# Oxidite Testing

Comprehensive testing utilities for the Oxidite web framework.

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/oxidite-testing.svg)](https://crates.io/crates/oxidite-testing)
[![Docs.rs](https://docs.rs/oxidite-testing/badge.svg)](https://docs.rs/oxidite-testing)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

</div>

## Overview

`oxidite-testing` provides a comprehensive set of utilities for testing Oxidite web applications. It includes tools for creating test requests, asserting on responses, mocking external dependencies, and setting up test environments with proper fixtures and teardown.

## Installation

Add this to your `Cargo.toml` as a development dependency:

```toml
[dev-dependencies]
oxidite-testing = "0.1"
```

## Features

- **Test Client**: Full-featured HTTP client for integration testing
- **Request Builder**: Fluent API for creating HTTP requests with various content types
- **Response Assertions**: Rich assertion methods for testing response properties
- **Async Test Support**: Built-in async test utilities with proper runtime setup
- **Mock Services**: Tools for mocking external APIs and services
- **Test Fixtures**: Utilities for setting up and tearing down test data
- **Database Testing**: Tools for testing with temporary databases
- **Route Testing**: Easy testing of individual routes and handlers
- **Middleware Testing**: Tools for testing middleware in isolation
- **Integration Testing**: Complete app testing with full middleware stack

## Usage

### Basic Integration Testing

Test your complete application with the TestClient:

```rust
use oxidite::prelude::*;
use oxidite_testing::TestClient;

async fn hello_handler(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    Ok(response::text("Hello, World!"))
}

#[tokio::test]
async fn test_hello_endpoint() {
    let mut router = Router::new();
    router.get("/", hello_handler);
    
    let client = TestClient::new(router);
    let response = client.get("/").send().await;
    
    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await, "Hello, World!");
}
```

### Request Building

Create complex test requests with the fluent API:

```rust
use oxidite_testing::TestClient;
use serde_json::json;

#[tokio::test]
async fn test_post_json() {
    let client = TestClient::new(my_router());
    
    let response = client
        .post("/api/users")
        .header("Content-Type", "application/json")
        .header("Authorization", "Bearer token")
        .json(&json!({
            "name": "John Doe",
            "email": "john@example.com"
        }))
        .send()
        .await;
    
    assert_eq!(response.status(), 201);
    assert_eq!(response.json::<serde_json::Value>().await["name"], "John Doe");
}
```

### Testing Different HTTP Methods

Test all HTTP methods with appropriate assertions:

```rust
use oxidite_testing::TestClient;

#[tokio::test]
async fn test_rest_endpoints() {
    let client = TestClient::new(my_router());
    
    // GET request
    let get_resp = client.get("/api/users/1").send().await;
    assert_eq!(get_resp.status(), 200);
    
    // POST request
    let post_resp = client
        .post("/api/users")
        .json(&json!({"name": "New User"}))
        .send()
        .await;
    assert_eq!(post_resp.status(), 201);
    
    // PUT request
    let put_resp = client
        .put("/api/users/1")
        .json(&json!({"name": "Updated User"}))
        .send()
        .await;
    assert_eq!(put_resp.status(), 200);
    
    // DELETE request
    let del_resp = client.delete("/api/users/1").send().await;
    assert_eq!(del_resp.status(), 204);
}
```

### Response Assertions

Use rich assertion methods to verify response properties:

```rust
use oxidite_testing::TestClient;

#[tokio::test]
async fn test_response_properties() {
    let client = TestClient::new(my_router());
    let response = client.get("/api/users").send().await;
    
    // Status assertions
    response.assert_status(200);
    response.assert_ok();
    response.assert_success();
    
    // Header assertions
    response.assert_header("content-type", "application/json");
    response.assert_has_header("x-request-id");
    
    // Body assertions
    let body = response.text().await;
    assert!(body.contains("users"));
    
    // JSON assertions
    let json_value = response.json::<serde_json::Value>().await;
    assert_eq!(json_value["data"].as_array().unwrap().len(), 5);
}
```

### Testing with State and Extractors

Test handlers that use Oxidite's extractor system:

```rust
use oxidite::prelude::*;
use oxidite_testing::TestClient;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    db_connection: String, // Mock database connection
}

async fn handler_with_state(
    _req: OxiditeRequest,
    State(state): State<Arc<AppState>>
) -> Result<OxiditeResponse> {
    Ok(response::json(serde_json::json!({
        "db": state.db_connection
    })))
}

#[tokio::test]
async fn test_with_state() {
    let app_state = Arc::new(AppState {
        db_connection: "mock_db".to_string(),
    });
    
    let mut router = Router::new();
    router.get("/", handler_with_state);
    
    let client = TestClient::with_state(router, app_state);
    let response = client.get("/").send().await;
    
    assert_eq!(response.status(), 200);
    let json = response.json::<serde_json::Value>().await;
    assert_eq!(json["db"], "mock_db");
}
```

### Testing Individual Handlers

Test individual handler functions in isolation:

```rust
use oxidite::prelude::*;
use oxidite_testing::TestRequest;

async fn create_user_handler(
    Json(payload): Json<serde_json::Value>
) -> Result<OxiditeResponse> {
    if payload["name"].is_null() {
        return Err(OxiditeError::BadRequest("Name is required".to_string()));
    }
    
    Ok(response::json(json!({
        "id": 1,
        "name": payload["name"].as_str().unwrap()
    })))
}

#[tokio::test]
async fn test_create_user_handler() {
    // Test successful creation
    let req = TestRequest::post("/")
        .json(&json!({"name": "John"}))
        .build();
    
    let response = create_user_handler(req.extract().await.unwrap()).await.unwrap();
    assert_eq!(response.status().as_u16(), 200);
    
    // Test validation error
    let req = TestRequest::post("/")
        .json(&json!({"email": "test@example.com"}))
        .build();
    
    let result = create_user_handler(req.extract().await.unwrap()).await;
    assert!(result.is_err());
    match result.err().unwrap() {
        OxiditeError::BadRequest(_) => {}, // Expected
        _ => panic!("Expected BadRequest error"),
    }
}
```

### Mocking External Services

Mock external API calls and services:

```rust
use oxidite_testing::MockServer;

#[tokio::test]
async fn test_with_external_service() {
    // Start a mock server
    let mock_server = MockServer::start().await;
    
    // Create a mock endpoint
    mock_server
        .mock(|when, then| {
            when.path("/api/external");
            then.status(200).json_body(json!({"data": "mocked"}));
        })
        .await;
    
    // Configure your app to use the mock server URL
    std::env::set_var("EXTERNAL_API_URL", mock_server.url());
    
    // Test your application
    let client = TestClient::new(my_router());
    let response = client.get("/use-external-api").send().await;
    
    assert_eq!(response.status(), 200);
    
    // Verify the mock was called
    let mocks = mock_server.received_requests().await;
    assert_eq!(mocks.len(), 1);
}
```

### Database Testing

Test with temporary databases:

```rust
use oxidite_testing::TestDatabase;

#[tokio::test]
async fn test_with_database() {
    // Create a temporary database
    let test_db = TestDatabase::new("sqlite").await;
    let pool = test_db.pool();
    
    // Insert test data
    // ... insert test records ...
    
    // Test your application
    let client = TestClient::with_state(
        my_router_with_db(pool.clone()),
        Arc::new(AppState { db_pool: pool })
    );
    
    let response = client.get("/api/users").send().await;
    assert_eq!(response.status(), 200);
    
    // Database is automatically cleaned up
}
```

### Testing Middleware

Test middleware components in isolation:

```rust
use tower::{Service, ServiceExt, ServiceBuilder};
use http::{Request, Response, StatusCode};
use oxidite_testing::TestRequest;
use oxidite_middleware::LoggerLayer;

#[tokio::test]
async fn test_logging_middleware() {
    let mut service = ServiceBuilder::new()
        .layer(LoggerLayer::new())
        .service_fn(echo_service);
    
    let request = Request::builder()
        .uri("/")
        .body(hyper::Body::empty())
        .unwrap();
    
    let response = service.ready().await.unwrap().call(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

async fn echo_service(req: Request<hyper::Body>) -> Result<Response<hyper::Body>, std::convert::Infallible> {
    Ok(Response::new(req.into_body()))
}
```

### Running Tests

Run your tests with cargo:

```bash
# Run all tests
cargo test

# Run tests with logging
RUST_LOG=debug cargo test

# Run specific test
cargo test test_hello_endpoint

# Run tests in parallel (default)
cargo test -- --test-threads=4

# Run tests sequentially
cargo test -- --test-threads=1
```

### Test Organization

Organize your tests effectively:

```rust
// src/lib.rs or src/main.rs
#[cfg(test)]
mod tests {
    use super::*;
    use oxidite_testing::TestClient;
    
    #[tokio::test]
    async fn unit_tests() {
        // Test individual functions
    }
    
    mod integration {
        use super::*;
        
        #[tokio::test]
        async fn api_tests() {
            // Test API endpoints
        }
        
        #[tokio::test]
        async fn auth_tests() {
            // Test authentication
        }
    }
    
    mod performance {
        use super::*;
        
        #[tokio::test]
        async fn load_tests() {
            // Performance/load tests
        }
    }
}
```

## Best Practices

- Use descriptive test names that explain what is being tested
- Test both success and failure scenarios
- Use appropriate assertion libraries for complex comparisons
- Mock external dependencies to isolate your code
- Test edge cases and error conditions
- Use test fixtures to set up common test data
- Keep tests fast and deterministic
- Organize tests in logical groups

## License

MIT
