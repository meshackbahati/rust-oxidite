# oxidite-template

Template engine for server-side rendering in Oxidite.

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/oxidite-template.svg)](https://crates.io/crates/oxidite-template)
[![Docs.rs](https://docs.rs/oxidite-template/badge.svg)](https://docs.rs/oxidite-template)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

</div>

## Overview

`oxidite-template` provides a powerful and flexible template engine for server-side rendering in the Oxidite web framework. It offers Jinja2-inspired syntax with features like template inheritance, includes, filters, and automatic HTML escaping for security.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
oxidite-template = "2.3.4"
```

## Features

- **Template inheritance** - Base templates with block placeholders for content
- **Template includes** - Reusable partial templates
- **Auto-escaping** - Protection against XSS attacks
- **Filters** - Transform values (uppercase, lowercase, truncate, etc.)
- **Loops and conditionals** - Control flow in templates
- **Custom filters** - Extend functionality with custom filters
- **Template caching** - Improved performance with compiled template caching
- **Internationalization support** - Multi-language template capabilities

## Usage

### Basic Setup

Create a shared template context for rendering templates:

```rust
use oxidite_template::TemplateContext;

// Point to your templates directory
let templates = TemplateContext::new("./templates");
```

> **Alternative**: Use `TemplateEngine` directly for one-off rendering:
> ```rust
> let mut engine = TemplateEngine::new();
> engine.load_dir("./templates")?;
> engine.load_templates().await?;
> ```

### Creating Templates

Create your base template (`templates/base.html`):

```html
<!DOCTYPE html>
<html>
<head>
    <title>{% block title %}Default Title{% endblock %}</title>
</head>
<body>
    <header>
        <nav>Navigation here</nav>
    </header>
    
    <main>
        {% block content %}{% endblock %}
    </main>
    
    <footer>
        <p>&copy; 2025 My App</p>
    </footer>
</body>
</html>
```

Create a child template (`templates/page.html`) that extends the base:

```html
{% extends "base.html" %}

{% block title %}{{ page_title }}{% endblock %}

{% block content %}
    <h1>{{ heading }}</h1>
    <p>{{ content }}</p>
    
    {% if show_button %}
        <button>{{ button_text }}</button>
    {% endif %}
    
    <ul>
    {% for item in items %}
        <li>{{ item.name | upper }}</li>
    {% endfor %}
    </ul>
{% endblock %}
```

### Rendering Templates

Render templates with dynamic data:

```rust
use oxidite_template::{TemplateContext, Context};

let templates = TemplateContext::new("templates");

// Create context with data
let mut ctx = Context::new();
ctx.set("page_title", "My Page");
ctx.set("heading", "Welcome!");
ctx.set("content", "This is my content.");
ctx.set("show_button", true);
ctx.set("button_text", "Click Me");

// Add a vector of items
let items = vec![
    Item { name: "First".to_string() },
    Item { name: "Second".to_string() },
];
ctx.set("items", items);

// Render the template
let html = templates.render("page.html", &ctx)?;
```

### Template Filters

Apply transformations to values in templates:

```html
<!-- Convert to uppercase -->
<h1>{{ title | upper }}</h1>

<!-- Convert to lowercase -->
<p>{{ description | lower }}</p>

<!-- Truncate text -->
<p>{{ long_text | truncate(100) }}</p>

<!-- Escape HTML (done automatically by default) -->
<p>{{ user_input | escape }}</p>

<!-- URL encode -->
<a href="/search?q={{ query | urlencode }}">Search</a>
```

### Conditional Logic

Handle conditional rendering in templates:

```html
{% if user.is_authenticated %}
    <p>Welcome back, {{ user.name }}!</p>
    {% if user.is_admin %}
        <a href="/dashboard">Dashboard</a>
    {% endif %}
{% else %}
    <a href="/login">Log In</a>
{% endif %}
```

### Loops

Iterate over collections in templates:

```html
<ul>
{% for product in products %}
    <li>
        <h3>{{ product.name }}</h3>
        <p>Price: ${{ product.price }}</p>
        {% if product.on_sale %}
            <span class="sale">On Sale!</span>
        {% endif %}
    </li>
{% else %}
    <li>No products available.</li>
{% endfor %}
</ul>
```

### Template Includes

Include reusable template parts:

```html
<!-- Include a header component -->
{% include "partials/header.html" %}

<!-- Include with context -->
{% include "components/alert.html" with { type: "info", message: "Hello" } %}
```

### Integration with Oxidite

Using templates with Oxidite's response utilities:

```rust
use std::sync::Arc;
use oxidite::prelude::*;
use oxidite_template::{TemplateContext, Context};

async fn home_page(
    _req: Request,
    State(templates): State<Arc<TemplateContext>>
) -> Result<Response> {
    let mut ctx = Context::new();
    ctx.set("title", "Home Page");
    ctx.set("welcome_message", "Welcome to our site!");
    
    let html = templates.render("home.html", &ctx)?;
    Ok(response::html(html))
}
```

Register the shared template context with the router:

```rust
let templates = Arc::new(TemplateContext::new("templates"));
app.router_mut().with_state(templates);
```

## Security

The template engine includes built-in security features:

- **Automatic HTML escaping** prevents cross-site scripting (XSS) attacks
- **Sandboxed execution** prevents arbitrary code execution in templates
- **Whitelisted filters** only safe operations are allowed

## Performance

- **Template caching** compiled templates are cached for improved performance
- **Efficient rendering** optimized algorithms for fast template processing
- **Memory management** efficient memory usage during rendering

## License

MIT
