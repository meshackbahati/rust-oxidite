use oxidite_core::{Request, Response, Error, State, FromRequest, html};
use crate::{AppState, models::{User, Post}};
use std::sync::Arc;
use oxidite_db::{Database,sqlx::Row};
use oxidite_template::Context;
use serde_json::json;

/// Show posts by a specific user
pub async fn user_posts(mut req: Request) -> Result<Response, Error> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    
    // Extract user_id from path (you would parse this from the request path)
    let path = req.uri().path();
    let parts: Vec<&str> = path.split('/').collect();
    let user_id = parts.get(2).ok_or(Error::BadRequest("Missing user ID".to_string()))?;
    
    // Get user details
    let user_query = format!("SELECT id, email, name, created_at FROM users WHERE id = '{}'", user_id.replace("'", "''"));
    let user_rows = state.db.query(&user_query).await
        .map_err(|e| Error::Server(format!("DB error: {}", e)))?;
    
    let user = if let Some(row) = user_rows.first() {
        User {
            id: row.try_get("id").unwrap_or(String::new()),
            email: row.try_get("email").unwrap_or(String::new()),
            name: row.try_get("name").unwrap_or(String::new()),
            created_at: row.try_get("created_at").ok(),
        }
    } else {
        return Err(Error::NotFound);
    };
    
    // Get user's posts
    let posts_query = format!("SELECT id, user_id, title, content, created_at FROM posts WHERE user_id = '{}' ORDER BY created_at DESC", user_id.replace("'", "''"));
    let rows = state.db.query(&posts_query).await
        .map_err(|e| Error::Server(format!("DB error: {}", e)))?;
        
    let mut posts = Vec::new();
    for row in rows {
        posts.push(Post {
            id: row.try_get("id").unwrap_or(String::new()),
            user_id: row.try_get("user_id").unwrap_or(String::new()),
            title: row.try_get("title").unwrap_or(String::new()),
            content: row.try_get("content").unwrap_or(String::new()),
            created_at: row.try_get("created_at").ok(),
        });
    }
    
    let context = Context::from_json(json!({
        "user": user,
        "posts": posts
    }));
    
    let rendered = state.templates.render("users/posts.html", &context)
        .map_err(|e| Error::Server(format!("Template error: {}", e)))?;
    
    Ok(html(rendered))
}
