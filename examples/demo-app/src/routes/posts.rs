use oxidite_core::{Request, Response, Error, State, FromRequest, html, json, RequestExt};
use crate::{AppState, models::{User, Post}};
use std::sync::Arc;
use oxidite_db::{Database, sqlx::Row};
use serde_json::json;
use oxidite_template::Context;
use serde::Deserialize;

/// List all posts (web page)
pub async fn list_posts(mut req: Request) -> Result<Response, Error> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    
    let rows = state.db.query("SELECT id, user_id, title, content, created_at FROM posts ORDER BY created_at DESC").await
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
        "posts": posts
    }));
    
    let rendered = state.templates.render("posts/list.html", &context)
        .map_err(|e| Error::Server(format!("Template error: {}", e)))?;
    
    Ok(html(rendered))
}

/// Show a single post
pub async fn show_post(mut req: Request) -> Result<Response, Error> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    
    // Extract post ID from path
    let path = req.uri().path();
    let parts: Vec<&str> = path.split('/').collect();
    // Expected path: /posts/{id}
    let post_id = parts.get(2).ok_or(Error::BadRequest("Missing post ID".to_string()))?;
    
    let query = format!("SELECT id, user_id, title, content, created_at FROM posts WHERE id = '{}'", post_id.replace("'", "''"));
    let rows = state.db.query(&query).await
        .map_err(|e| Error::Server(format!("DB error: {}", e)))?;
        
    let post = if let Some(row) = rows.first() {
        Post {
            id: row.try_get("id").unwrap_or(String::new()),
            user_id: row.try_get("user_id").unwrap_or(String::new()),
            title: row.try_get("title").unwrap_or(String::new()),
            content: row.try_get("content").unwrap_or(String::new()),
            created_at: row.try_get("created_at").ok(),
        }
    } else {
        return Err(Error::NotFound);
    };
    
    let context = Context::from_json(json!({
        "post": post
    }));
    
    let rendered = state.templates.render("posts/show.html", &context)
        .map_err(|e| Error::Server(format!("Template error: {}", e)))?;
    
    Ok(html(rendered))
}

/// Show new post form
pub async fn new_post_form(mut req: Request) -> Result<Response, Error> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    
    let context = Context::from_json(json!({}));
    let rendered = state.templates.render("posts/new.html", &context)
        .map_err(|e| Error::Server(format!("Template error: {}", e)))?;
    
    Ok(html(rendered))
}

/// List all posts (API)
pub async fn api_list_posts(mut req: Request) -> Result<Response, Error> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    
    let rows = state.db.query("SELECT id, user_id, title, content, created_at FROM posts ORDER BY created_at DESC").await
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
    
    Ok(json(json!({
        "posts": posts,
        "total": posts.len()
    })))
}

/// Create a new post (API)
pub async fn api_create_post(mut req: Request) -> Result<Response, Error> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    let body = req.body_string().await?;
    
    #[derive(Deserialize)]
    struct CreatePostReq {
        user_id: String,
        title: String,
        content: String,
    }
    
    let post_req: CreatePostReq = serde_json::from_str(&body)
        .map_err(|e| Error::BadRequest(format!("Invalid JSON: {}", e)))?;
        
    let id = uuid::Uuid::new_v4().to_string();
    
    // Escape single quotes for SQL
    let user_id = post_req.user_id.replace("'", "''");
    let title = post_req.title.replace("'", "''");
    let content = post_req.content.replace("'", "''");
    
    let query = format!(
        "INSERT INTO posts (id, user_id, title, content) VALUES ('{}', '{}', '{}', '{}')",
        id, user_id, title, content
    );
    
    state.db.execute(&query).await
        .map_err(|e| Error::Server(format!("DB error: {}", e)))?;
    
    Ok(json(json!({
        "success": true,
        "message": "Post created",
        "post": {
            "id": id,
            "user_id": post_req.user_id,
            "title": post_req.title,
            "content": post_req.content
        }
    })))
}
