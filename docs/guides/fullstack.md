# Building Fullstack Applications with Oxidite

Oxidite makes it easy to build server-side rendered applications with modern tooling.

## Creating a Fullstack Project

Use the CLI to scaffold a fullstack project:

```bash
oxidite new my-app --type fullstack
```

This creates a project with the following structure:

```
my-app/
├── src/
│   ├── ...
│   └── main.rs
├── templates/         # HTML templates (Tera)
│   ├── base.html
│   └── index.html
├── public/            # Static assets
│   ├── css/
│   ├── js/
│   └── images/
└── config.toml
```

## Templates

Oxidite uses a Django-inspired template syntax. Templates are located in the `templates/` directory.

### Layouts (`base.html`)

```html
<!DOCTYPE html>
<html>
<head>
    <title>{% block title %}My App{% endblock %}</title>
    <link rel="stylesheet" href="/public/css/style.css">
</head>
<body>
    <nav>...</nav>
    
    <main>
        {% block content %}{% endblock %}
    </main>
</body>
</html>
```

### Pages (`index.html`)

```html
{% extends "base.html" %}

{% block title %}Home{% endblock %}

{% block content %}
    <h1>Welcome to Oxidite!</h1>
    <p>{{ message }}</p>
{% endblock %}
```

## Serving Static Files

The `fullstack` template automatically configures static file serving. Files in the `public/` directory are served from the root URL.

For example:
- `public/css/style.css` is accessible at `/css/style.css`
- `public/js/app.js` is accessible at `/js/app.js`

In `src/main.rs`, the static file handler is registered as a fallback route:

```rust
// Static files are served from the "public" directory
// This matches any path not handled by previous routes
router.get("/*", serve_static);
```

### Custom Directory

You can serve files from a different directory using `static_handler`:

```rust
use oxidite_template::static_handler;

// Serve files from "assets" directory
router.get("/*", static_handler("assets"));
```

## Rendering Templates

In your controllers, use the `TemplateEngine` to render views.

```rust
use oxidite_template::TemplateEngine;

async fn index(req: Request) -> Result<Response, Error> {
    let mut context = Context::new();
    context.insert("message", "Hello World");
    
    let html = engine.render("index.html", &context)?;
    Ok(Response::html(html))
}
```
