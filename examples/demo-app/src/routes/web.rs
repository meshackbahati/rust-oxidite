use oxidite_core::{Request, Response, Error, State, FromRequest, html};
use crate::{AppState, models::User};
use std::sync::Arc;
use oxidite_db::{Database, sqlx::Row};
use serde_json::json;
use oxidite_template::Context;

/// List all users (web page)
pub async fn list_users(mut req: Request) -> Result<Response, Error> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    
    // Fetch users from database
    let rows = state.db.query("SELECT id, email, name, created_at FROM users ORDER BY created_at DESC").await
        .map_err(|e| Error::Server(format!("DB error: {}", e)))?;
        
    let mut users = Vec::new();
    for row in rows {
        users.push(User {
            id: row.try_get("id").unwrap_or(String::new()),
            email: row.try_get("email").unwrap_or(String::new()),
            name: row.try_get("name").unwrap_or(String::new()),
            created_at: row.try_get("created_at").ok(),
        });
    }
    
    // Render template
    let context = Context::from_json(json!({
        "users": users
    }));
    
    let rendered = state.templates.render("users/list.html", &context)
        .map_err(|e| Error::Server(format!("Template error: {}", e)))?;
    
    Ok(html(rendered))
}

/// Show new user form
pub async fn new_user_form(mut req: Request) -> Result<Response, Error> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    
    let context = Context::from_json(json!({}));
    let rendered = state.templates.render("users/new.html", &context)
        .map_err(|e| Error::Server(format!("Template error: {}", e)))?;
    
    Ok(html(rendered))
}

