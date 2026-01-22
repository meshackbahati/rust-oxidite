# Hello World

Let's start with the classic "Hello, World!" example to get familiar with Oxidite's basic concepts.

## The Simplest Application

Here's the most basic Oxidite application:

```rust
use oxidite::prelude::*;

async fn hello(_req: Request) -> Result<Response> {
    Ok(Response::text("Hello, World!"))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    router.get("/", hello);

    Server::new(router)
        .listen("127.0.0.1:3000".parse()?)
        .await
}
```

Let's break this down:

1. `use oxidite::prelude::*;` - Imports all the essential types and functions
2. `async fn hello(...)` - Defines a handler function that takes a request and returns a response
3. `_req: Request` - The incoming request (we use `_` since we don't use it)
4. `Ok(Response::text(...))` - Creates a text response
5. `Result<Response>` - The handler returns a Result with either a Response or an Error
6. `Router::new()` - Creates a new router to define routes
7. `router.get("/", hello)` - Registers the hello function to handle GET requests to "/"
8. `Server::new(router)` - Creates a server with the configured router
9. `.listen(...)` - Starts the server on port 3000

## Different Response Types

Let's explore different ways to respond:

### JSON Response
```rust
async fn api_hello(_req: Request) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "message": "Hello, World!",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}
```

### HTML Response
```rust
async fn html_hello(_req: Request) -> Result<Response> {
    Ok(Response::html(r#"
        <!DOCTYPE html>
        <html>
        <head><title>Hello</title></head>
        <body>
            <h1>Hello, World!</h1>
            <p>Welcome to Oxidite!</p>
        </body>
        </html>
    "#.to_string()))
}
```

### Different Routes
```rust
use oxidite::prelude::*;

async fn home(_req: Request) -> Result<Response> {
    Ok(Response::text("Welcome to the home page!"))
}

async fn about(_req: Request) -> Result<Response> {
    Ok(Response::text("About us page"))
}

async fn contact(_req: Request) -> Result<Response> {
    Ok(Response::text("Contact information"))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    
    router.get("/", home);
    router.get("/about", about);
    router.get("/contact", contact);

    Server::new(router)
        .listen("127.0.0.1:3000".parse()?)
        .await
}
```

## Using Path Parameters

Oxidite supports path parameters that you can extract:

```rust
use oxidite::prelude::*;

async fn greet(Path(name): Path<String>) -> Result<Response> {
    Ok(Response::text(format!("Hello, {}!", name)))
}

async fn user_details(Path(user_id): Path<u32>) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "user_id": user_id,
        "name": format!("User {}", user_id),
        "active": true
    })))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    
    // Path parameter: /greet/Alice will extract "Alice"
    router.get("/greet/:name", greet);
    
    // Numeric parameter: /users/123 will extract 123
    router.get("/users/:user_id", user_details);

    Server::new(router)
        .listen("127.0.0.1:3000".parse()?)
        .await
}
```

## Using Query Parameters

You can also extract query parameters:

```rust
use oxidite::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct GreetingParams {
    name: Option<String>,
    title: Option<String>,
}

async fn personalized_greeting(Query(params): Query<GreetingParams>) -> Result<Response> {
    let name = params.name.unwrap_or_else(|| "World".to_string());
    let title = params.title.unwrap_or_else(|| "".to_string());
    
    let greeting = if title.is_empty() {
        format!("Hello, {}!", name)
    } else {
        format!("Hello, {} {}!", title, name)
    };
    
    Ok(Response::text(greeting))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    router.get("/greet", personalized_greeting);

    Server::new(router)
        .listen("127.0.0.1:3000".parse()?)
        .await
}

// This handles URLs like:
// /greet?name=Alice
// /greet?name=Bob&title=Mr.
```

## Using Request Body

For POST requests, you can extract JSON data:

```rust
use oxidite::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

async fn create_user(Json(payload): Json<CreateUser>) -> Result<Response> {
    // Process the payload...
    Ok(Response::json(serde_json::json!({
        "message": "User created successfully",
        "user": {
            "id": 123, // In a real app, this would come from your database
            "name": payload.name,
            "email": payload.email
        }
    })))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    router.post("/users", create_user);

    Server::new(router)
        .listen("127.0.0.1:3000".parse()?)
        .await
}
```

## Error Handling

Let's add some error handling:

```rust
use oxidite::prelude::*;

async fn maybe_error(query: Query<serde_json::Value>) -> Result<Response> {
    let should_error = query.0.get("error").and_then(|v| v.as_bool()).unwrap_or(false);
    
    if should_error {
        return Err(Error::BadRequest("Something went wrong".to_string()));
    }
    
    Ok(Response::text("Success!"))
}

async fn not_found_handler(_req: Request) -> Result<Response> {
    Err(Error::NotFound)
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    
    router.get("/maybe-error", maybe_error);
    router.get("/not-found", not_found_handler);

    Server::new(router)
        .listen("127.0.0.1:3000".parse()?)
        .await
}
```

## Complete Example with Multiple Features

Here's a more complete example combining multiple features:

```rust
use oxidite::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct UserQuery {
    limit: Option<u32>,
    offset: Option<u32>,
}

async fn home(_req: Request) -> Result<Response> {
    Ok(Response::html(r#"
        <!DOCTYPE html>
        <html>
        <head><title>Oxidite Demo</title></head>
        <body>
            <h1>Welcome to Oxidite!</h1>
            <nav>
                <a href="/api/hello">API Hello</a> |
                <a href="/users?page=1">Users API</a> |
                <a href="/greet/World">Greet Route</a>
            </nav>
        </body>
        </html>
    "#.to_string()))
}

async fn api_hello(_req: Request) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "message": "Hello from API",
        "framework": "Oxidite",
        "version": "2.0"
    })))
}

async fn get_users(Query(params): Query<UserQuery>) -> Result<Response> {
    let limit = params.limit.unwrap_or(10);
    let offset = params.offset.unwrap_or(0);
    
    Ok(Response::json(serde_json::json!({
        "users": [
            {"id": 1, "name": "Alice", "email": "alice@example.com"},
            {"id": 2, "name": "Bob", "email": "bob@example.com"}
        ],
        "pagination": {
            "limit": limit,
            "offset": offset,
            "total": 2
        }
    })))
}

async fn greet_user(Path(name): Path<String>) -> Result<Response> {
    Ok(Response::text(format!("Hello, {}!", name)))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    
    router.get("/", home);
    router.get("/api/hello", api_hello);
    router.get("/users", get_users);
    router.get("/greet/:name", greet_user);

    Server::new(router)
        .listen("127.0.0.1:3000".parse()?)
        .await
}
```

## Running Your Application

To run any of these examples:

1. Create a new Rust project: `cargo new hello-oxidite`
2. Add Oxidite to your `Cargo.toml`:
   ```toml
   [dependencies]
   oxidite = { version = "2.0", features = ["full"] }
   tokio = { version = "1.0", features = ["full"] }
   serde = { version = "1.0", features = ["derive"] }
   serde_json = "1.0"
   chrono = { version = "0.4", features = ["serde"] }
   ```
3. Replace the contents of `src/main.rs` with your example
4. Run with `cargo run`
5. Visit `http://127.0.0.1:3000` in your browser

This Hello World example demonstrates the fundamental concepts of Oxidite: handlers, routes, responses, and request data extraction. These concepts form the foundation for building more complex applications.