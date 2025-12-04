use std::fs;
use std::path::Path;

pub fn create_project(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Creating new Oxidite project: {}", name);
    
    // Create project directory
    fs::create_dir(name)?;
    let project_path = Path::new(name);
    
    // Create subdirectories
    fs::create_dir(project_path.join("src"))?;
    fs::create_dir(project_path.join("src/models"))?;
    fs::create_dir(project_path.join("src/controllers"))?;
    fs::create_dir(project_path.join("src/middleware"))?;
    fs::create_dir(project_path.join("templates"))?;
    fs::create_dir(project_path.join("public"))?;
    fs::create_dir(project_path.join("public/css"))?;
    fs::create_dir(project_path.join("public/js"))?;
    fs::create_dir(project_path.join("public/images"))?;
    fs::create_dir(project_path.join("migrations"))?;
    fs::create_dir(project_path.join("tests"))?;
    
    // Create Cargo.toml
    let cargo_toml = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
oxidite-core = "0.1"
oxidite-db = "0.1"
oxidite-auth = "0.1"
oxidite-middleware = "0.1"
oxidite-template = "0.1"
oxidite-config = "0.1"
tokio = {{ version = "1", features = ["full"] }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
"#, name);
    
    fs::write(project_path.join("Cargo.toml"), cargo_toml)?;
    
    // Create config.toml
    let config_toml = r#"# Oxidite Configuration

[server]
host = "127.0.0.1"
port = 8080

[database]
url = "sqlite://data.db"
# url = "postgres://user:pass@localhost/mydb"

[cache]
type = "memory"
# type = "redis"
# url = "redis://localhost"

[templates]
directory = "templates"

[static]
directory = "public"
"#;
    fs::write(project_path.join("config.toml"), config_toml)?;
    
    // Create src/main.rs
    let main_rs = r#"use oxidite_core::{Router, Server, Request, Response};
use oxidite_config::Config;
use oxidite_template::TemplateEngine;

async fn index(_req: Request) -> Result<Response, oxidite_core::Error> {
    Ok(Response::html("<h1>Welcome to Oxidite!</h1>"))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::from_file("config.toml")?;
    let host = format!("{}:{}", 
        config.get("server.host")?,
        config.get("server.port")?
    );
    
    // Setup router
    let mut router = Router::new();
    router.get("/", index);
    
    // Start server
    let server = Server::new(host.parse()?, router);
    println!("🚀 Server running on http://{}", host);
    server.run().await?;
    
    Ok(())
}
"#;
    
    fs::write(project_path.join("src/main.rs"), main_rs)?;
    
    // Create base template
    let base_html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% block title %}Oxidite App{% endblock %}</title>
    <link rel="stylesheet" href="/css/style.css">
</head>
<body>
    <header>
        <nav>
            <h1>My Oxidite App</h1>
        </nav>
    </header>
    
    <main>
        {% block content %}{% endblock %}
    </main>
    
    <footer>
        <p>&copy; 2024 My Oxidite App</p>
    </footer>
    
    <script src="/js/app.js"></script>
</body>
</html>
"#;
    fs::write(project_path.join("templates/base.html"), base_html)?;
    
    // Create index template
   let index_html = r#"{% extends "base.html" %}

{% block title %}Home - Oxidite App{% endblock %}

{% block content %}
    <h1>Welcome to Oxidite!</h1>
    <p>Your modern Rust web framework.</p>
{% endblock %}
"#;
    fs::write(project_path.join("templates/index.html"), index_html)?;
    
    // Create CSS
    let style_css = r#"* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
    line-height: 1.6;
    color: #333;
}

header {
    background: #2c3e50;
    color: white;
    padding: 1rem;
}

nav h1 {
    font-size: 1.5rem;
}

main {
    max-width: 1200px;
    margin: 2rem auto;
    padding: 0 1rem;
}

footer {
    background: #34495e;
    color: white;
    text-align: center;
    padding: 1rem;
    margin-top: 2rem;
}
"#;
    fs::write(project_path.join("public/css/style.css"), style_css)?;
    
    // Create JS
    let app_js = r#"console.log('Oxidite app loaded');
"#;
    fs::write(project_path.join("public/js/app.js"), app_js)?;
    
    // Create README.md
    let readme = format!(r#"# {}

A new Oxidite web application.

## Getting Started

```bash
cd {}
cargo run
```

Visit: http://localhost:8080

## Project Structure

```
{}/
├── src/
│   ├── main.rs          # Application entry point
│   ├── models/          # Database models
│   ├── controllers/     # Request handlers
│   └── middleware/      # Custom middleware
├── templates/           # HTML templates
│   └── base.html        # Base template layout
├── public/              # Static files
│   ├── css/
│   ├── js/
│   └── images/
├── migrations/          # Database migrations
├── tests/               # Tests
├── config.toml          # Configuration
└── Cargo.toml

```

## Database Migrations

Create migration:
```bash
oxidite migrate create create_users_table
```

Run migrations:
```bash
oxidite migrate run
```

## Configuration

Edit `config.toml` to configure:
- Server host/port
- Database connection
- Cache settings
- Template directory

## Learn More

- [Oxidite Documentation](https://github.com/yourusername/oxidite)
- [Template Guide](https://github.com/yourusername/oxidite/docs/guides/templating.md)
- [Database Guide](https://github.com/yourusername/oxidite/docs/guides/database.md)
"#, name, name, name);
    
    fs::write(project_path.join("README.md"), readme)?;
    
    // Create .gitignore
    let gitignore = r#"/target
Cargo.lock
*.db
*.log
.env
"#;
    fs::write(project_path.join(".gitignore"), gitignore)?;
    
    println!("\n✅ Project created successfully!");
    println!("\n📂 Project structure:");
    println!("  ├── src/            Source code");
    println!("  ├── templates/      HTML templates");
    println!("  ├── public/         Static files (CSS, JS, images)");
    println!("  ├── migrations/     Database migrations");
    println!("  └── config.toml     Configuration");
    println!("\n🚀 Next steps:");
    println!("  cd {}", name);
    println!("  cargo run");
    
    Ok(())
}
