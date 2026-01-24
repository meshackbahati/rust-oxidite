# Fullstack Web Development with Oxidite

Build complete fullstack applications with server-side rendering, authentication, and database integration.

## Installation

```toml
[dependencies]
oxidite = { version = "1.0", features = ["full"] }
# or specify features:
oxidite = { version = "1.0", features = ["database", "auth", "templates", "queue", "cache"] }
```

## Complete Example: Blog Application

### Project Setup

```bash
cargo new blog
cd blog
cargo add oxidite
cargo add tokio --features full
cargo add serde --features derive
```

### Main Application

```rust
use oxidite::prelude::*;
use oxidite::db::DbPool;
use oxidite::template::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Database
    let db = DbPool::connect(&std::env::var("DATABASE_URL")?).await?;
    
    // Template engine
    let mut templates = TemplateEngine::new();
    templates.load_dir("templates")?;
    
    // Routes
    let mut app = Router::new();
    
    // Public routes
    app.get("/", home);
    app.get("/posts/:id", show_post);
    
    // Auth routes
    app.get("/login", login_form);
    app.post("/login", login);
    
    // Protected routes
    app.get("/posts/new", new_post_form).middleware(AuthMiddleware);
    app.post("/posts", create_post).middleware(AuthMiddleware);
    
    // Middleware
    let app = ServiceBuilder::new()
        .layer(LoggerLayer)
        .layer(CorsLayer::permissive())
        .service(app);
    
    Server::new(app).listen("127.0.0.1:3000".parse()?).await
}
```

### Models

```rust
use oxidite::db::*;

#[derive(Model, Serialize, Deserialize)]
#[table_name = "users"]
struct User {
    id: i64,
    name: String,
    email: String,
    password_hash: String,
}

#[derive(Model, Serialize, Deserialize)]
#[table_name = "posts"]
struct Post {
    id: i64,
    user_id: i64,
    title: String,
    content: String,
    published: bool,
    created_at: DateTime<Utc>,
}
```

### Controllers

```rust
async fn home(State(db): State<Database>) -> Result<Response> {
    let posts = Post::where_eq(&db, "published", true)
        .order_by("created_at", "DESC")
        .get()
        .await?;
    
    let html = templates.render("home.html", context! {
        posts: posts
    }).await?;
    
    Ok(Response::html(html))
}

async fn show_post(
    Path(params): Path<HashMap<String, String>>,
    State(db): State<Database>,
) -> Result<Response> {
    let id = params.get("id").unwrap().parse()?;
    let post = Post::find(&db, id).await?;
    let author = User::find(&db, post.user_id).await?;
    
    let html = templates.render("post.html", context! {
        post: post,
        author: author
    }).await?;
    
    Ok(Response::html(html))
}

async fn create_post(
    auth: Auth,
    Json(data): Json<CreatePostRequest>,
    State(db): State<Database>,
) -> Result<Response> {
    let post = Post {
        user_id: auth.user.id,
        title: data.title,
        content: data.content,
        published: false,
        ..Default::default()
    };
    
    post.save(&db).await?;
    
    Ok(Response::redirect("/posts"))
}
```

### Templates

`templates/home.html`:
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

### Background Jobs

```rust
use oxidite::queue::*;

#[derive(Serialize, Deserialize)]
struct SendWelcomeEmail {
    user_id: i64,
}

#[async_trait]
impl Job for SendWelcomeEmail {
    async fn perform(&self) -> JobResult {
        let user = User::find(&db, self.user_id).await?;
        send_email(&user.email, "Welcome!", "Thanks for joining!").await?;
        Ok(())
    }
}

// Enqueue after user registration
queue.enqueue(JobWrapper::new(&SendWelcomeEmail {
    user_id: new_user.id
})?).await?;
```

### File Uploads

```rust
use oxidite::storage::*;

async fn upload_avatar(
    auth: Auth,
    multipart: Multipart,
    State(storage): State<Storage>,
) -> Result<Response> {
    let file = multipart.file("avatar").await?;
    
    let path = storage.put(
        &format!("avatars/{}.jpg", auth.user.id),
        file.data
    ).await?;
    
    Ok(Json(json!({ "url": path })))
}
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
