// Example: Demonstration of enhanced extractors in Oxidite
// Shows the new Form, Cookies, and Body extractors

use oxidite::prelude::*;
use serde::{Deserialize, Serialize};
use http_body_util::Full;
use bytes::Bytes;

#[derive(Debug, Deserialize, Serialize)]
struct User {
    id: u64,
    name: String,
    email: String,
}

#[derive(Debug, Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct RegistrationForm {
    name: String,
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct Pagination {
    page: Option<u32>,
    limit: Option<u32>,
}

// GET / - Hello world
async fn index(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    Ok(hyper::Response::new(Full::new(Bytes::from(
        r#"
        <!DOCTYPE html>
        <html>
        <head><title>Oxidite Enhanced Extractors Demo</title></head>
        <body>
            <h1>Oxidite Enhanced Extractors Demo</h1>
            <ul>
                <li><a href="/users">GET /users</a> - List users with query params</li>
                <li><a href="/login">GET /login</a> - Show login form</li>
                <li><a href="/webhook">GET /webhook</a> - Show webhook endpoint</li>
                <li><a href="/cookies">GET /cookies</a> - Show cookies endpoint</li>
            </ul>
            
            <form method="post" action="/register">
                <h2>Register User (Form Data)</h2>
                <input type="text" name="name" placeholder="Name" required><br>
                <input type="email" name="email" placeholder="Email" required><br>
                <input type="password" name="password" placeholder="Password" required><br>
                <button type="submit">Register</button>
            </form>
            
            <form method="post" action="/login" style="margin-top: 20px;">
                <h2>Login (Form Data)</h2>
                <input type="text" name="username" placeholder="Username" required><br>
                <input type="password" name="password" placeholder="Password" required><br>
                <button type="submit">Login</button>
            </form>
        </body>
        </html>
        "#
    )))
}

// GET /users - List users with pagination
async fn list_users(Query(params): Query<Pagination>) -> Result<OxiditeResponse> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(10);
    
    let users = vec![
        User {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        },
        User {
            id: 2,
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
        },
    ];

    let response_data = serde_json::json!({
        "users": users,
        "pagination": {
            "page": page,
            "limit": limit,
            "total": users.len()
        }
    });

    Ok(response::json(response_data))
}

// GET /login - Show login form
async fn show_login_form(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    Ok(hyper::Response::new(Full::new(Bytes::from(
        r#"
        <!DOCTYPE html>
        <html>
        <head><title>Login</title></head>
        <body>
            <h1>Login</h1>
            <form method="post" action="/login">
                <input type="text" name="username" placeholder="Username" required><br>
                <input type="password" name="password" placeholder="Password" required><br>
                <button type="submit">Login</button>
            </form>
        </body>
        </html>
        "#
    )))
}

// POST /login - Process login with form data
async fn process_login(Form(login_data): Form<LoginForm>) -> Result<OxiditeResponse> {
    // In a real app, you'd validate credentials here
    println!("Processing login for: {}", login_data.username);
    
    let response_data = serde_json::json!({
        "message": "Login successful",
        "user": {
            "username": login_data.username
        }
    });

    Ok(response::json(response_data))
}

// POST /register - Process registration with form data
async fn process_registration(Form(reg_data): Form<RegistrationForm>) -> Result<OxiditeResponse> {
    // In a real app, you'd create a user in the database
    println!("Processing registration for: {}", reg_data.name);
    
    let user = User {
        id: 3,
        name: reg_data.name,
        email: reg_data.email,
    };

    let response_data = serde_json::json!({
        "message": "Registration successful",
        "user": user
    });

    Ok(response::json(response_data))
}

// GET /webhook - Endpoint that receives raw body
async fn show_webhook_info(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    Ok(hyper::Response::new(Full::new(Bytes::from(
        r#"
        <!DOCTYPE html>
        <html>
        <head><title>Webhook Info</title></head>
        <body>
            <h1>Webhook Endpoint Info</h1>
            <p>This endpoint expects POST requests with raw body data.</p>
            <p>Try sending a POST request with some raw data to /webhook to see the Body extractor in action.</p>
        </body>
        </html>
        "#
    )))
}

// POST /webhook - Process webhook with raw body
async fn process_webhook(Body(raw_body): Body<String>) -> Result<OxiditeResponse> {
    println!("Received webhook with body length: {}", raw_body.len());
    println!("Raw body: {}", raw_body);
    
    let response_data = serde_json::json!({
        "message": "Webhook received",
        "body_length": raw_body.len(),
        "received_at": chrono::Utc::now().to_rfc3339()
    });

    Ok(response::json(response_data))
}

// GET /cookies - Show cookies
async fn show_cookies(cookies: Cookies) -> Result<OxiditeResponse> {
    let mut cookies_list = Vec::new();
    for (name, value) in cookies.iter() {
        cookies_list.push(serde_json::json!({
            "name": name,
            "value": value
        }));
    }

    let response_data = serde_json::json!({
        "message": "Cookies found",
        "cookies": cookies_list,
        "count": cookies_list.len()
    });

    Ok(response::json(response_data))
}

// POST /set-cookie - Set a cookie
async fn set_cookie(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    let mut response = hyper::Response::new(Full::new(Bytes::from(
        "Cookie set! Visit /cookies to see it."
    )));
    
    // Set a cookie in the response
    response.headers_mut().insert(
        "Set-Cookie",
        "test_cookie=test_value; Path=/; HttpOnly".parse().unwrap()
    );

    Ok(response)
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting Oxidite Enhanced Extractors Demo");
    println!("📍 Listening on http://127.0.0.1:3000");
    println!("📝 Available endpoints:");
    println!("   GET  /                    - Main demo page");
    println!("   GET  /users              - List users with query params");
    println!("   GET  /login              - Show login form");
    println!("   POST /login              - Process login (form data)");
    println!("   POST /register           - Process registration (form data)");
    println!("   GET  /webhook            - Webhook info");
    println!("   POST /webhook            - Process webhook (raw body)");
    println!("   GET  /cookies            - Show cookies");
    println!("   POST /set-cookie         - Set a test cookie");
    println!();

    let mut router = Router::new();
    
    // Register routes
    router.get("/", index);
    router.get("/users", list_users);
    router.get("/login", show_login_form);
    router.post("/login", process_login);
    router.post("/register", process_registration);
    router.get("/webhook", show_webhook_info);
    router.post("/webhook", process_webhook);
    router.get("/cookies", show_cookies);
    router.post("/set-cookie", set_cookie);

    // Start server
    Server::new(router)
        .listen("127.0.0.1:3000".parse().unwrap())
        .await?;

    Ok(())
}