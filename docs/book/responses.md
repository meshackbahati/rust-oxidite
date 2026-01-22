# Responses

In Oxidite, responses are how you send data back to clients. The framework provides multiple ways to create responses, from simple text to complex HTML templates.

## Basic Response Types

Oxidite provides several convenience methods on the `Response` type to create different kinds of responses:

### JSON Responses

The most common response type for APIs is JSON:

```rust
use oxidite::prelude::*;

async fn api_handler(_req: Request) -> Result<Response> {
    let data = serde_json::json!({
        "message": "Hello, World!",
        "status": "success"
    });
    
    Ok(Response::json(data))
}
```

### HTML Responses

For server-rendered content, you can create HTML responses:

```rust
async fn home_page(_req: Request) -> Result<Response> {
    Ok(Response::html("<h1>Welcome to Oxidite!</h1>".to_string()))
}
```

### Text Responses

For plain text responses:

```rust
async fn plain_text_handler(_req: Request) -> Result<Response> {
    Ok(Response::text("This is plain text"))
}
```

### Empty Responses

Sometimes you just need to return an empty response with a specific status:

```rust
async fn empty_ok(_req: Request) -> Result<Response> {
    Ok(Response::ok()) // 200 OK
}

async fn no_content(_req: Request) -> Result<Response> {
    Ok(Response::no_content()) // 204 No Content
}
```

## Using the Response Utilities

While the direct methods on `Response` are preferred, you can also use the response utilities:

```rust
use oxidite::response;

async fn alternative_json(_req: Request) -> Result<Response> {
    Ok(response::json(serde_json::json!({ "data": "value" })))
}

async fn alternative_html(_req: Request) -> Result<Response> {
    Ok(response::html("<p>Alternative HTML</p>".to_string()))
}
```

## Custom Responses

For more control, you can create custom responses with specific headers and status codes:

```rust
use hyper::header::{CONTENT_TYPE, LOCATION};
use http::StatusCode;

async fn custom_response(_req: Request) -> Result<Response> {
    use http_body_util::Full;
    use bytes::Bytes;
    
    let mut response = hyper::Response::builder()
        .status(StatusCode::CREATED)
        .header(CONTENT_TYPE, "application/json")
        .header(LOCATION, "/resources/123")
        .body(Full::new(Bytes::from(r#"{"id": 123, "status": "created"}"#)))
        .map_err(|e| Error::Server(e.to_string()))?;
    
    Ok(response)
}
```

## Template Responses

When using the template engine, you can render templates directly as responses:

```rust
use oxidite::prelude::*;
use oxidite_template::{TemplateEngine, Context};

async fn template_handler(_req: Request) -> Result<Response> {
    let mut engine = TemplateEngine::new();
    engine.add_template("index", "<h1>Hello {{ name }}!</h1>")?;
    
    let mut context = Context::new();
    context.set("name", "Oxidite");
    
    // Render directly as response
    let response = engine.render_response("index", &context)?;
    Ok(response)
}
```

## Error Responses

Oxidite provides various error response types that automatically map to appropriate HTTP status codes:

```rust
async fn error_example(_req: Request) -> Result<Response> {
    // This will return a 404 Not Found
    if !resource_exists() {
        return Err(Error::NotFound);
    }
    
    // This will return a 400 Bad Request
    if !valid_input() {
        return Err(Error::BadRequest("Invalid input".to_string()));
    }
    
    // This will return a 401 Unauthorized
    if !authenticated() {
        return Err(Error::Unauthorized("Authentication required".to_string()));
    }
    
    // This will return a 403 Forbidden
    if !authorized() {
        return Err(Error::Forbidden("Access denied".to_string()));
    }
    
    // This will return a 409 Conflict
    if conflict_exists() {
        return Err(Error::Conflict("Resource conflict".to_string()));
    }
    
    // This will return a 422 Unprocessable Entity
    if validation_fails() {
        return Err(Error::Validation("Validation failed".to_string()));
    }
    
    // This will return a 429 Too Many Requests
    if rate_limited() {
        return Err(Error::RateLimited);
    }
    
    // This will return a 503 Service Unavailable
    if service_unavailable() {
        return Err(Error::ServiceUnavailable("Service temporarily unavailable".to_string()));
    }
    
    // Success response
    Ok(Response::json(serde_json::json!({ "status": "success" })))
}
```

## Response Headers

You can also add custom headers to your responses. While the direct Response methods don't expose headers directly, you can create custom responses when needed:

```rust
use hyper::header::{HeaderMap, HeaderValue, CACHE_CONTROL};

async fn cached_response(_req: Request) -> Result<Response> {
    use http_body_util::Full;
    use bytes::Bytes;
    
    let mut response = hyper::Response::builder()
        .status(http::StatusCode::OK)
        .header(CONTENT_TYPE, "application/json")
        .header(CACHE_CONTROL, "public, max-age=3600")
        .body(Full::new(Bytes::from(r#"{"data": "cached"}"#)))
        .map_err(|e| Error::Server(e.to_string()))?;
    
    Ok(response)
}
```

## Streaming Responses

For large data or streaming content, you can create responses with streaming bodies, though this requires more advanced usage:

```rust
use futures::stream::{self, StreamExt};
use http_body_util::StreamBody;
use hyper::body::Frame;

async fn streaming_response(_req: Request) -> Result<Response> {
    let stream = stream::iter(vec![
        Ok::<_, hyper::Error>(Frame::data("chunk1")),
        Ok::<_, hyper::Error>(Frame::data("chunk2")),
        Ok::<_, hyper::Error>(Frame::data("chunk3")),
    ]);
    
    let body = StreamBody::new(stream);
    
    let response = hyper::Response::builder()
        .status(http::StatusCode::OK)
        .header(hyper::header::CONTENT_TYPE, "text/plain")
        .body(body.boxed())
        .map_err(|e| Error::Server(e.to_string()))?;
    
    Ok(response)
}
```

## Summary

The Response API in Oxidite is designed to be intuitive and flexible:

- Use `Response::json()`, `Response::html()`, and `Response::text()` for the most common response types
- Use `Response::ok()` and `Response::no_content()` for empty responses
- Use the template engine's `render_response()` method for server-side rendering
- Handle errors with appropriate `Error` variants that map to correct HTTP status codes
- Fall back to manual response construction for complex scenarios

This approach provides both convenience for common use cases and flexibility for advanced scenarios.