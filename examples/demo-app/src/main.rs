// Demo Application showcasing Oxidite v2 features

use oxidite::prelude::*;
use oxidite_template::{TemplateEngine, Context};
use serde::Deserialize;
use std::sync::{Arc, Mutex};

#[derive(Deserialize)]
struct QueryParams {
    page: Option<u32>,
    limit: Option<u32>,
}

#[derive(Clone)]
struct AppState {
    template_engine: Arc<Mutex<TemplateEngine>>,
    visit_count: Arc<Mutex<u32>>,
}

// Home page with HTML response
async fn home(state: State<AppState>) -> Result<Response> {
    let mut visit_count = state.visit_count.lock().unwrap();
    *visit_count += 1;
    let count = *visit_count;
    drop(visit_count); // Release the lock early

    let mut context = Context::new();
    context.set("title", "Oxidite Demo App");
    context.set("welcome_message", "Welcome to the Oxidite v2 Demo!");
    context.set("visit_count", count);
    context.set("features", vec![
        "Type-safe Request Handling",
        "Multiple Response Types",
        "Template Engine",
        "Path Parameters",
        "Query Parameters",
        "JSON Body Parsing",
        "Cookie Handling",
        "Comprehensive Error Handling"
    ]);

    // Use the template engine's render_response method
    let template_engine = state.template_engine.lock().unwrap();
    let response = template_engine.render_response("home.html", &context)
        .map_err(|e| Error::Server(e.to_string()))?;
    
    Ok(response)
}

// API endpoint with JSON response
async fn api_status(_req: Request) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "status": "online",
        "framework": "Oxidite",
        "version": "2.0",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "uptime": "running"
    })))
}

// Users API endpoint
async fn get_users(Query(params): Query<QueryParams>) -> Result<Response> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(10);
    let offset = (page - 1) * limit;

    // Simulated user data
    let users: Vec<_> = ((offset + 1)..=(offset + limit))
        .map(|i| serde_json::json!({
            "id": i,
            "name": format!("User {}", i),
            "email": format!("user{}@example.com", i),
            "active": true
        }))
        .collect();

    Ok(Response::json(serde_json::json!({
        "users": users,
        "pagination": {
            "page": page,
            "limit": limit,
            "offset": offset,
            "total": 100 // Simulated total
        }
    })))
}

// User detail endpoint with path parameter
async fn get_user_detail(Path(user_id): Path<u32>) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "user": {
            "id": user_id,
            "name": format!("User {}", user_id),
            "email": format!("user{}@example.com", user_id),
            "joined_date": chrono::Utc::now().date_naive(),
            "active": true
        }
    })))
}

// Form endpoint to demonstrate body parsing
#[derive(Deserialize)]
struct UserFormData {
    name: String,
    email: String,
    message: String,
}

async fn submit_form(Json(form_data): Json<UserFormData>) -> Result<Response> {
    // Process the form data
    println!("Received form data: {:?}", form_data);
    
    Ok(Response::json(serde_json::json!({
        "status": "success",
        "message": "Form submitted successfully",
        "received_data": {
            "name": form_data.name,
            "email": form_data.email,
            "message_length": form_data.message.len()
        }
    })))
}

// Cookie handling example
async fn handle_cookies(cookies: Cookies) -> Result<Response> {
    let mut response_data = serde_json::json!({
        "message": "Cookie information retrieved",
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

// Error handling example
async fn error_example(Query(params): Query<serde_json::Value>) -> Result<Response> {
    if let Some(error_type) = params.get("type").and_then(|v| v.as_str()) {
        match error_type {
            "not_found" => Err(Error::NotFound),
            "bad_request" => Err(Error::BadRequest("Bad request example".to_string())),
            "unauthorized" => Err(Error::Unauthorized("Unauthorized access".to_string())),
            "forbidden" => Err(Error::Forbidden("Access forbidden".to_string())),
            "conflict" => Err(Error::Conflict("Resource conflict".to_string())),
            "validation" => Err(Error::Validation("Validation failed".to_string())),
            "rate_limited" => Err(Error::RateLimited),
            "service_unavailable" => Err(Error::ServiceUnavailable("Service temporarily unavailable".to_string())),
            _ => Ok(Response::json(serde_json::json!({ "status": "unknown_error_type" })))
        }
    } else {
        Ok(Response::json(serde_json::json!({ "status": "no_error_requested" })))
    }
}

// Text response example
async fn health_check(_req: Request) -> Result<Response> {
    Ok(Response::text("OK".to_string()))
}

// Empty response example
async fn ping(_req: Request) -> Result<Response> {
    Ok(Response::ok())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Set up template engine
    let mut template_engine = TemplateEngine::new();
    
    // Add home template
    template_engine.add_template("home.html", r#"
<!DOCTYPE html>
<html>
<head>
    <title>{{ title }}</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        body { 
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; 
            max-width: 800px; 
            margin: 0 auto; 
            padding: 20px; 
            background-color: #f8f9fa;
            color: #333;
        }
        header { 
            text-align: center; 
            padding: 2rem 0; 
            border-bottom: 2px solid #007acc; 
            margin-bottom: 2rem;
        }
        h1 { color: #007acc; }
        .stats { 
            display: flex; 
            justify-content: space-around; 
            margin: 2rem 0; 
            flex-wrap: wrap;
        }
        .stat-card { 
            background: white; 
            padding: 1rem; 
            border-radius: 8px; 
            box-shadow: 0 2px 4px rgba(0,0,0,0.1); 
            min-width: 150px;
            text-align: center;
        }
        .features { 
            background: white; 
            padding: 1.5rem; 
            border-radius: 8px; 
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            margin-top: 2rem;
        }
        ul { 
            list-style-type: none; 
            padding: 0; 
        }
        li { 
            padding: 0.5rem; 
            margin: 0.25rem 0; 
            background: #e9ecef; 
            border-radius: 4px;
        }
        .api-links { 
            margin-top: 2rem; 
            padding-top: 1rem; 
            border-top: 1px solid #dee2e6;
        }
        .api-links a { 
            display: inline-block; 
            margin: 0.25rem; 
            padding: 0.5rem 1rem; 
            background: #007acc; 
            color: white; 
            text-decoration: none; 
            border-radius: 4px;
        }
        .api-links a:hover { 
            background: #005a9e; 
        }
    </style>
</head>
<body>
    <header>
        <h1>{{ welcome_message }}</h1>
        <p>Demo application showcasing Oxidite v2 features</p>
    </header>
    
    <div class="stats">
        <div class="stat-card">
            <h3>Visits</h3>
            <p>{{ visit_count }}</p>
        </div>
        <div class="stat-card">
            <h3>Framework</h3>
            <p>Oxidite v2</p>
        </div>
        <div class="stat-card">
            <h3>Status</h3>
            <p>Running</p>
        </div>
    </div>
    
    <div class="features">
        <h2>Framework Features</h2>
        <ul>
        {% for feature in features %}
            <li>{{ feature }}</li>
        {% endfor %}
        </ul>
    </div>
    
    <div class="api-links">
        <h3>Try the API Endpoints:</h3>
        <a href="/api/status" target="_blank">API Status</a>
        <a href="/users?page=1&limit=5" target="_blank">Users API</a>
        <a href="/users/123" target="_blank">User Detail</a>
        <a href="/health" target="_blank">Health Check</a>
        <a href="/ping" target="_blank">Ping</a>
        <a href="/error?type=not_found" target="_blank">Error Example</a>
        <a href="/cookies" target="_blank">Cookies</a>
    </div>
</body>
</html>
    "#)?;

    let app_state = AppState {
        template_engine: Arc::new(Mutex::new(template_engine)),
        visit_count: Arc::new(Mutex::new(0)),
    };

    let mut router = Router::new();

    // Main routes
    router.get("/", {
        let state = app_state.clone();
        move |_| home(State(state))
    });
    
    // API routes
    router.get("/api/status", api_status);
    router.get("/users", get_users);
    router.get("/users/:id", get_user_detail);
    
    // Other routes
    router.post("/submit", submit_form);
    router.get("/cookies", handle_cookies);
    router.get("/error", error_example);
    router.get("/health", health_check);
    router.get("/ping", ping);

    let server = Server::new(router);
    println!("🚀 Oxidite Demo App running on http://127.0.0.1:3000");
    println!("");
    println!("📋 Available Endpoints:");
    println!("   GET  /                    - Home page (HTML)");
    println!("   GET  /api/status          - API status (JSON)");
    println!("   GET  /users               - Users list (JSON)");
    println!("   GET  /users/:id           - User detail (JSON)");
    println!("   POST /submit              - Form submission (JSON body)");
    println!("   GET  /cookies             - Cookie handling (JSON)");
    println!("   GET  /error               - Error examples");
    println!("   GET  /health              - Health check (text)");
    println!("   GET  /ping                - Ping (empty response)");
    println!("");
    println!("💡 Tips:");
    println!("   - Visit the home page to see the template engine in action");
    println!("   - Try /users?page=1&limit=3 to see query parameters");
    println!("   - Try /error?type=not_found to see error handling");
    println!("   - Submit JSON to /submit to see body parsing");

    server.listen("127.0.0.1:3000".parse().unwrap()).await
}