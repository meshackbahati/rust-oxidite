# Requests

The Request type in Oxidite represents incoming HTTP requests. This chapter covers how to work with requests, extract information from them, and handle different types of request data.

## Overview

In Oxidite, the `Request` type wraps the underlying `hyper::Request` and provides access to all the information contained in an HTTP request. While extractors provide a convenient way to access specific parts of the request, sometimes you need direct access to the request object itself.

## Basic Request Access

You can access a request directly in your handler:

```rust
use oxidite::prelude::*;

async fn inspect_request(req: Request) -> Result<Response> {
    let method = req.method();
    let uri = req.uri();
    let version = req.version();
    
    Ok(Response::json(serde_json::json!({
        "method": method.to_string(),
        "uri": uri.to_string(),
        "version": format!("{:?}", version),
        "headers": extract_headers(&req)
    })))
}

fn extract_headers(req: &Request) -> serde_json::Value {
    let mut headers = serde_json::Map::new();
    
    for (name, value) in req.headers() {
        if let Ok(value_str) = value.to_str() {
            headers.insert(name.as_str().to_string(), serde_json::Value::String(value_str.to_string()));
        }
    }
    
    serde_json::Value::Object(headers)
}
```

## Accessing Request Headers

You can access headers from the request object:

```rust
use oxidite::prelude::*;

async fn handle_headers(req: Request) -> Result<Response> {
    // Access specific headers
    let content_type = req.headers().get("content-type")
        .and_then(|hv| hv.to_str().ok())
        .unwrap_or("unknown");
    
    let user_agent = req.headers().get("user-agent")
        .and_then(|hv| hv.to_str().ok())
        .unwrap_or("unknown");
    
    let authorization = req.headers().get("authorization")
        .and_then(|hv| hv.to_str().ok())
        .map(|s| s.to_string());
    
    Ok(Response::json(serde_json::json!({
        "content_type": content_type,
        "user_agent": user_agent,
        "has_auth": authorization.is_some(),
        "auth_scheme": authorization.as_ref().map(|auth| {
            auth.split_whitespace().next().unwrap_or("unknown").to_string()
        })
    })))
}

// Case-insensitive header access
use hyper::header::USER_AGENT;

async fn handle_specific_header(req: Request) -> Result<Response> {
    let user_agent = req.headers()
        .get(USER_AGENT)
        .and_then(|hv| hv.to_str().ok())
        .unwrap_or("unknown");
    
    Ok(Response::json(serde_json::json!({ "user_agent": user_agent })))
}
```

## Accessing Request URI and Query Parameters

You can access the URI and its components:

```rust
use oxidite::prelude::*;

async fn inspect_uri(req: Request) -> Result<Response> {
    let uri = req.uri();
    
    Ok(Response::json(serde_json::json!({
        "scheme": uri.scheme().map(|s| s.to_string()).unwrap_or_default(),
        "authority": uri.authority().map(|a| a.to_string()).unwrap_or_default(),
        "path": uri.path(),
        "query": uri.query().unwrap_or_default(),
        "full_uri": uri.to_string()
    })))
}

// Parse query parameters manually (though Query extractor is preferred)
use std::collections::HashMap;

async fn manual_query_parsing(req: Request) -> Result<Response> {
    let query_string = req.uri().query().unwrap_or_default();
    
    let params: HashMap<String, String> = query_string
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.split('=');
            let key = parts.next()?;
            let value = parts.next().unwrap_or("");
            Some((key.to_string(), value.to_string()))
        })
        .collect();
    
    Ok(Response::json(serde_json::json!(params)))
}
```

## Accessing Request Method and Version

You can inspect the HTTP method and protocol version:

```rust
use oxidite::prelude::*;

async fn method_inspector(req: Request) -> Result<Response> {
    let method = req.method();
    let version = req.version();
    
    let method_str = match *method {
        http::Method::GET => "GET",
        http::Method::POST => "POST",
        http::Method::PUT => "PUT",
        http::Method::DELETE => "DELETE",
        http::Method::PATCH => "PATCH",
        http::Method::HEAD => "HEAD",
        http::Method::OPTIONS => "OPTIONS",
        _ => "OTHER",
    };
    
    let version_str = match version {
        http::Version::HTTP_09 => "HTTP/0.9",
        http::Version::HTTP_10 => "HTTP/1.0",
        http::Version::HTTP_11 => "HTTP/1.1",
        http::Version::HTTP_2 => "HTTP/2.0",
        http::Version::HTTP_3 => "HTTP/3.0",
        _ => "UNKNOWN",
    };
    
    Ok(Response::json(serde_json::json!({
        "method": method_str,
        "version": version_str,
        "is_standard_method": method.is_safe() || method.is_idempotent() || method.is_extension()
    })))
}
```

## Working with Request Extensions

Request extensions provide a way to store and access custom data:

```rust
use oxidite::prelude::*;
use std::any::Any;

// Define custom extension types
#[derive(Debug, Clone)]
struct RequestMetadata {
    request_id: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

async fn handle_extensions(mut req: Request) -> Result<Response> {
    // Store data in request extensions
    req.extensions_mut().insert(RequestMetadata {
        request_id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now(),
    });
    
    // Later, you can retrieve it
    if let Some(metadata) = req.extensions().get::<RequestMetadata>() {
        return Ok(Response::json(serde_json::json!({
            "request_id": metadata.request_id,
            "timestamp": metadata.timestamp.to_rfc3339()
        })));
    }
    
    Ok(Response::json(serde_json::json!({ "status": "no_metadata" })))
}
```

## Accessing the Request Body

While extractors are preferred for body access, you can access the raw body directly:

```rust
use oxidite::prelude::*;
use http_body_util::BodyExt;

async fn access_raw_body(mut req: Request) -> Result<Response> {
    // Collect the entire body
    let body_bytes = req
        .body_mut()
        .collect()
        .await
        .map_err(|e| Error::Server(e.to_string()))?
        .to_bytes();
    
    let body_str = String::from_utf8_lossy(&body_bytes);
    
    Ok(Response::json(serde_json::json!({
        "body_size": body_bytes.len(),
        "body_content": body_str.to_string(),
        "is_valid_utf8": std::str::from_utf8(&body_bytes).is_ok()
    })))
}
```

## Request Validation

You can perform validation directly on the request:

```rust
use oxidite::prelude::*;

async fn validate_request(req: Request) -> Result<Response> {
    // Validate content type
    let content_type = req.headers()
        .get("content-type")
        .and_then(|hv| hv.to_str().ok())
        .unwrap_or("");
    
    if !content_type.starts_with("application/json") {
        return Err(Error::BadRequest("Content-Type must be application/json".to_string()));
    }
    
    // Validate method
    if *req.method() != http::Method::POST {
        return Err(Error::BadRequest("Only POST method allowed".to_string()));
    }
    
    // Validate size limits
    let content_length = req.headers()
        .get("content-length")
        .and_then(|hv| hv.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);
    
    const MAX_SIZE: usize = 1024 * 1024; // 1MB
    if content_length > MAX_SIZE {
        return Err(Error::BadRequest("Request body too large".to_string()));
    }
    
    Ok(Response::json(serde_json::json!({ "status": "validated" })))
}
```

## Request Context and State

Access application state alongside the request:

```rust
use oxidite::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
struct AppContext {
    app_name: String,
    version: String,
    maintenance_mode: bool,
}

async fn contextual_handler(
    req: Request,
    State(ctx): State<Arc<AppContext>>
) -> Result<Response> {
    // Combine request data with application context
    let user_agent = req.headers()
        .get("user-agent")
        .and_then(|hv| hv.to_str().ok())
        .unwrap_or("unknown");
    
    if ctx.maintenance_mode {
        return Err(Error::ServiceUnavailable("Service temporarily unavailable".to_string()));
    }
    
    Ok(Response::json(serde_json::json!({
        "app": {
            "name": ctx.app_name,
            "version": ctx.version
        },
        "client": {
            "user_agent": user_agent
        },
        "request_info": {
            "method": req.method().to_string(),
            "path": req.uri().path()
        }
    })))
}
```

## Middleware with Request Access

You can access and modify requests in middleware:

```rust
use oxidite::prelude::*;

async fn request_logging_middleware(req: Request, next: Next) -> Result<Response> {
    let method = req.method().clone();
    let uri = req.uri().clone();
    
    println!("Incoming request: {} {}", method, uri);
    
    let start = std::time::Instant::now();
    let response = next.run(req).await?;
    let duration = start.elapsed();
    
    println!("Request completed in {:?}", duration);
    
    Ok(response)
}

async fn add_request_id_middleware(req: Request, next: Next) -> Result<Response> {
    // Add a request ID to the request extensions
    let request_id = uuid::Uuid::new_v4().to_string();
    
    let mut req = req;
    req.extensions_mut().insert(("request_id", request_id.clone()));
    
    let mut response = next.run(req).await?;
    
    // Add request ID to response headers
    use hyper::header::HeaderMap;
    let mut headers = HeaderMap::new();
    headers.insert("X-Request-ID", request_id.parse().unwrap());
    
    // In a real implementation, you'd merge these headers with the response
    Ok(response)
}
```

## Security Considerations

When working with requests, consider these security aspects:

```rust
use oxidite::prelude::*;

async fn secure_request_handler(req: Request) -> Result<Response> {
    // Check for suspicious headers
    let suspicious_headers = ["x-forwarded-for", "x-real-ip", "x-client-ip"];
    for header in suspicious_headers {
        if req.headers().get(header).is_some() {
            println!("Warning: Suspicious header {} detected", header);
        }
    }
    
    // Validate host header to prevent host header attacks
    if let Some(host) = req.headers().get("host") {
        if let Ok(host_str) = host.to_str() {
            // Validate against allowed hosts
            if !is_allowed_host(host_str) {
                return Err(Error::BadRequest("Invalid Host header".to_string()));
            }
        }
    }
    
    // Check for potential SQL injection patterns in URI
    let uri_path = req.uri().path();
    if contains_sql_patterns(uri_path) {
        return Err(Error::BadRequest("Potential SQL injection detected".to_string()));
    }
    
    Ok(Response::json(serde_json::json!({ "status": "secure" })))
}

fn is_allowed_host(host: &str) -> bool {
    // In a real app, check against allowed hosts
    host.ends_with(".yourdomain.com") || host.starts_with("localhost")
}

fn contains_sql_patterns(text: &str) -> bool {
    let sql_patterns = ["'", "\"", "--", "/*", "*/", "xp_", "sp_"];
    sql_patterns.iter().any(|pattern| text.to_lowercase().contains(pattern))
}
```

## Performance Tips

1. **Use Extractors**: Use extractors instead of manual request parsing when possible - they're optimized and handle errors properly.

2. **Minimize Body Access**: Access the request body only when necessary, as it consumes the body stream.

3. **Cache Computed Values**: If you compute something from the request multiple times, store it in extensions.

4. **Validate Early**: Perform validation early in the request lifecycle to fail fast.

## Summary

Working with requests in Oxidite involves:

- Direct access to request metadata (method, URI, headers, version)
- Use of extensions for storing custom request data
- Proper validation and security checks
- Integration with state and middleware
- Following security best practices

While extractors are often the preferred approach for accessing specific request data, direct request access gives you full control over request inspection and manipulation.