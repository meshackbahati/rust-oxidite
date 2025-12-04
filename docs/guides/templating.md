# Templating Guide

## Overview

Oxidite's template engine provides a Django-inspired templating system with support for variables, control structures, template inheritance, and filters.

## Basic Usage

```rust
use oxidite_template::{Template, Context};

let tmpl = Template::new("Hello {{ name }}!").unwrap();
let mut ctx = Context::new();
ctx.set("name", "World");

let output = tmpl.render(&ctx).unwrap();
// Output: "Hello World!"
```

## Template Syntax

### Variables

Use `{{ variable }}` to insert values:

```html
<h1>{{ title }}</h1>
<p>Welcome, {{ user.name }}!</p>
```

Variables support dot notation for accessing nested fields.

### Filters

Apply filters to variables using the pipe (`|`) syntax:

```html
{{ name | upper }}
{{ content | escape }}
```

**Built-in filters:**
- `upper` - Convert to uppercase
- `lower` - Convert to lowercase  
- `escape` - HTML escape (applied by default)

### Control Structures

#### If Statements

```html
{% if user.is_authenticated %}
    <p>Welcome back, {{ user.name }}!</p>
{% else %}
    <p>Please log in.</p>
{% endif %}
```

#### For Loops

```html
<ul>
{% for item in items %}
    <li>{{ item.name }}</li>
{% endfor %}
</ul>
```

## Template Inheritance

### Base Template

Create a base template (`base.html`):

```html
<!DOCTYPE html>
<html>
<head>
    <title>{% block title %}My Site{% endblock %}</title>
</head>
<body>
    <header>
        {% block header %}
        <h1>Welcome</h1>
        {% endblock %}
    </header>
    
    <main>
        {% block content %}{% endblock %}
    </main>
    
    <footer>
        {% block footer %}
        <p>&copy; 2024 My Site</p>
        {% endblock %}
    </footer>
</body>
</html>
```

### Child Template

Extend the base template (`page.html`):

```html
{% extends "base.html" %}

{% block title %}My Page{% endblock %}

{% block content %}
    <h2>Page Content</h2>
    <p>This overrides the content block.</p>
{% endblock %}
```

### Using Template Engine

```rust
use oxidite_template::{TemplateEngine, Context};

let mut engine = TemplateEngine::new();

// Load templates
engine.add_template("base.html", base_html).unwrap();
engine.add_template("page.html", child_html).unwrap();

// Render child template (inheritance is automatic)
let ctx = Context::new();
let output = engine.render("page.html", &ctx).unwrap();
```

## Includes

Include partial templates:

```html
<!-- header.html -->
<header>
    <h1>{{ site_name }}</h1>
</header>

<!-- main.html -->
{% include "header.html" %}
<main>
    <p>Content here</p>
</main>
```

## Advanced

### Nested Blocks

Blocks can be nested within other blocks for complex inheritance patterns:

```html
{% extends "base.html" %}

{% block content %}
    <div class="wrapper">
        {% block inner %}
        Default inner content
        {% endblock %}
    </div>
{% endblock %}
```

### Context with JSON

```rust
let mut ctx = Context::new();
ctx.set("user", serde_json::json!({
    "name": "Alice",
    "role": "admin",
    "permissions": ["read", "write"]
}));
```

## Best Practices

1. **Keep templates simple** - Move complex logic to Rust code
2. **Use inheritance** - Create base templates for consistent layouts
3. **Organize partials** - Use includes for reusable components
4. **Pass structured data** - Use JSON objects for complex data
5. **Escape by default** - Variables are HTML-escaped automatically

## Next Steps

- [Database Guide](database.md) - Learn about ORM and query building
- [Authentication Guide](authentication.md) - Add user authentication
