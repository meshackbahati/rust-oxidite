# Templating Guide

Server-side rendering with the Oxidite template engine.

## Installation

```toml
[dependencies]
oxidite = { version = "1.0", features = ["templates"] }
```

## Quick Start

```rust
use oxidite::prelude::*;
use oxidite::template::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut templates = TemplateEngine::new();
    templates.load_dir("templates")?;
    
    let mut app = Router::new();
    
    app.get("/", {
        let templates = templates.clone();
        move |_req| {
            let templates = templates.clone();
            async move {
                let html = templates.render("index.html", context! {
                    title: "Home",
                    message: "Welcome!"
                }).await?;
                
                Ok(OxiditeResponse::html(html))
            }
        }
    });
    
    Server::new(app).listen("127.0.0.1:3000".parse()?).await
}
```

## Template Syntax

`templates/index.html`:
```html
<!DOCTYPE html>
<html>
<head>
    <title>{{ title }}</title>
</head>
<body>
    <h1>{{ message }}</h1>
    
    {% if user %}
        <p>Hello, {{ user.name }}!</p>
    {% else %}
        <p>Please log in</p>
    {% endif %}
    
    <ul>
    {% for item in items %}
        <li>{{ item.name }}</li>
    {% endfor %}
    </ul>
</body>
</html>
```

## Template Inheritance

`templates/layout.html`:
```html
<!DOCTYPE html>
<html>
<head>
    <title>{% block title %}My App{% endblock %}</title>
</head>
<body>
    <nav>Navigation</nav>
    
    <main>
        {% block content %}{% endblock %}
    </main>
    
    <footer>Footer</footer>
</body>
</html>
```

`templates/home.html`:
```html
{% extends "layout.html" %}

{% block title %}Home - My App{% endblock %}

{% block content %}
    <h1>Welcome Home!</h1>
{% endblock %}
```

## Rendering

```rust
use oxidite::template::*;

async fn home() -> Result<OxiditeResponse> {
    let html = templates.render("home.html", context! {
        user: current_user,
        posts: recent_posts
    }).await?;
    
    Ok(Response::html(html))
}
```

## Filters

Use built-in filters:

```html
{{ "hello world" | capitalize }}
{{ 123.456 | round(2) }}
{{ date | format_date("%Y-%m-%d") }}
```

## Complete Example

```rust
use oxidite::prelude::*;
use oxidite::template::*;

#[derive(Serialize)]
struct Post {
    title: String,
    content: String,
    author: String,
}

async fn blog_index(State(db): State<Database>) -> Result<OxiditeResponse> {
    let posts = Post::all(&db).await?;
    
    let html = templates.render("blog/index.html", context! {
        title: "Blog",
        posts: posts
    }).await?;
    
    Ok(OxiditeResponse::html(html))
}
```

`templates/blog/index.html`:
```html
{% extends "layout.html" %}

{% block title %}Blog{% endblock %}

{% block content %}
<h1>Blog Posts</h1>

{% for post in posts %}
<article>
    <h2>{{ post.title }}</h2>
    <p>By {{ post.author }}</p>
    <div>{{ post.content }}</div>
</article>
{% endfor %}
{% endblock %}
```

Complete documentation at [docs.rs/oxidite](https://docs.rs/oxidite)
