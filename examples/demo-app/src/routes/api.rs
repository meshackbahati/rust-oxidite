use oxidite_core::{Request, Response, Error, json, RequestExt, State, FromRequest};
use serde::{Deserialize, Serialize};
use crate::AppState;
use std::sync::Arc;
use oxidite_db::{Database, sqlx::Row};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
}

/// List all users
pub async fn list_users(mut req: Request) -> Result<Response, Error> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    
    let rows = state.db.query("SELECT id, email, name FROM users").await
        .map_err(|e| Error::Server(format!("DB error: {}", e)))?;
        
    let mut users = Vec::new();
    for row in rows {
        users.push(User {
            id: row.try_get("id").unwrap_or(String::new()),
            email: row.try_get("email").unwrap_or(String::new()),
            name: row.try_get("name").unwrap_or(String::new()),
        });
    }
    
    Ok(json(serde_json::json!({
        "users": users,
        "total": users.len()
    })))
}

/// Create a new user
pub async fn create_user(mut req: Request) -> Result<Response, Error> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    let body = req.body_string().await?;
    
    #[derive(Deserialize)]
    struct CreateUserReq {
        email: String,
        name: String,
    }
    
    let user_req: CreateUserReq = serde_json::from_str(&body)
        .map_err(|e| Error::BadRequest(format!("Invalid JSON: {}", e)))?;
        
    let id = uuid::Uuid::new_v4().to_string();
    
    // Simple SQL injection protection for demo
    let email = user_req.email.replace("'", "''");
    let name = user_req.name.replace("'", "''");
    
    let query = format!(
        "INSERT INTO users (id, email, name) VALUES ('{}', '{}', '{}')",
        id, email, name
    );
    
    state.db.execute(&query).await
        .map_err(|e| Error::Server(format!("DB error: {}", e)))?;
    
    Ok(json(serde_json::json!({
        "success": true,
        "message": "User created",
        "user": {
            "id": id,
            "email": user_req.email,
            "name": user_req.name
        }
    })))
}
