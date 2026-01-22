# oxidite-openapi

OpenAPI/Swagger documentation generation for the Oxidite web framework.

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/oxidite-openapi.svg)](https://crates.io/crates/oxidite-openapi)
[![Docs.rs](https://docs.rs/oxidite-openapi/badge.svg)](https://docs.rs/oxidite-openapi)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

</div>

## Overview

`oxidite-openapi` provides automatic OpenAPI/Swagger documentation generation for Oxidite web applications. It allows you to generate comprehensive API documentation that follows the OpenAPI specification, making it easy for developers to understand and consume your APIs.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
oxidite-openapi = "0.2.0"
```

## Features

- **Automatic documentation generation** - Generate OpenAPI specs from your route definitions
- **Swagger UI integration** - Built-in Swagger UI for interactive API documentation
- **Schema inference** - Automatically infer schemas from Rust types
- **Parameter documentation** - Document query, path, and header parameters
- **Response documentation** - Describe response bodies and status codes
- **Request body documentation** - Document request schemas
- **Security schemes** - Document authentication methods
- **Custom annotations** - Fine-tune documentation with custom attributes
- **Multiple OpenAPI versions** - Support for OpenAPI 3.0 and 3.1

## Usage

### Basic Setup

Add OpenAPI documentation to your Oxidite application:

```rust
use oxidite::prelude::*;
use oxidite_openapi::{OpenApiSpec, OpenApiBuilder};

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    
    // Add your routes
    router.get("/users", get_users);
    router.post("/users", create_user);
    router.get("/users/:id", get_user);
    
    // Build OpenAPI specification
    let spec = OpenApiBuilder::new()
        .title("My API")
        .version("1.0.0")
        .description("My awesome API built with Oxidite")
        .contact_name("API Support")
        .contact_email("support@example.com")
        .license_name("MIT")
        .build();
    
    // Add route to serve OpenAPI spec
    router.get("/openapi.json", move |_| async move {
        Ok(response::json(spec.clone()))
    });
    
    Server::new(router)
        .listen("127.0.0.1:3000".parse().unwrap())
        .await
}
```

### Documenting Endpoints

Document your API endpoints with detailed information:

```rust
use oxidite::prelude::*;
use oxidite_openapi::{EndpointDoc, Parameter, Response};

// Document a GET endpoint
#[EndpointDoc(
    summary = "Get all users",
    description = "Returns a list of all users in the system",
    parameters = [
        Parameter::query("page", "integer", "Page number", Some("1")),
        Parameter::query("limit", "integer", "Items per page", Some("10"))
    ],
    responses = [
        Response::ok("List of users", "Vec<User>"),
        Response::bad_request("Invalid parameters")
    ]
)]
async fn get_users(
    Query(params): Query<PaginationParams>,
    State(db): State<Database>
) -> Result<OxiditeResponse> {
    let users = User::all(&db).limit(params.limit).offset((params.page - 1) * params.limit).await?;
    Ok(response::json(users))
}

// Document a POST endpoint
#[EndpointDoc(
    summary = "Create a new user",
    description = "Creates a new user in the system",
    request_body = "CreateUserRequest",
    responses = [
        Response::created("User created", "User"),
        Response::conflict("User already exists"),
        Response::unprocessable_entity("Validation error")
    ]
)]
async fn create_user(
    Json(payload): Json<CreateUserRequest>,
    State(db): State<Database>
) -> Result<OxiditeResponse> {
    // Validate payload
    if !is_valid_email(&payload.email) {
        return Err(OxiditeError::Validation("Invalid email format".to_string()));
    }
    
    let user = User {
        id: 0,
        name: payload.name,
        email: payload.email,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    user.create(&db).await?;
    Ok(response::json(user).status(http::StatusCode::CREATED))
}
```

### Schema Definitions

Define schemas for your API models:

```rust
use oxidite_openapi::Schema;

#[derive(Serialize, Deserialize, Schema)]
pub struct User {
    /// Unique identifier for the user
    pub id: i64,
    
    /// User's full name
    pub name: String,
    
    /// User's email address
    pub email: String,
    
    /// Timestamp when the user was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Timestamp when the user was last updated
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize, Schema)]
pub struct CreateUserRequest {
    /// User's full name (required)
    #[schema(min_length = 1, max_length = 100)]
    pub name: String,
    
    /// User's email address (required)
    #[schema(pattern = r"^[^@]+@[^@]+\.[^@]+$")]
    pub email: String,
}
```

### Security Documentation

Document authentication and authorization:

```rust
use oxidite_openapi::{SecurityScheme, ApiKeyLocation};

let spec = OpenApiBuilder::new()
    .title("My API")
    .version("1.0.0")
    // Add API key security scheme
    .security_scheme(
        "api_key",
        SecurityScheme::ApiKey {
            name: "X-API-Key".to_string(),
            location: ApiKeyLocation::Header,
            description: Some("API key for authentication".to_string()),
        }
    )
    .build();
```

### Serving Swagger UI

Serve the interactive Swagger UI with your documentation:

```rust
use oxidite::prelude::*;
use oxidite_openapi::{OpenApiSpec, SwaggerUi};

async fn setup_swagger_ui(router: &mut Router) {
    // First, generate your OpenAPI spec
    let spec = generate_openapi_spec();
    
    // Add the spec endpoint
    router.get("/openapi.json", move |_| async move {
        Ok(response::json(spec.clone()))
    });
    
    // Add Swagger UI
    let swagger_ui = SwaggerUi::new("/swagger-ui/")
        .spec_url("/openapi.json")
        .title("My API - Swagger UI");
    
    // Register Swagger UI routes
    router.get("/swagger-ui/", swagger_ui.handler());
    router.get("/swagger-ui/*", swagger_ui.handler());
}
```

### Custom Documentation Attributes

Fine-tune your API documentation:

```rust
use oxidite_openapi::{EndpointDoc, Parameter, Response, Schema};

#[EndpointDoc(
    operation_id = "get_user_by_id",
    summary = "Get user by ID",
    description = "Retrieve a specific user by their unique identifier",
    tags = ["users"],
    parameters = [
        Parameter::path("id", "integer", "User ID"),
    ],
    responses = [
        Response::ok("User details", "User"),
        Response::not_found("User not found"),
    ],
    deprecated = false
)]
async fn get_user(
    Path(params): Path<UserId>,
    State(db): State<Database>
) -> Result<OxiditeResponse> {
    match User::find(&db, params.id).await? {
        Some(user) => Ok(response::json(user)),
        None => Err(OxiditeError::NotFound("User not found".to_string())),
    }
}
```

### Validation and Testing

Validate your OpenAPI specification:

```rust
use oxidite_openapi::OpenApiValidator;

#[tokio::test]
async fn test_openapi_spec_validity() {
    let spec = generate_openapi_spec();
    
    // Validate the specification
    let validator = OpenApiValidator::new();
    let result = validator.validate(&spec);
    
    assert!(result.is_valid(), "OpenAPI spec should be valid");
    
    // Check for specific issues
    for warning in result.warnings() {
        eprintln!("Warning: {}", warning);
    }
}
```

## Integration with Oxidite

The OpenAPI integration works seamlessly with Oxidite's architecture:

```rust
use oxidite::prelude::*;
use oxidite_openapi::OpenApiBuilder;

async fn create_documented_app() -> Router {
    let mut router = Router::new();
    
    // Add your API routes
    router.get("/api/users", get_users);
    router.post("/api/users", create_user);
    router.get("/api/users/:id", get_user);
    router.put("/api/users/:id", update_user);
    router.delete("/api/users/:id", delete_user);
    
    // Generate OpenAPI documentation
    let spec = OpenApiBuilder::new()
        .title("User Management API")
        .version("1.0.0")
        .description("API for managing users")
        .server("https://api.example.com", "Production server")
        .server("https://staging-api.example.com", "Staging server")
        .build();
    
    // Add documentation endpoints
    router.get("/api/openapi.json", move |_| async move {
        Ok(response::json(spec.clone()))
    });
    
    // Add a catch-all for API docs
    router.get("/api/docs", |_req| async move {
        Ok(response::html(include_str!("../docs/swagger.html")))
    });
    
    router
}
```

## Best Practices

- **Document all public endpoints** - Every public API endpoint should have documentation
- **Use meaningful descriptions** - Provide clear, concise descriptions for all endpoints and parameters
- **Include examples** - Add request/response examples where helpful
- **Keep documentation up-to-date** - Update docs when changing API behavior
- **Validate your specs** - Regularly validate your OpenAPI specifications
- **Use appropriate HTTP status codes** - Document all possible response codes

## License

MIT