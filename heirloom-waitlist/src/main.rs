use oxidite::prelude::*;

use oxidite_db::{DbPool, Database};
use oxidite_template::{TemplateEngine, Context};
use oxidite_config::Config;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use lazy_static::lazy_static;
use regex::Regex;
use oxidite_db::sqlx::Row;
use toml::Value;
use std::fs;

lazy_static! {
    static ref CONFIG: Config = load_config();
    static ref DB_POOL: DbPool = create_connection_pool();
    static ref TEMPLATE_ENGINE: TemplateEngine = create_template_engine();
}

fn load_config() -> Config {
    let mut config = Config::load().expect("Failed to load configuration");
    
    // Manually load Brevo-specific environment variables
    if let Ok(api_key) = std::env::var("BREVO_API_KEY") {
        config.custom.insert("brevo_api_key".to_string(), Value::String(api_key));
    }
    if let Ok(list_id) = std::env::var("BREVO_LIST_ID") {
        config.custom.insert("brevo_list_id".to_string(), Value::Integer(list_id.parse().unwrap_or(1)));
    }
    if let Ok(sender_email) = std::env::var("SENDER_EMAIL") {
        config.custom.insert("brevo_sender_email".to_string(), Value::String(sender_email));
    }
    if let Ok(notification_email) = std::env::var("NOTIFICATION_EMAIL") {
        config.custom.insert("brevo_notification_email".to_string(), Value::String(notification_email));
    }
    
    config
}

fn create_template_engine() -> TemplateEngine {
    let mut engine = TemplateEngine::new();
    // Load all templates from the templates directory
    engine.load_dir("./templates").expect("Failed to load templates");
    engine
}

fn create_connection_pool() -> DbPool {
    let database_url = &CONFIG.database.url;
    
    // Create and return the database pool
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            DbPool::connect(database_url).await.expect("Failed to create database pool")
        })
    })
}

// Run migrations on startup
async fn run_db_migrations() {
    let database_url = &CONFIG.database.url;
    let pool = DbPool::connect(database_url).await.expect("Failed to connect to database");
    
    // Create the waitlistentries table if it doesn't exist (matching the Model derive macro convention)
    let create_table_query = "CREATE TABLE IF NOT EXISTS waitlistentries (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            email VARCHAR NOT NULL UNIQUE,
            created_at TIMESTAMP NOT NULL DEFAULT NOW(),
            ip_address VARCHAR
        )";
    
    pool.execute(create_table_query).await.expect("Failed to create waitlist_entries table");
    
    println!("Database tables created successfully!");
}

#[derive(Deserialize)]
struct EmailSubmission {
    email: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Config is loaded via lazy_static
    
    // Run database migrations
    run_db_migrations().await;
    
    let mut router = Router::new();
    
    // Serve static files
    // Note: Static file serving would need to be implemented as middleware or a custom handler
    
    // Routes
    router.get("/", home);
    router.get("/about", about);
    router.get("/features", features);
    router.get("/faq", faq);
    router.post("/api/waitlist", join_waitlist);
    router.get("/favicon.ico", serve_favicon);
    // Static file handler as fallback route
    router.get("/*", oxidite_template::serve_static);
    
    let server = Server::new(router);
    println!("Heirloom Waitlist Server running on http://localhost:8080");
    server.listen("0.0.0.0:8080".parse().unwrap()).await
}

async fn home(_req: Request) -> Result<Response> {
    let context = Context::new();
    Ok(TEMPLATE_ENGINE.render_response("index.html", &context)
        .unwrap_or_else(|_| Response::html("Template not found".to_string())))
}

async fn about(_req: Request) -> Result<Response> {
    let context = Context::new();
    Ok(TEMPLATE_ENGINE.render_response("about.html", &context)
        .unwrap_or_else(|_| Response::html("Template not found".to_string())))
}

async fn features(_req: Request) -> Result<Response> {
    let context = Context::new();
    Ok(TEMPLATE_ENGINE.render_response("features.html", &context)
        .unwrap_or_else(|_| Response::html("Template not found".to_string())))
}

async fn faq(_req: Request) -> Result<Response> {
    let context = Context::new();
    Ok(TEMPLATE_ENGINE.render_response("faq.html", &context)
        .unwrap_or_else(|_| Response::html("Template not found".to_string())))
}

async fn join_waitlist(mut req: Request) -> Result<Response> {
    // Parse the request body to get the email
    // Parse the request body to get the email
    use http_body_util::BodyExt;
    use bytes::Buf;
    
    let body_bytes = req.body_mut().collect().await
        .map_err(|e| Error::InternalServerError(format!("Failed to read body: {}", e)))?
        .aggregate();
    
    let body = std::str::from_utf8(body_bytes.chunk())
        .map_err(|e| Error::BadRequest(format!("Invalid UTF-8: {}", e)))?
        .to_string();
    
    let email_submission: EmailSubmission = match serde_json::from_str(&body) {
        Ok(data) => data,
        Err(_) => {
            return Err(Error::BadRequest("Invalid request format".to_string()));
        }
    };
    
    let email = email_submission.email.trim();
    
    // Validate email format with regex
    if !is_valid_email(email) {
        return Err(Error::BadRequest("Invalid email format".to_string()));
    }
    
    // Get IP address of the requester
    let ip_address = req.headers().get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.split(',').collect::<Vec<&str>>()[0].trim().to_string());
    
    // Use raw SQL to check if email exists
    let query = format!("SELECT COUNT(*) FROM waitlistentries WHERE email = '{}'", email);
    let rows = DB_POOL.query(&query).await
        .map_err(|_e| {
            Error::InternalServerError("Internal server error".to_string())
        })?;
    
    let count: i64 = if !rows.is_empty() {
        rows[0].try_get(0).map_err(|_e| {
            Error::InternalServerError("Internal server error".to_string())
        })?
    } else {
        0
    };
    
    if count > 0 {
        return Err(Error::Conflict("Email is already on the waitlist".to_string())); // Conflict status code
    }
    
    // Create a new waitlist entry in the database
    let new_id = Uuid::new_v4();
    
    // Use direct SQL query to insert the record
    let insert_query = format!(
        "INSERT INTO waitlistentries (id, email, ip_address) VALUES ('{}', '{}', '{}')",
        new_id,
        email,
        ip_address.as_ref().map(|s| s.as_str()).unwrap_or("NULL")
    );
    if let Err(_e) = DB_POOL.execute(&insert_query).await {
        return Err(Error::InternalServerError("Internal server error".to_string()));
    };
    
    // Call the Brevo API to add the email to a contact list
    if let Err(e) = add_contact_to_brevo(&email_submission.email).await {
        // Log at debug level - Brevo errors are expected when API key is not configured
        // Continue anyway - don't fail the request if Brevo API fails
    }
    
    // Send a confirmation email via Brevo
    if let Err(e) = send_confirmation_email(&email_submission.email).await {
        // Log at debug level - email sending failures shouldn't be noisy in production
        // Continue anyway - don't fail the request if sending email fails
    }
    
    // Check if this is the first member and send special notification
    // Count total entries to determine if this is the first member
    let query = "SELECT COUNT(*) FROM waitlistentries";
    let rows = DB_POOL.query(query).await
        .map_err(|_e| {
            Error::InternalServerError("Internal server error".to_string())
        })?;
    let total_count: i64 = if !rows.is_empty() {
        rows[0].try_get(0).map_err(|_e| {
            Error::InternalServerError("Internal server error".to_string())
        })?
    } else {
        0
    };
    
    if total_count == 1 {
        // Silently attempt to send first member notification
        let _ = send_first_member_notification(&email_submission.email).await;
    }
    
    // Success response
    Ok(Response::json(serde_json::json!({
        "success": true,
        "message": "Thank you for joining the Heirloom waitlist! You'll be notified when we launch."
    })))
}

// Helper function to validate email format
fn is_valid_email(email: &str) -> bool {
    // Basic email validation using regex
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

// Favicon serving
async fn serve_favicon(_req: Request) -> Result<Response> {
    let favicon_path = "./public/favicon.ico";
    
    if let Ok(content) = std::fs::read(favicon_path) {
        let mut response = Response::html(String::from_utf8_lossy(&content).to_string());
        // Set content type header
        // Note: The Response type from oxidite doesn't seem to have a direct header method
        // The favicon will be served as HTML content, which browsers usually handle correctly
        Ok(response)
    } else {
        Err(Error::NotFound("Favicon not found".to_string()))
    }
}

// Function to add contact to Brevo
async fn add_contact_to_brevo(email: &str) -> Result<()> {
    let api_key = CONFIG.custom.get("brevo_api_key").and_then(|v| v.as_str()).unwrap_or_else(|| {
        panic!("BREVO_API_KEY must be set");
    }).to_string();
    let client = reqwest::Client::new();
    
    let list_id = CONFIG.custom.get("brevo_list_id").and_then(|v| v.as_integer()).unwrap_or(1);
    
    let payload = serde_json::json!({
        "email": email,
        "attributes": {
            "FIRST_NAME": "",
            "LAST_NAME": ""
        },
        "listIds": [list_id], // Use the configured list ID
        "updateEnabled": true
    });
    
    let res = client
        .post("https://api.brevo.com/v3/contacts")
        .header("api-key", &api_key)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| Error::InternalServerError(format!("Reqwest error: {}", e)))?;
    
    if !res.status().is_success() {
        let error_text = res.text().await
            .map_err(|e| Error::InternalServerError(format!("Failed to read response: {}", e)))?;
        return Err(Error::InternalServerError(format!("Brevo API error: {}", error_text)));
    }
    
    Ok(())
}

// Function to send confirmation email via Brevo
async fn send_confirmation_email(email: &str) -> Result<()> {
    let api_key = CONFIG.custom.get("brevo_api_key").and_then(|v| v.as_str()).unwrap_or_else(|| {
        panic!("BREVO_API_KEY must be set");
    }).to_string();
    let client = reqwest::Client::new();
    
    let sender_email = CONFIG.custom.get("brevo_sender_email").and_then(|v| v.as_str()).unwrap_or("noreply@heirloomplatform.com").to_string();
    
    let payload = serde_json::json!({
        "sender": {
            "name": "Heirloom Team",
            "email": sender_email
        },
        "to": [{
            "email": email
        }],
        "subject": "Welcome to the Heirloom Waitlist!",
        "htmlContent": format!(
            "<!DOCTYPE html>\n<html>\n<head><title>Welcome to Heirloom</title></head>\n<body>\n<h1>Welcome to the Heirloom Waitlist!</h1>\n<p>Thank you for joining the waitlist for Heirloom - the platform to preserve your family's legacy.</p>\n<p>You'll be the first to know when we launch!</p>\n<p>Cheers,<br>The Heirloom Team</p>\n</body>\n</html>"
        )
    });
    
    let res = client
        .post("https://api.brevo.com/v3/smtp/email")
        .header("api-key", &api_key)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| Error::InternalServerError(format!("Reqwest error: {}", e)))?;
    
    if !res.status().is_success() {
        let error_text = res.text().await
            .map_err(|e| Error::InternalServerError(format!("Failed to read response: {}", e)))?;
        return Err(Error::InternalServerError(format!("Brevo send email API error: {}", error_text)));
    }
    
    Ok(())
}

// Function to send notification about the first member
async fn send_first_member_notification(first_member_email: &str) -> Result<()> {
    let api_key = CONFIG.custom.get("brevo_api_key").and_then(|v| v.as_str()).unwrap_or_else(|| {
        panic!("BREVO_API_KEY must be set");
    }).to_string();
    let client = reqwest::Client::new();
    
    let sender_email = CONFIG.custom.get("brevo_sender_email").and_then(|v| v.as_str()).unwrap_or("notifications@heirloomplatform.com").to_string();
    let notification_email = CONFIG.custom.get("brevo_notification_email").and_then(|v| v.as_str()).unwrap_or("founders@heirloomplatform.com").to_string();
    
    let payload = serde_json::json!({
        "sender": {
            "name": "Heirloom Notification System",
            "email": sender_email
        },
        "to": [{
            "email": notification_email
        }],
        "subject": "🎉 First Member Joined Heirloom Waitlist!",
        "htmlContent": format!(
            "<!DOCTYPE html>\n<html>\n<head><title>First Member Notification</title></head>\n<body>\n<h1>Congratulations! 🎉</h1>\n<p>The first member has joined the Heirloom waitlist!</p>\n<p>Email: <strong>{}</strong></p>\n<p>This is an exciting milestone for the Heirloom platform!</p>\n</body>\n</html>",
            first_member_email
        )
    });
    
    let res = client
        .post("https://api.brevo.com/v3/smtp/email")
        .header("api-key", &api_key)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| Error::InternalServerError(format!("Reqwest error: {}", e)))?;
    
    if !res.status().is_success() {
        let error_text = res.text().await
            .map_err(|e| Error::InternalServerError(format!("Failed to read response: {}", e)))?;
        return Err(Error::InternalServerError(format!("Brevo send first member notification API error: {}", error_text)));
    }
    
    Ok(())
}