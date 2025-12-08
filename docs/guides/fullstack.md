# Fullstack Web Development with Oxidite

Build complete fullstack applications with server-side rendering, authentication, and database integration.

## Project Setup

The best way to start a fullstack project is with the `oxidite-cli`:

```bash
oxidite new my-blog --project-type fullstack
cd my-blog
```

This will generate a project with a complete structure, including directories for templates, static files, and database models.

## Complete Example: Blog Application

### Main Application (`src/main.rs`)

```rust
use oxidite::prelude::*;
use oxidite_db::Database;
use oxidite_template::TemplateEngine;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    db: Arc<Database>,
    templates: Arc<TemplateEngine>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Database
    let db = Arc::new(Database::connect(&std::env::var("DATABASE_URL")?).await?);
    
    // Template engine
    let templates = Arc::new(TemplateEngine::new("templates"));

    let state = AppState { db, templates };
    
    // Routes
    let mut app = Router::new();
    
    // Public routes
    app.get("/", home);
    app.get("/posts/:id", show_post);
    
    // ... more routes
    
    // Middleware and state
    let app = ServiceBuilder::new()
        .layer(LoggerLayer)
        .layer(CorsLayer::permissive())
        .state(state)
        .service(app);
    
    Server::new(app).listen("127.0.0.1:3000".parse()?).await
}
```

### Models (`src/models/post.rs`)

```rust
use oxidite_db::Model;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Model, Serialize, Deserialize, Default)]
#[table_name = "posts"]
pub struct Post {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub content: String,
    pub published: bool,
    pub created_at: DateTime<Utc>,
}
```

### Controllers (`src/controllers/post_controller.rs`)

```rust
use crate::AppState;
use oxidite::prelude::*;
use crate::models::post::Post;
use std::collections::HashMap;

pub async fn home(State(state): State<AppState>) -> Result<OxiditeResponse> {
    let posts = Post::query()
        .where_("published", "=", true)
        .order_by("created_at", "DESC")
        .get_all(&*state.db)
        .await?;
    
    let mut context = oxidite_template::Context::new();
    context.insert("posts", &posts);

    let html = state.templates.render("home.html", &context)?;
    
    Ok(OxiditeResponse::html(html))
}

pub async fn show_post(
    Path(params): Path<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<OxiditeResponse> {
    let id: i64 = params.get("id").unwrap().parse()?;
    let post = Post::find(id, &*state.db).await?;
    
    let mut context = oxidite_template::Context::new();
    context.insert("post", &post);

    let html = state.templates.render("post.html", &context)?;
    
    Ok(OxiditeResponse::html(html))
}
```

### Templates (`templates/home.html`)

```html
<!DOCTYPE html>
<html>
<head>
    <title>My Blog</title>
</head>
<body>
    <h1>Latest Posts</h1>
    {% for post in posts %}
    <article>
        <h2><a href="/posts/{{ post.id }}">{{ post.title }}</a></h2>
        <p>{{ post.created_at }}</p>
    </article>
    {% endfor %}
</body>
</html>
```

## Best Practices

1. **Separate concerns**: Models, Controllers, Views
2. **Use middleware**: Auth, CORS, logging
3. **Background jobs**: For slow operations
4. **Caching**: For frequently accessed data
5. **Validation**: Validate all input
6. **Security**: Use CSRF protection, sanitize HTML

## Deploy

See the [Deployment Guide](deployment.md) for production deployment.
