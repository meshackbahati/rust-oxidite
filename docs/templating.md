# Templating System

Oxidite includes a powerful templating engine for server-side rendering of HTML content.

## Overview

The templating system provides:

- Jinja2-like syntax for familiar template creation
- Template inheritance for reusable layouts
- Context variables for dynamic content
- Built-in filters for content manipulation
- Static file serving capabilities
- Auto-escaping for XSS prevention

## Installation

Add the template feature to your `Cargo.toml`:

```toml
[dependencies]
oxidite = { version = "1.0", features = ["templates"] }
```

## Basic Usage

### Creating a Template Engine

```rust
use oxidite::template::{TemplateEngine, Context};

// Create a new template engine
let mut engine = TemplateEngine::new();

// Add a template from a string
engine.add_template("hello", "<h1>Hello, {{ name }}!</h1>")?;
```

### Rendering Templates

```rust
use oxidite::template::{TemplateEngine, Context};

let mut engine = TemplateEngine::new();
engine.add_template("greeting", "<h1>Hello, {{ name }}!</h1>")?;

let mut context = Context::new();
context.set("name", "World");

let html = engine.render("greeting", &context)?;
println!("{}", html); // Output: <h1>Hello, World!</h1>
```

## Template Syntax

### Variables

Variables are enclosed in double curly braces:

```html
<h1>{{ title }}</h1>
<p>Welcome, {{ user.name }}!</p>
```

### Conditionals

```html
{% if user.is_admin %}
    <p>You have administrator privileges.</p>
{% elif user.is_member %}
    <p>You are a member.</p>
{% else %}
    <p>Please log in.</p>
{% endif %}
```

### Loops

```html
<ul>
{% for item in items %}
    <li>{{ item.name }} - {{ item.description }}</li>
{% endfor %}
</ul>
```

### Template Inheritance

Base template (`layout.html`):

```html
<!DOCTYPE html>
<html>
<head>
    <title>{% block title %}Default Title{% endblock %}</title>
</head>
<body>
    <header>
        {% block header %}
            <h1>My Website</h1>
        {% endblock %}
    </header>
    
    <main>
        {% block content %}{% endblock %}
    </main>
    
    <footer>
        {% block footer %}
            <p>&copy; 2024 My Website</p>
        {% endblock %}
    </footer>
</body>
</html>
```

Child template (`page.html`):

```html
{% extends "layout.html" %}

{% block title %}Page Title{% endblock %}

{% block content %}
    <h2>Welcome to the page</h2>
    <p>This is the page content.</p>
{% endblock %}
```

## Context Variables

### Setting Variables

```rust
use oxidite::template::Context;

let mut context = Context::new();

// Simple values
context.set("title", "My Page");
context.set("count", 42);

// Objects
context.set("user", serde_json::json!({
    "name": "John",
    "email": "john@example.com"
}));

// Arrays
context.set("items", vec!["apple", "banana", "cherry"]);
```

### Nested Object Access

```rust
use serde_json::json;

let mut context = Context::new();
context.set("user", json!({
    "profile": {
        "name": "John Doe",
        "settings": {
            "theme": "dark"
        }
    }
}));

// Access nested properties: user.profile.name, user.profile.settings.theme
```

## Template Engine Operations

### Loading Templates from Directory

```rust
use oxidite::template::TemplateEngine;

let mut engine = TemplateEngine::new();

// Load all templates from a directory (recursively)
let count = engine.load_dir("templates/")?;

println!("Loaded {} templates", count);
```

### Adding Multiple Templates

```rust
use oxidite::template::TemplateEngine;

let mut engine = TemplateEngine::new();

// Add multiple templates
engine.add_template("home", "<h1>Home Page</h1><p>{{ message }}</p>")?;
engine.add_template("about", "<h1>About Us</h1><p>{{ content }}</p>")?;
engine.add_template("contact", r#"
<div class="contact-form">
    <h2>Contact {{ company.name }}</h2>
    <p>Email: {{ company.email }}</p>
</div>
"#)?;
```

### Checking for Templates

```rust
use oxidite::template::TemplateEngine;

let engine = TemplateEngine::new();
// ... add templates ...

if engine.get_template("home").is_some() {
    println!("Home template exists");
} else {
    println!("Home template not found");
}
```

## Using Templates in Routes

### Basic Template Route

```rust
use oxidite::prelude::*;
use oxidite::template::{TemplateEngine, Context};
use serde_json::json;

// Global template engine (in a real app, you'd want to store this in application state)
static mut TEMPLATE_ENGINE: Option<TemplateEngine> = None;

async fn home_page(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    // In a real app, you'd get the template engine from application state
    let mut engine = TemplateEngine::new();
    engine.add_template("home", r#"
    <!DOCTYPE html>
    <html>
    <head><title>{{ title }}</title></head>
    <body>
        <h1>{{ heading }}</h1>
        <p>{{ message }}</p>
    </body>
    </html>
    "#)?;
    
    let mut context = Context::new();
    context.set("title", "My Website");
    context.set("heading", "Welcome");
    context.set("message", "This is a template-rendered page");
    
    let html = engine.render("home", &context)?;
    Ok(response::html(html))
}
```

### Template Route with State

```rust
use oxidite::prelude::*;
use oxidite::template::{TemplateEngine, Context};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    template_engine: Arc<TemplateEngine>,
}

async fn user_profile(
    State(state): State<Arc<AppState>>,
    Path(params): Path<serde_json::Value>
) -> Result<OxiditeResponse> {
    let user_id = params["id"].as_str().unwrap_or("1");
    
    let mut context = Context::new();
    context.set("user_id", user_id);
    context.set("user_name", "John Doe");
    context.set("email", "john@example.com");
    
    let html = state.template_engine.render("user_profile.html", &context)
        .map_err(|e| Error::Server(format!("Template error: {}", e)))?;
    
    Ok(response::html(html))
}
```

## Static File Serving

### Serving Static Files

```rust
use oxidite::prelude::*;
use oxidite::template::serve_static;

// Serve static files from the "public" directory
// This should be registered as the last route to avoid interfering with other routes
let mut router = Router::new();
router.get("/css/*", serve_static);   // CSS files
router.get("/js/*", serve_static);    // JavaScript files
router.get("/images/*", serve_static); // Image files
router.get("/static/*", serve_static); // General static files
```

### Static File Configuration

```rust
use oxidite::template::{StaticFiles, serve_static};

// Configure static file serving
let static_files = StaticFiles::new("public/")
    .with_max_age(3600)  // Cache for 1 hour
    .with_gzip(true);    // Enable gzip compression
```

## Complete Example

Here's a complete example showing how to use the templating system:

```rust
use oxidite::prelude::*;
use oxidite::template::{TemplateEngine, Context};
use serde_json::json;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    template_engine: Arc<TemplateEngine>,
}

// Template for the home page
const HOME_TEMPLATE: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>{{ title }}</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        .nav { margin-bottom: 20px; }
        .nav a { margin-right: 10px; text-decoration: none; color: blue; }
        .content { border: 1px solid #ccc; padding: 20px; }
    </style>
</head>
<body>
    <nav class="nav">
        <a href="/">Home</a>
        <a href="/users">Users</a>
        <a href="/about">About</a>
    </nav>
    
    <div class="content">
        <h1>{{ heading }}</h1>
        <p>{{ message }}</p>
        
        {% if show_users %}
        <h2>Users:</h2>
        <ul>
        {% for user in users %}
            <li>{{ user.name }} ({{ user.role }})</li>
        {% endfor %}
        </ul>
        {% endif %}
    </div>
</body>
</html>
"#;

async fn home_handler(
    State(state): State<Arc<AppState>>
) -> Result<OxiditeResponse> {
    let mut context = Context::new();
    context.set("title", "My Oxidite App");
    context.set("heading", "Welcome to Oxidite");
    context.set("message", "This page is rendered with the Oxidite templating engine");
    context.set("show_users", true);
    context.set("users", vec![
        json!({"name": "Alice", "role": "admin"}),
        json!({"name": "Bob", "role": "user"}),
        json!({"name": "Charlie", "role": "moderator"})
    ]);
    
    let html = state.template_engine.render("home", &context)
        .map_err(|e| Error::Server(format!("Template error: {}", e)))?;
    
    Ok(response::html(html))
}

async fn about_handler(
    State(state): State<Arc<AppState>>
) -> Result<OxiditeResponse> {
    let about_template = r#"
    <!DOCTYPE html>
    <html>
    <head><title>About</title></head>
    <body>
        <h1>About This Site</h1>
        <p>Built with the Oxidite web framework.</p>
        <p>Current time: {{ current_time }}</p>
    </body>
    </html>
    "#;
    
    // In a real app, you'd probably add this template when initializing the engine
    let mut temp_engine = TemplateEngine::new();
    temp_engine.add_template("about", about_template)?;
    
    let mut context = Context::new();
    context.set("current_time", chrono::Utc::now().to_rfc2822());
    
    let html = temp_engine.render("about", &context)
        .map_err(|e| Error::Server(format!("Template error: {}", e)))?;
    
    Ok(response::html(html))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize template engine
    let mut template_engine = TemplateEngine::new();
    template_engine.add_template("home", HOME_TEMPLATE)?;
    
    let state = Arc::new(AppState {
        template_engine: Arc::new(template_engine),
    });
    
    let mut router = Router::new();
    router.get("/", home_handler);
    router.get("/about", about_handler);
    
    // Serve static files (should be last to avoid catching other routes)
    router.get("/public/*", serve_static);
    
    let service = ServiceBuilder::new()
        .layer(AddExtensionLayer::new(state))
        .service(router);
    
    Server::new(service)
        .listen("127.0.0.1:3000".parse().unwrap())
        .await
}
```

## Best Practices

1. **Pre-load templates**: Load all templates at startup rather than on-demand
2. **Use template inheritance**: Create base layouts to avoid repetition
3. **Escape user content**: The template engine should auto-escape variables by default
4. **Organize templates**: Group related templates in subdirectories
5. **Cache compiled templates**: Store template engine in application state
6. **Separate concerns**: Keep logic in handlers, presentation in templates
7. **Use context properly**: Pass only necessary data to templates

## Security Considerations

- **Auto-escaping**: Templates should automatically escape variables to prevent XSS
- **Input validation**: Validate and sanitize data before passing to templates
- **Template sandboxing**: Prevent templates from executing arbitrary code
- **File access restrictions**: Limit template loading to designated directories
- **Context isolation**: Prevent templates from accessing unauthorized data