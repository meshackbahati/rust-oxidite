# Building a Fullstack App with Oxidite

This guide walks you through building a complete fullstack application using Oxidite. We'll build a simple blog application with authentication, database storage, and server-side rendered templates.

## Prerequisites

- Rust installed (latest stable)
- Oxidite CLI installed (`cargo install --path oxidite-cli`)
- A database (SQLite for this guide)

## 1. Create a New Project

Use the CLI to scaffold a new project:

```bash
oxidite new my-blog
cd my-blog
```

## 2. Database Setup

Configure your database in `.env`:

```env
DATABASE_URL=sqlite:blog.db
```

Create a migration for users and posts:

```bash
oxidite migrate create init_schema
```

Edit the generated migration file in `migrations/`:

```sql
-- Up
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE posts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    user_id INTEGER NOT NULL REFERENCES users(id),
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Down
DROP TABLE posts;
DROP TABLE users;
```

Run the migration:

```bash
oxidite migrate run
```

## 3. Models

Create structs to represent your data. You can use `oxidite make model` or create them manually in `src/models.rs`.

```rust
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub user_id: i64,
}
```

## 4. Templates

Create your HTML templates in the `templates/` directory. Oxidite uses a Jinja2-like syntax.

`templates/base.html`:
```html
<!DOCTYPE html>
<html>
<head>
    <title>{% block title %}My Blog{% endblock %}</title>
</head>
<body>
    <nav>
        <a href="/">Home</a>
        {% if user %}
            <a href="/logout">Logout ({{ user.username }})</a>
        {% else %}
            <a href="/login">Login</a>
        {% endif %}
    </nav>
    <main>
        {% block content %}{% endblock %}
    </main>
</body>
</html>
```

`templates/home.html`:
```html
{% extends "base.html" %}

{% block content %}
    <h1>Latest Posts</h1>
    {% for post in posts %}
        <article>
            <h2>{{ post.title }}</h2>
            <p>{{ post.content }}</p>
        </article>
    {% endfor %}
{% endblock %}
```

## 5. Controllers & Routes

Implement your application logic in `src/main.rs` or separate controller files.

```rust
use oxidite_core::{Router, Server, Request, Response, Result};
use oxidite_template::TemplateEngine;
use oxidite_db::Database;

// ... (imports for models and auth)

async fn home(req: Request) -> Result<Response> {
    let db = req.state::<Database>().unwrap();
    let tmpl = req.state::<TemplateEngine>().unwrap();

    let posts: Vec<Post> = sqlx::query_as("SELECT * FROM posts ORDER BY created_at DESC")
        .fetch_all(&db.pool)
        .await
        .map_err(|e| oxidite_core::Error::Server(e.to_string()))?;

    let mut context = tera::Context::new();
    context.insert("posts", &posts);

    let html = tmpl.render("home.html", &context)?;
    Ok(Response::html(html))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize components
    let db = Database::new("sqlite:blog.db").await?;
    let tmpl = TemplateEngine::new("templates/**/*")?;

    let mut router = Router::new();
    router.get("/", home);
    // ... add other routes

    // Add state to router
    router.with_state(db);
    router.with_state(tmpl);

    let server = Server::new("127.0.0.1:3000".parse()?, router);
    server.run().await
}
```

## 6. Authentication

Use `oxidite-auth` for handling user sessions and password hashing.

```rust
use oxidite_auth::{AuthMiddleware, SessionManager};

// ... inside main
let auth_middleware = AuthMiddleware::new(SessionManager::new());
router.layer(auth_middleware);
```

## 7. Running the App

```bash
cargo run
```

Visit `http://localhost:3000` to see your blog!

## Next Steps

- Add form handling for creating posts.
- Implement user registration and login handlers.
- Add input validation.
- Deploy your application using Docker.
