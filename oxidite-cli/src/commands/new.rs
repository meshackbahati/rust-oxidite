use std::fs;
use std::path::Path;
use dialoguer::{theme::ColorfulTheme, Select};
use colored::*;

#[derive(Debug, Clone, Copy)]
pub enum ProjectType {
    Fullstack,
    Api,
    Microservice,
    Serverless,
}

impl ProjectType {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "fullstack" => Some(Self::Fullstack),
            "api" => Some(Self::Api),
            "microservice" => Some(Self::Microservice),
            "serverless" => Some(Self::Serverless),
            _ => None,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Fullstack => "Fullstack Application",
            Self::Api => "REST API",
            Self::Microservice => "Microservice",
            Self::Serverless => "Serverless Function",
        }
    }
}

pub fn create_project(name: &str, project_type: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", format!("🚀 Initializing new Oxidite project: {}", name).green().bold());
    
    let p_type = if let Some(t) = project_type {
        ProjectType::from_str(&t).ok_or("Invalid project type. Options: fullstack, api, microservice, serverless")?
    } else {
        let selections = &[
            "Fullstack Application (Frontend + Backend)",
            "REST API (Backend only)",
            "Microservice (Minimal, specialized)",
            "Serverless Function (Event-driven)",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select project type")
            .default(0)
            .items(&selections[..])
            .interact()?;

        match selection {
            0 => ProjectType::Fullstack,
            1 => ProjectType::Api,
            2 => ProjectType::Microservice,
            3 => ProjectType::Serverless,
            _ => unreachable!(),
        }
    };

    println!("Creating {}...", p_type.as_str().cyan());

    // Create project directory
    fs::create_dir(name)?;
    let project_path = Path::new(name);
    
    // Common directories
    let src_path = project_path.join("src");
    fs::create_dir(&src_path)?;
    
    // Create standard structure
    fs::create_dir(src_path.join("models"))?;
    fs::create_dir(src_path.join("routes"))?;
    fs::create_dir(src_path.join("controllers"))?;
    fs::create_dir(src_path.join("services"))?;
    fs::create_dir(src_path.join("middleware"))?;
    fs::create_dir(src_path.join("utils"))?;
    fs::create_dir(src_path.join("config"))?;
    
    // Create tests directory
    fs::create_dir(project_path.join("tests"))?;

    // Specific directories based on type
    match p_type {
        ProjectType::Fullstack => {
            fs::create_dir(project_path.join("templates"))?;
            fs::create_dir(project_path.join("public"))?;
            fs::create_dir(project_path.join("public/css"))?;
            fs::create_dir(project_path.join("public/js"))?;
            fs::create_dir(project_path.join("public/images"))?;
        },
        ProjectType::Microservice => {
             fs::create_dir(src_path.join("queues"))?;
        },
        _ => {}
    }

    // Create Cargo.toml
    create_cargo_toml(project_path, name, p_type)?;
    
    // Create config.toml
    create_config_toml(project_path, p_type)?;
    
    // Create src/main.rs
    create_main_rs(project_path, p_type)?;
    
    // Create other boilerplate files
    create_boilerplate(project_path, p_type)?;

    // Create .gitignore
    let gitignore = r#"/target
Cargo.lock
*.db
*.log
.env
"#;
    fs::write(project_path.join(".gitignore"), gitignore)?;
    
    println!("\n{}", "✅ Project created successfully!".green().bold());
    println!("\n📂 Project structure:");
    println!("  ├── src/");
    println!("  │   ├── models/");
    println!("  │   ├── routes/");
    println!("  │   ├── controllers/");
    println!("  │   ├── services/");
    println!("  │   └── middleware/");
    
    if let ProjectType::Fullstack = p_type {
        println!("  ├── templates/");
        println!("  └── public/");
    }
    
    println!("  └── config.toml");
    println!("\n🚀 Next steps:");
    println!("  cd {}", name);
    println!("  cargo run");
    
    Ok(())
}

fn create_cargo_toml(path: &Path, name: &str, p_type: ProjectType) -> std::io::Result<()> {
    let mut dependencies = String::from(r#"
oxidite-core = "0.1.0"
oxidite-config = "0.1.0"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
"#);

    match p_type {
        ProjectType::Fullstack => {
            dependencies.push_str(r#"
oxidite-template = "0.1.0"
oxidite-auth = "0.1.0"
oxidite-db = "0.1.0"
oxidite-middleware = "0.1.0"
"#);
        },
        ProjectType::Api => {
            dependencies.push_str(r#"
oxidite-auth = "0.1.0"
oxidite-db = "0.1.0"
oxidite-middleware = "0.1.0"
"#);
        },
        ProjectType::Microservice => {
            dependencies.push_str(r#"
oxidite-queue = "0.1.0"
oxidite-middleware = "0.1.0"
"#);
        },
        ProjectType::Serverless => {
            // Minimal deps
        }
    }

    let content = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]{}
"#, name, dependencies);

    fs::write(path.join("Cargo.toml"), content)
}

fn create_config_toml(path: &Path, p_type: ProjectType) -> std::io::Result<()> {
    let mut content = String::from(r#"# Oxidite Configuration

[server]
host = "127.0.0.1"
port = 8080
"#);

    match p_type {
        ProjectType::Fullstack => {
            content.push_str(r#"
[database]
url = "sqlite://data.db"

[templates]
directory = "templates"

[static]
directory = "public"
"#);
        },
        ProjectType::Api => {
            content.push_str(r#"
[database]
url = "sqlite://data.db"
"#);
        },
        ProjectType::Microservice => {
             content.push_str(r#"
[queue]
url = "redis://localhost"
"#);
        },
        _ => {}
    }

    fs::write(path.join("oxidite.toml"), content)
}

fn create_main_rs(path: &Path, p_type: ProjectType) -> std::io::Result<()> {
    let content = match p_type {
        ProjectType::Fullstack => r#"use oxidite_core::{Router, Server};
use oxidite_config::Config;
use oxidite_template::serve_static;

mod routes;
mod controllers;
mod models;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let addr = format!("{}:{}", config.server.host, config.server.port);

    let mut router = Router::new();
    
    // Register routes
    routes::register(&mut router);
    
    // Static files
    // Static files (fallback)
    router.get("/*", serve_static);

    let server = Server::new(router);
    println!("🚀 Server running on http://{}", addr);
    server.listen(addr.parse()?).await?;

    Ok(())
}
"#,
        ProjectType::Api => r#"use oxidite_core::{Router, Server};
use oxidite_config::Config;

mod routes;
mod controllers;
mod models;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let addr = format!("{}:{}", config.server.host, config.server.port);

    let mut router = Router::new();
    
    // Register routes
    routes::register(&mut router);

    let server = Server::new(router);
    println!("🚀 API Server running on http://{}", addr);
    server.listen(addr.parse()?).await?;

    Ok(())
}
"#,
        ProjectType::Microservice => r#"use oxidite_core::{Router, Server};
use oxidite_config::Config;

mod routes;
mod services;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let addr = format!("{}:{}", config.server.host, config.server.port);

    let mut router = Router::new();
    routes::register(&mut router);

    let server = Server::new(router);
    println!("🚀 Microservice running on http://{}", addr);
    server.listen(addr.parse()?).await?;

    Ok(())
}
"#,
        ProjectType::Serverless => r#"use oxidite_core::{OxiditeRequest, OxiditeResponse};

pub async fn handler(req: OxiditeRequest) -> Result<OxiditeResponse, oxidite_core::Error> {
    Ok(OxiditeResponse::json(serde_json::json!({
        "message": "Hello from Serverless Function!"
    })))
}

// Local dev server
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use oxidite_core::{Router, Server};
    
    let mut router = Router::new();
    router.get("/", handler);
    
    let server = Server::new(router);
    println!("🚀 Function running locally on http://127.0.0.1:8080");
    server.listen("127.0.0.1:8080".parse()?).await?;
    
    Ok(())
}
"#,
    };

    fs::write(path.join("src/main.rs"), content)
}

fn create_boilerplate(path: &Path, p_type: ProjectType) -> std::io::Result<()> {
    // Create mod.rs files
    fs::write(path.join("src/models/mod.rs"), "")?;
    fs::write(path.join("src/controllers/mod.rs"), "")?;
    fs::write(path.join("src/services/mod.rs"), "")?;
    fs::write(path.join("src/middleware/mod.rs"), "")?;
    
    // Create routes/mod.rs
    let routes_content = match p_type {
        ProjectType::Fullstack => r#"use oxidite_core::{response, Router, OxiditeRequest, OxiditeResponse};
use oxidite_template::{TemplateEngine, Context};

pub fn register(router: &mut Router) {
    router.get("/", index);
}

async fn index(_req: OxiditeRequest) -> Result<OxiditeResponse, oxidite_core::Error> {
    let mut engine = TemplateEngine::new();
    engine.load_dir("templates").unwrap();

    let mut context = Context::new();
    context.set("name", "Oxidite");

    let body = engine.render("index.html", &context).unwrap();
    Ok(response::html(body))
}
"#,
        ProjectType::Api => r#"use oxidite_core::{response, Router, OxiditeRequest, OxiditeResponse};

pub fn register(router: &mut Router) {
    router.get("/api/health", health);
}

async fn health(_req: OxiditeRequest) -> Result<OxiditeResponse, oxidite_core::Error> {
    Ok(response::json(serde_json::json!({"status": "ok"})))
}
"#,
        _ => r#"use oxidite_core::Router;

pub fn register(_router: &mut Router) {
    // Register routes
}
"#,
    };
    fs::write(path.join("src/routes/mod.rs"), routes_content)?;

    // Fullstack specific files
    if let ProjectType::Fullstack = p_type {
        // CSS
        let css_content = r#"
body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100vh;
    margin: 0;
    background-color: #1a1a1a;
    color: #fff;
}

.container {
    text-align: center;
    padding: 40px;
    background-color: #2a2a2a;
    border-radius: 8px;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
}

img {
    width: 150px;
    margin-bottom: 20px;
}

h1 {
    font-size: 2.5em;
    color: #fff;
}

p {
    font-size: 1.2em;
    color: #ccc;
}
"#;
        fs::write(path.join("public/css/style.css"), css_content)?;
        // JS
        fs::write(path.join("public/js/app.js"), "console.log('App loaded');")?;
        // Templates
        let base_html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Welcome to Oxidite</title>
    <link rel="stylesheet" href="/css/style.css">
</head>
<body>
    {% block content %}{% endblock %}
</body>
</html>"#;
        fs::write(path.join("templates/base.html"), base_html)?;

        let index_html = r#"{% extends "base.html" %}
{% block content %}
<div class="container">
    <img src="/images/oxidite.svg" alt="Oxidite Logo">
    <h1>Welcome to {{ name }}!</h1>
    <p>Your new full-stack application is up and running.</p>
</div>
{% endblock %}"#;
        fs::write(path.join("templates/index.html"), index_html)?;

        // Embed logo
        let logo_content = include_str!("../templates/oxidite.svg");
        fs::write(path.join("public/images/oxidite.svg"), logo_content)?;
    }

    Ok(())
}
