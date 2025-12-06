pub mod api;
pub mod api_v2;
pub mod auth;
pub mod realtime;
pub mod web;
pub mod posts;
pub mod user_posts;

use oxidite_core::{Request, Response, Error, html, State, FromRequest};
use oxidite_template::Context;
use serde_json::json;
use crate::AppState;
use std::sync::Arc;

/// Home page
pub async fn index(mut req: Request) -> Result<Response, Error> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    
    let context = Context::from_json(json!({}));
    let rendered = state.templates.render("home.html", &context)
        .map_err(|e| Error::Server(format!("Template error: {}", e)))?;
    
    Ok(html(rendered))
}

/// API Documentation page
pub async fn api_docs(mut req: Request) -> Result<Response, Error> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    
    // We pass the spec URL to the template
    let context = Context::from_json(json!({
        "spec_url": "/api/openapi.json"
    }));
    
    let rendered = state.templates.render("api_docs.html", &context)
        .map_err(|e| Error::Server(format!("Template error: {}", e)))?;
    
    Ok(html(rendered))
}

/// Serve OpenAPI Spec JSON
pub async fn openapi_spec(_req: Request) -> Result<Response, Error> {
    use oxidite_openapi::{OpenApiBuilder, Info, PathItem, Operation, Response as OpenApiResponse};
    
    let mut builder = OpenApiBuilder::new(
        "Oxidite Demo API",
        "1.0.0"
    ).description("API documentation for the Oxidite Demo Application");

    // Define /api/users
    builder = builder.path("/api/users", PathItem {
        get: Some(Operation {
            summary: Some("List all users".to_string()),
            description: Some("Returns a list of all registered users".to_string()),
            responses: {
                let mut map = std::collections::HashMap::new();
                map.insert("200".to_string(), OpenApiResponse {
                    description: "List of users".to_string(),
                    content: None, 
                });
                map
            },
            ..Default::default()
        }),
        post: None,
        ..Default::default()
    });

    // Define /api/posts
    builder = builder.path("/api/posts", PathItem {
        get: Some(Operation {
            summary: Some("List all posts".to_string()),
            description: Some("Returns a list of all posts".to_string()),
            responses: {
                let mut map = std::collections::HashMap::new();
                map.insert("200".to_string(), OpenApiResponse {
                    description: "List of posts".to_string(),
                    content: None,
                });
                map
            },
            ..Default::default()
        }),
        post: Some(Operation {
            summary: Some("Create a post".to_string()),
            description: Some("Creates a new post".to_string()),
            responses: {
                let mut map = std::collections::HashMap::new();
                map.insert("201".to_string(), OpenApiResponse {
                    description: "Post created".to_string(),
                    content: None,
                });
                map
            },
            ..Default::default()
        }),
        ..Default::default()
    });

    let spec = builder.build();
    Ok(oxidite_core::response::json(spec))
}

/// Favicon handler
pub async fn favicon(_req: Request) -> Result<Response, Error> {
    let content = std::fs::read_to_string("public/images/oxidite.svg")
        .map_err(|_| Error::NotFound)?;
        
    let mut response = Response::new(content.into());
    response.headers_mut().insert(
        "content-type",
        "image/svg+xml".parse().unwrap()
    );
    Ok(response)
}


/// 404 Not Found handler
pub async fn not_found(_req: Request) -> Result<Response, Error> {
    let html_content = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>404 - Not Found</title>
    <link rel="stylesheet" href="/public/css/style.css">
</head>
<body>
    <div class="container">
        <div class="empty-state" style="padding: 100px 20px;">
            <div class="empty-state-icon">🔍</div>
            <h2>404 - Page Not Found</h2>
            <p>The page you're looking for doesn't exist or has been moved.</p>
            <div style="margin-top: 30px; display: flex; gap: 15px; justify-content: center;">
                <a href="/" class="btn">🏠 Go Home</a>
                <a href="/users" class="btn btn-secondary">👥 View Users</a>
            </div>
        </div>
    </div>
</body>
</html>
    "#;
    
    Ok(html(html_content.to_string()))
}
