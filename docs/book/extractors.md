# Request Extractors

Request extractors are a key feature in Oxidite that allow you to extract data from incoming HTTP requests in a type-safe manner. This chapter covers all the available extractors and how to use them effectively.

## Overview

Extractors in Oxidite implement the `FromRequest` trait, which allows them to automatically extract data from requests when used as parameters in handler functions. This provides a clean and type-safe way to access different parts of the request.

## Available Extractors

### Path Extractor

The `Path` extractor extracts path parameters from the URL:

```rust
use oxidite::prelude::*;

// Single parameter
async fn get_user(Path(user_id): Path<u32>) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "user_id": user_id,
        "name": format!("User {}", user_id)
    })))
}

// Multiple parameters as tuple
async fn get_user_post(Path((user_id, post_id)): Path<(u32, u32)>) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "user_id": user_id,
        "post_id": post_id
    })))
}

// Parameters as struct
use serde::Deserialize;

#[derive(Deserialize)]
struct UserParams {
    user_id: u32,
}

async fn get_user_by_struct(Path(params): Path<UserParams>) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "user_id": params.user_id,
        "name": format!("User {}", params.user_id)
    })))
}
```

### Query Extractor

The `Query` extractor extracts query parameters from the URL:

```rust
use oxidite::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct UserQuery {
    page: Option<u32>,
    limit: Option<u32>,
    sort: Option<String>,
    active: Option<bool>,
}

async fn get_users(Query(params): Query<UserQuery>) -> Result<Response> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(10);
    let sort = params.sort.unwrap_or_else(|| "id".to_string());
    let active = params.active.unwrap_or(true);
    
    Ok(Response::json(serde_json::json!({
        "pagination": {
            "page": page,
            "limit": limit
        },
        "sorting": sort,
        "filter": { "active": active }
    })))
}

// Raw query string access
async fn handle_raw_query(Query(raw): Query<serde_json::Value>) -> Result<Response> {
    Ok(Response::json(raw))
}
```

### Json Extractor

The `Json` extractor parses JSON from the request body:

```rust
use oxidite::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
    age: u8,
}

async fn create_user(Json(payload): Json<CreateUser>) -> Result<Response> {
    // payload contains the deserialized JSON data
    Ok(Response::json(serde_json::json!({
        "message": "User created successfully",
        "user": {
            "id": 123, // In a real app, this would come from your database
            "name": payload.name,
            "email": payload.email,
            "age": payload.age
        }
    })))
}

// Generic JSON handling
async fn handle_generic_json(Json(data): Json<serde_json::Value>) -> Result<Response> {
    // Process any JSON data
    Ok(Response::json(serde_json::json!({
        "received": data,
        "processed": true
    })))
}
```

### Form Extractor

The `Form` extractor handles `application/x-www-form-urlencoded` data:

```rust
use oxidite::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
    remember_me: Option<bool>,
}

async fn login_handler(Form(login_data): Form<LoginForm>) -> Result<Response> {
    // login_data contains the deserialized form data
    if login_data.username == "admin" && login_data.password == "secret" {
        Ok(Response::json(serde_json::json!({
            "status": "success",
            "message": "Login successful",
            "remember_me": login_data.remember_me.unwrap_or(false)
        })))
    } else {
        Err(Error::Unauthorized("Invalid credentials".to_string()))
    }
}

// Generic form handling
async fn handle_generic_form(Form(data): Form<serde_json::Value>) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "form_data": data,
        "status": "received"
    })))
}
```

### Cookies Extractor

The `Cookies` extractor provides access to request cookies:

```rust
use oxidite::prelude::*;

async fn handle_cookies(cookies: Cookies) -> Result<Response> {
    let mut response_data = serde_json::json!({
        "cookie_count": 0,
        "cookies": {}
    });

    let mut cookies_map = serde_json::Map::new();
    let mut count = 0;

    for (name, value) in cookies.iter() {
        cookies_map.insert(name.to_string(), serde_json::Value::String(value.to_string()));
        count += 1;
    }

    if count > 0 {
        response_data["cookie_count"] = serde_json::Value::Number(count.into());
        response_data["cookies"] = serde_json::Value::Object(cookies_map);
    }

    Ok(Response::json(response_data))
}

// Access specific cookies
async fn get_session(cookies: Cookies) -> Result<Response> {
    let session_id = cookies.get("session_id");
    let theme = cookies.get("theme").unwrap_or("light");
    
    Ok(Response::json(serde_json::json!({
        "session_id": session_id,
        "theme": theme,
        "has_session": session_id.is_some()
    })))
}
```

### Body Extractor

The `Body` extractor provides access to the raw request body:

```rust
use oxidite::prelude::*;

// Extract as string
async fn handle_text_body(Body(raw_body): Body<String>) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "length": raw_body.len(),
        "content": raw_body,
        "type": "text"
    })))
}

// Extract as bytes
async fn handle_binary_body(Body(bytes): Body<Vec<u8>>) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "size": bytes.len(),
        "type": "binary"
    })))
}

// Extract as Bytes
use bytes::Bytes;

async fn handle_bytes_body(Body(bytes): Body<Bytes>) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "size": bytes.len(),
        "type": "bytes"
    })))
}
```

### State Extractor

The `State` extractor provides access to application state:

```rust
use oxidite::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    app_name: String,
    version: String,
    database_url: String,
}

async fn handler_with_state(State(state): State<Arc<AppState>>) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "app_name": state.app_name,
        "version": state.version,
        "has_database": !state.database_url.is_empty()
    })))
}

#[tokio::main]
async fn main() -> Result<()> {
    let app_state = Arc::new(AppState {
        app_name: "MyApp".to_string(),
        version: "1.0.0".to_string(),
        database_url: "postgresql://localhost/myapp".to_string(),
    });

    let mut router = Router::new();
    
    // Attach state to router
    router.with_state(app_state);
    router.get("/info", handler_with_state);
    
    Server::new(router)
        .listen("127.0.0.1:3000".parse()?)
        .await
}
```

## Combining Multiple Extractors

You can use multiple extractors in a single handler:

```rust
use oxidite::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct CommentQuery {
    page: Option<u32>,
    limit: Option<u32>,
}

#[derive(Deserialize)]
struct CreateComment {
    content: String,
    parent_id: Option<u32>,
}

// Example combining multiple extractors
async fn complex_handler(
    Path((post_id, comment_id)): Path<(u32, u32)>,
    Query(params): Query<CommentQuery>,
    Json(payload): Json<CreateComment>,
    cookies: Cookies,
    State(app_state): State<Arc<AppState>>
) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "post_id": post_id,
        "comment_id": comment_id,
        "query_params": params,
        "request_body": payload,
        "cookies_present": cookies.iter().count(),
        "app_info": app_state.app_name
    })))
}
```

## Custom Extractors

You can create custom extractors by implementing the `FromRequest` trait:

```rust
use oxidite::prelude::*;
use std::future::Future;
use std::pin::Pin;

// Custom extractor for authenticated users
#[derive(Clone)]
struct AuthenticatedUser {
    id: u32,
    username: String,
    permissions: Vec<String>,
}

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &Request) -> Self::Future {
        Box::pin(async move {
            // Extract auth token from headers
            let auth_header = req.headers()
                .get("authorization")
                .and_then(|hv| hv.to_str().ok());

            match auth_header {
                Some(token) if token.starts_with("Bearer ") => {
                    let token = token.trim_start_matches("Bearer ");
                    
                    // Validate token and fetch user (simplified)
                    if validate_and_fetch_user(token).await.is_ok() {
                        Ok(AuthenticatedUser {
                            id: 1,
                            username: "john_doe".to_string(),
                            permissions: vec!["read".to_string(), "write".to_string()],
                        })
                    } else {
                        Err(Error::Unauthorized("Invalid token".to_string()))
                    }
                }
                _ => Err(Error::Unauthorized("Missing or invalid token".to_string()))
            }
        })
    }
}

async fn validate_and_fetch_user(_token: &str) -> Result<(), ()> {
    // In a real app, validate against your auth system
    Ok(())
}

async fn protected_handler(user: AuthenticatedUser) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "user_id": user.id,
        "username": user.username,
        "permissions": user.permissions
    })))
}
```

## Error Handling with Extractors

Extractors automatically handle parsing errors and return appropriate HTTP status codes:

```rust
use oxidite::prelude::*;

// If JSON parsing fails, returns 400 Bad Request
async fn handle_bad_json(Json(data): Json<serde_json::Value>) -> Result<Response> {
    Ok(Response::json(data))
}

// If path parameter parsing fails (e.g., "abc" for u32), returns 400 Bad Request
async fn handle_bad_path(Path(id): Path<u32>) -> Result<Response> {
    Ok(Response::json(serde_json::json!({ "id": id })))
}

// If query parameter parsing fails, returns 400 Bad Request
use serde::Deserialize;

#[derive(Deserialize)]
struct BadQuery {
    number: u32,
}

async fn handle_bad_query(Query(params): Query<BadQuery>) -> Result<Response> {
    Ok(Response::json(serde_json::json!({ "number": params.number })))
}
```

## Performance Considerations

1. **Extractor Ordering**: Place extractors that are most likely to fail early in the handler signature to fail fast.

2. **Body Consumption**: Be aware that the request body can only be consumed once. If you need to access the body multiple times, you'll need to store it in state or parse it once and store the result.

3. **Validation**: Consider validating data after extraction rather than during extraction for better performance:

```rust
use oxidite::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct UserData {
    email: String,
    age: u8,
}

async fn create_user_validated(Json(mut user): Json<UserData>) -> Result<Response> {
    // Validate after extraction
    if !is_valid_email(&user.email) {
        return Err(Error::Validation("Invalid email format".to_string()));
    }
    
    if user.age < 13 {
        return Err(Error::Validation("User must be at least 13 years old".to_string()));
    }
    
    // Process valid user
    Ok(Response::json(serde_json::json!({ "status": "created", "user": user })))
}

fn is_valid_email(email: &str) -> bool {
    email.contains('@') && email.contains('.')
}
```

## Summary

Request extractors provide a powerful and type-safe way to access different parts of HTTP requests:

- Use `Path<T>` for path parameters
- Use `Query<T>` for query parameters
- Use `Json<T>` for JSON request bodies
- Use `Form<T>` for form data
- Use `Cookies` for cookie access
- Use `Body<T>` for raw request bodies
- Use `State<T>` for application state
- Combine multiple extractors as needed
- Handle errors appropriately
- Consider performance implications

Extractors are a fundamental part of Oxidite's design and enable clean, readable handler functions.