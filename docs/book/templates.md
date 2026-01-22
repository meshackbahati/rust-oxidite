# Template Engine

Oxidite provides a powerful template engine for server-side rendering. The engine supports Jinja2-style syntax with features like variable interpolation, control structures, and template inheritance.

## Basic Template Usage

### Setting Up the Template Engine

First, you need to set up the template engine:

```rust
use oxidite::prelude::*;
use oxidite_template::{TemplateEngine, Context};

async fn setup_template_example(_req: Request) -> Result<Response> {
    // Create a new template engine
    let mut engine = TemplateEngine::new();
    
    // Add a simple template
    engine.add_template(
        "hello", 
        "<h1>Hello {{ name }}!</h1><p>Welcome to {{ framework }}.</p>"
    )?;
    
    // Create context with data
    let mut context = Context::new();
    context.set("name", "Developer");
    context.set("framework", "Oxidite");
    
    // Render the template as an HTML response
    let response = engine.render_response("hello", &context)?;
    Ok(response)
}
```

### Loading Templates from Files

You can load templates from a directory structure:

```rust
use std::path::PathBuf;

async fn file_templates_example(_req: Request) -> Result<Response> {
    let mut engine = TemplateEngine::new();
    
    // Load all templates from a directory (assuming you have template files)
    let templates_dir = PathBuf::from("templates");
    let count = engine.load_dir(&templates_dir)?;
    
    println!("Loaded {} templates", count);
    
    let mut context = Context::new();
    context.set("title", "My Page");
    context.set("content", "Page content here");
    
    let response = engine.render_response("index.html", &context)?;
    Ok(response)
}
```

## Template Syntax

### Variables

Variables in templates are wrapped in `{{ }}`:

```html
<p>Hello {{ name }}!</p>
<p>Your email is {{ user.email }}.</p> <!-- Dotted notation -->
```

### Control Structures

The template engine supports basic control structures:

```html
<!-- Conditionals -->
{% if user.admin %}
    <p>Welcome, administrator!</p>
{% else %}
    <p>Welcome, {{ user.name }}!</p>
{% endif %}

<!-- Loops -->
<ul>
{% for item in items %}
    <li>{{ item }}</li>
{% endfor %}
</ul>
```

## Template Context

The Context struct is used to pass data to templates:

```rust
use oxidite_template::Context;
use serde_json::json;

// Create context in different ways
let mut context = Context::new();

// Set simple values
context.set("name", "Alice");
context.set("age", 30);

// Set complex objects
context.set("user", json!({
    "name": "Bob",
    "email": "bob@example.com",
    "active": true
}));

// Set arrays
context.set("items", vec!["apple", "banana", "cherry"]);

// Create context from JSON
let json_data = json!({
    "title": "My Blog",
    "posts": [
        {"title": "Post 1", "content": "Content 1"},
        {"title": "Post 2", "content": "Content 2"}
    ]
});
let context = Context::from_json(json_data);
```

## Rendering Templates

You can render templates in several ways:

### Render to String

```rust
use oxidite_template::{TemplateEngine, Context};

let mut engine = TemplateEngine::new();
engine.add_template("greeting", "Hello {{ name }}!")?;

let mut context = Context::new();
context.set("name", "World");

let html = engine.render("greeting", &context)?;
assert_eq!(html, "Hello World!");
```

### Render Directly as Response

```rust
use oxidite::prelude::*;
use oxidite_template::{TemplateEngine, Context};

async fn render_as_response(_req: Request) -> Result<Response> {
    let mut engine = TemplateEngine::new();
    engine.add_template("page", "<h1>{{ title }}</h1><div>{{ content }}</div>")?;
    
    let mut context = Context::new();
    context.set("title", "My Page");
    context.set("content", "Page content");
    
    // Render directly as HTML response
    let response = engine.render_response("page", &context)?;
    Ok(response)
}
```

## Template Inheritance

Template inheritance allows you to create base templates that other templates can extend:

Base template (base.html):
```html
<!DOCTYPE html>
<html>
<head>
    <title>{% block title %}Default Title{% endblock %}</title>
</head>
<body>
    <header>
        {% block header %}
        <h1>Default Header</h1>
        {% endblock %}
    </header>
    
    <main>
        {% block content %}{% endblock %}
    </main>
    
    <footer>
        {% block footer %}
        <p>&copy; 2025</p>
        {% endblock %}
    </footer>
</body>
</html>
```

Child template (page.html):
```html
{% extends "base.html" %}

{% block title %}My Page Title{% endblock %}

{% block content %}
    <h2>Page Content</h2>
    <p>This is the main content of the page.</p>
{% endblock %}
```

## Filters

Filters allow you to transform variables:

```html
<!-- Uppercase filter -->
<p>{{ name | upper }}</p>

<!-- Length filter -->
<p>Items count: {{ items | length }}</p>

<!-- Default value if variable is not set -->
<p>Name: {{ user.name | default("Anonymous") }}</p>
```

## Static File Serving

The template engine also includes utilities for serving static files:

```rust
use oxidite::prelude::*;
use oxidite_template::serve_static;

// In your router, register the static file handler
// Note: This should be registered last to avoid blocking other routes
// router.get("/*", serve_static); // Serves files from "public" directory
```

## Complete Example

Here's a complete example showing template usage in a web application:

```rust
use oxidite::prelude::*;
use oxidite_template::{TemplateEngine, Context};
use serde_json::json;

struct AppState {
    template_engine: TemplateEngine,
}

async fn home_page(state: State<AppState>) -> Result<Response> {
    let mut context = Context::new();
    context.set("title", "Home Page");
    context.set("welcome_message", "Welcome to our application!");
    context.set("features", vec![
        "Fast performance",
        "Easy to use",
        "Type-safe",
        "Full-featured"
    ]);
    
    let response = state.template_engine
        .render_response("home.html", &context)?;
    Ok(response)
}

async fn blog_page(state: State<AppState>) -> Result<Response> {
    let posts = vec![
        json!({"title": "First Post", "date": "2025-01-01", "excerpt": "This is the first post"}),
        json!({"title": "Second Post", "date": "2025-01-02", "excerpt": "This is the second post"}),
    ];
    
    let mut context = Context::new();
    context.set("title", "Blog");
    context.set("posts", posts);
    
    let response = state.template_engine
        .render_response("blog.html", &context)?;
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Set up template engine
    let mut template_engine = TemplateEngine::new();
    
    // Add some templates
    template_engine.add_template("home", r#"
        <!DOCTYPE html>
        <html>
        <head><title>{{ title }}</title></head>
        <body>
            <h1>{{ welcome_message }}</h1>
            <ul>
            {% for feature in features %}
                <li>{{ feature }}</li>
            {% endfor %}
            </ul>
        </body>
        </html>
    "#)?;
    
    template_engine.add_template("blog", r#"
        <!DOCTYPE html>
        <html>
        <head><title>{{ title }}</title></head>
        <body>
            <h1>{{ title }}</h1>
            {% for post in posts %}
            <article>
                <h2>{{ post.title }}</h2>
                <small>{{ post.date }}</small>
                <p>{{ post.excerpt }}</p>
            </article>
            {% endfor %}
        </body>
        </html>
    "#)?;
    
    let app_state = AppState { template_engine };
    
    let mut router = Router::new();
    router.get("/", {
        let state = app_state.clone();
        move |_| home_page(State(state))
    });
    router.get("/blog", {
        let state = app_state.clone();
        move |_| blog_page(State(state))
    });
    
    Server::new(router)
        .listen("127.0.0.1:3000".parse()?)
        .await
}
```

## Best Practices

1. **Organize Templates**: Keep templates in a dedicated directory (usually `templates/`)
2. **Use Base Templates**: Create base templates with common layout elements
3. **Context Management**: Use structured context data rather than individual variables
4. **Error Handling**: Always handle template rendering errors appropriately
5. **Caching**: Consider implementing template caching for production applications
6. **Security**: The template engine automatically escapes HTML to prevent XSS

## Security Considerations

The Oxidite template engine includes built-in security features:

- Automatic HTML escaping to prevent XSS
- Context isolation between different template renders
- Input validation for template variables

Remember to always validate and sanitize user input before passing it to templates, especially when dealing with dynamic content.