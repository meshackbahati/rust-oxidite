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
use oxidite_template::{TemplateEngine, Context};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    templates: Arc<TemplateEngine>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let templates = Arc::new(TemplateEngine::new("templates"));
    let state = AppState { templates };
    
    let mut app = Router::new();
    
    app.get("/", |State(state): State<AppState>| async move {
        let mut context = Context::new();
        context.insert("title", "Home");
        context.insert("message", "Welcome!");

        let html = state.templates.render("index.html", &context)?;

        Ok(OxiditeResponse::html(html))
    });
    
    let app = app.with_state(state);

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
use oxidite::prelude::*;
use oxidite_template::Context;
use serde::Serialize;

#[derive(Serialize)]
struct User { name: String }

#[derive(Serialize)]
struct Post { title: String }

async fn home(State(state): State<AppState>) -> Result<OxiditeResponse> {
    let mut context = Context::new();
    context.insert("user", &User { name: "Alice".to_string() });
    context.insert("posts", &vec![Post { title: "First Post".to_string() }]);
    
    let html = state.templates.render("home.html", &context)?;
    
    Ok(OxiditeResponse::html(html))
}
```
