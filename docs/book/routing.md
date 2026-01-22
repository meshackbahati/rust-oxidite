# Basic Routing

Routing is how your Oxidite application maps HTTP requests to handler functions. This chapter covers the fundamentals of routing in Oxidite.

## Basic Route Definitions

Routes in Oxidite are defined by mapping HTTP methods and paths to handler functions:

```rust
use oxidite::prelude::*;

// Define a handler function
async fn hello_world(_req: Request) -> Result<Response> {
    Ok(Response::text("Hello, World!"))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    
    // Register a GET route at "/"
    router.get("/", hello_world);
    
    Server::new(router)
        .listen("127.0.0.1:3000".parse()?)
        .await
}
```

## Supported HTTP Methods

Oxidite supports all standard HTTP methods:

```rust
use oxidite::prelude::*;

async fn handle_get(_req: Request) -> Result<Response> {
    Ok(Response::text("GET request handled"))
}

async fn handle_post(_req: Request) -> Result<Response> {
    Ok(Response::text("POST request handled"))
}

async fn handle_put(_req: Request) -> Result<Response> {
    Ok(Response::text("PUT request handled"))
}

async fn handle_delete(_req: Request) -> Result<Response> {
    Ok(Response::text("DELETE request handled"))
}

async fn handle_patch(_req: Request) -> Result<Response> {
    Ok(Response::text("PATCH request handled"))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    
    router.get("/resource", handle_get);
    router.post("/resource", handle_post);
    router.put("/resource", handle_put);
    router.delete("/resource", handle_delete);
    router.patch("/resource", handle_patch);
    
    Server::new(router)
        .listen("127.0.0.1:3000".parse()?)
        .await
}
```

## Path Parameters

Oxidite supports path parameters that can be extracted using the `Path` extractor:

```rust
use oxidite::prelude::*;
use serde::Deserialize;

// Handler with path parameter
async fn get_user(Path(user_id): Path<u32>) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "user_id": user_id,
        "name": format!("User {}", user_id),
        "email": format!("user{}@example.com", user_id)
    })))
}

// Handler with multiple path parameters
async fn get_user_post(Path((user_id, post_id)): Path<(u32, u32)>) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "user_id": user_id,
        "post_id": post_id,
        "title": format!("Post {} by User {}", post_id, user_id)
    })))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    
    // Single parameter: /users/123
    router.get("/users/:user_id", get_user);
    
    // Multiple parameters: /users/123/posts/456
    router.get("/users/:user_id/posts/:post_id", get_user_post);
    
    Server::new(router)
        .listen("127.0.0.1:3000".parse()?)
        .await
}
```

### Named Struct for Path Parameters

You can also use a named struct for better organization:

```rust
use oxidite::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct UserId {
    user_id: u32,
}

#[derive(Deserialize)]
struct UserPostId {
    user_id: u32,
    post_id: u32,
}

async fn get_user_by_struct(Path(params): Path<UserId>) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "user_id": params.user_id,
        "name": format!("User {}", params.user_id)
    })))
}

async fn get_user_post_by_struct(Path(params): Path<UserPostId>) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "user_id": params.user_id,
        "post_id": params.post_id,
        "title": format!("Post {} by User {}", params.post_id, params.user_id)
    })))
}
```

## Query Parameters

Query parameters can be extracted using the `Query` extractor:

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
        "users": [], // In a real app, this would come from your database
        "pagination": {
            "page": page,
            "limit": limit,
            "total": 100 // In a real app, this would be the actual count
        },
        "filters": {
            "sort": sort,
            "active": active
        }
    })))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    
    // Handles: /users?page=2&limit=20&sort=name&active=true
    router.get("/users", get_users);
    
    Server::new(router)
        .listen("127.0.0.1:3000".parse()?)
        .await
}
```

## Route Groups and Nesting

You can group related routes for better organization:

```rust
use oxidite::prelude::*;

// API versioning example
async fn v1_users(_req: Request) -> Result<Response> {
    Ok(Response::json(serde_json::json!({ "version": "v1", "endpoint": "users" })))
}

async fn v2_users(_req: Request) -> Result<Response> {
    Ok(Response::json(serde_json::json!({ "version": "v2", "endpoint": "users", "enhanced": true })))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    
    // Versioned APIs
    router.get("/api/v1/users", v1_users);
    router.get("/api/v2/users", v2_users);
    
    Server::new(router)
        .listen("127.0.0.1:3000".parse()?)
        .await
}
```

## Wildcard Routes

Oxidite supports wildcard routes for catch-all functionality:

```rust
use oxidite::prelude::*;

async fn catch_all(_req: Request) -> Result<Response> {
    Ok(Response::text("Page not found".to_string()))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    
    // Register your specific routes first
    router.get("/", |_req| async { Ok(Response::text("Home page".to_string())) });
    router.get("/about", |_req| async { Ok(Response::text("About page".to_string())) });
    
    // Wildcard route should be registered last
    // This will catch any routes not matched by previous handlers
    router.get("/*", catch_all);
    
    Server::new(router)
        .listen("127.0.0.1:3000".parse()?)
        .await
}
```

## Route Middleware

You can apply middleware to specific routes or route groups:

```rust
use oxidite::prelude::*;

async fn logging_middleware(req: Request, next: Next) -> Result<Response> {
    println!("Request: {} {}", req.method(), req.uri());
    let response = next.run(req).await?;
    println!("Response: {}", response.status());
    Ok(response)
}

async fn protected_route(_req: Request) -> Result<Response> {
    Ok(Response::text("This is a protected route".to_string()))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    
    // Apply middleware to a specific route
    router.get("/protected")
        .middleware(logging_middleware)
        .handler(protected_route);
    
    Server::new(router)
        .listen("127.0.0.1:3000".parse()?)
        .await
}
```

## Route Organization Best Practices

### 1. Order Matters
Register more specific routes before general ones:

```rust
// ✅ Correct order
router.get("/users/:id", get_user);
router.get("/users/list", list_users); // More specific than /users/:id

// ❌ Wrong order - this would never be reached
// router.get("/users/list", list_users);
// router.get("/users/:id", get_user);  // Would match /users/list first
```

### 2. Group Related Routes
Keep related functionality together:

```rust
// Group user-related routes
router.get("/users", get_users);
router.post("/users", create_user);
router.get("/users/:id", get_user);
router.put("/users/:id", update_user);
router.delete("/users/:id", delete_user);

// Group post-related routes
router.get("/posts", get_posts);
router.post("/posts", create_post);
router.get("/posts/:id", get_post);
```

### 3. Use Descriptive Names
Make your route patterns descriptive and consistent:

```rust
// Good: clear and RESTful
"/users/:user_id/posts/:post_id/comments"
"/api/v1/users/search"
"/admin/dashboard/stats"

// Less ideal: unclear or inconsistent
"/u/:id/p/:pid/c"
"/search/v1/user"
"/dashboard/admin/stats"
```

## Summary

Routing in Oxidite is straightforward and flexible:

- Use `.get()`, `.post()`, `.put()`, `.delete()`, etc. to register routes
- Extract path parameters with `Path<T>`
- Extract query parameters with `Query<T>`
- Organize routes logically and consistently
- Register specific routes before general/wildcard routes
- Apply middleware as needed for specific routes or groups

With these routing fundamentals, you can create well-structured applications that handle various types of requests effectively.