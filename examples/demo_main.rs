// Demo application showcasing Oxidite v2 features

use oxidite::prelude::*;
use oxidite_template::{TemplateEngine, Context};
use serde::Deserialize;

#[derive(Deserialize)]
struct QueryParams {
    page: Option<u32>,
    format: Option<String>,
}

// Home page with HTML response
async fn home(_req: Request) -> Result<Response> {
    Ok(Response::html(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Oxidite Demo</title>
        <style>
            body { font-family: Arial, sans-serif; margin: 40px; }
            .container { max-width: 800px; margin: 0 auto; }
            .feature { margin: 10px 0; padding: 10px; background: #f0f0f0; border-radius: 4px; }
        </style>
    </head>
    <body>
        <div class="container">
            <h1>Welcome to Oxidite v2 Demo!</h1>
            <p>This demonstrates the updated framework features.</p>
            
            <div class="feature">
                <h3>Response Types</h3>
                <ul>
                    <li><a href="/api">JSON Response</a></li>
                    <li><a href="/">HTML Response (this page)</a></li>
                    <li><a href="/text">Text Response</a></li>
                    <li><a href="/empty">Empty Response</a></li>
                </ul>
            </div>
            
            <div class="feature">
                <h3>Parameters</h3>
                <ul>
                    <li><a href="/users/123">Path Parameter</a></li>
                    <li><a href="/search?q=rust&page=1">Query Parameter</a></li>
                </ul>
            </div>
            
            <div class="feature">
                <h3>Error Handling</h3>
                <ul>
                    <li><a href="/error?type=not_found">404 Error</a></li>
                    <li><a href="/error?type=bad_request">400 Error</a></li>
                </ul>
            </div>
        </div>
    </body>
    </html>
    "#.to_string()))
}

// API endpoint with JSON response
async fn api_endpoint(_req: Request) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "message": "Hello from API",
        "framework": "Oxidite",
        "version": "2.0",
        "features": [
            "Type-safe request handling",
            "Multiple response types",
            "Template engine",
            "Path parameters",
            "Query parameters",
            "JSON body parsing",
            "Cookie handling",
            "Comprehensive error handling"
        ]
    })))
}

// Text response
async fn text_response(_req: Request) -> Result<Response> {
    Ok(Response::text("This is a plain text response from Oxidite v2!".to_string()))
}

// Empty response
async fn empty_response(_req: Request) -> Result<Response> {
    Ok(Response::ok())
}

// Path parameter example
async fn user_detail(Path(user_id): Path<u32>) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "user": {
            "id": user_id,
            "name": format!("User {}", user_id),
            "email": format!("user{}@example.com", user_id)
        }
    })))
}

// Query parameter example
async fn search_handler(Query(params): Query<QueryParams>) -> Result<Response> {
    let page = params.page.unwrap_or(1);
    let query = params.format.unwrap_or_else(|| "default".to_string());
    
    Ok(Response::json(serde_json::json!({
        "search_results": [],
        "query": query,
        "page": page,
        "total_results": 0
    })))
}

// Error handling example
async fn error_handler(Query(params): Query<serde_json::Value>) -> Result<Response> {
    if let Some(error_type) = params.get("type").and_then(|v| v.as_str()) {
        match error_type {
            "not_found" => Err(Error::NotFound("Resource not found".to_string())),
            "bad_request" => Err(Error::BadRequest("Bad request example".to_string())),
            "unauthorized" => Err(Error::Unauthorized("Unauthorized".to_string())),
            _ => Ok(Response::json(serde_json::json!({ "status": "unknown_error" })))
        }
    } else {
        Ok(Response::json(serde_json::json!({ "status": "no_error" })))
    }
}

// Template example
async fn template_example(_req: Request) -> Result<Response> {
    let mut template_engine = TemplateEngine::new();
    template_engine.add_template("demo", r#"
        <html>
        <head><title>{{ title }}</title></head>
        <body>
            <h1>{{ heading }}</h1>
            <p>{{ content }}</p>
            <ul>
            {% for item in items %}
                <li>{{ item }}</li>
            {% endfor %}
            </ul>
        </body>
        </html>
    "#).map_err(|e| Error::Server(e.to_string()))?;
    
    let mut context = Context::new();
    context.set("title", "Template Demo");
    context.set("heading", "Template Engine Example");
    context.set("content", "This page was rendered using the Oxidite template engine.");
    context.set("items", vec!["Feature 1", "Feature 2", "Feature 3"]);
    
    // Use the new render_response method
    let response = template_engine.render_response("demo", &context)
        .map_err(|e| Error::Server(e.to_string()))?;
    
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    
    // Main routes
    router.get("/", home);
    router.get("/api", api_endpoint);
    router.get("/text", text_response);
    router.get("/empty", empty_response);
    router.get("/users/:id", user_detail);
    router.get("/search", search_handler);
    router.get("/error", error_handler);
    router.get("/template", template_example);
    
    let server = Server::new(router);
    println!("🚀 Oxidite Demo running on http://127.0.0.1:3000");
    println!("📋 Visit http://127.0.0.1:3000 to see the demo");
    
    server.listen("127.0.0.1:3000".parse().unwrap()).await
}