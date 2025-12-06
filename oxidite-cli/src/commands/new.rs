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
oxidite-core = "0.1"
oxidite-config = "0.1"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
"#);

    match p_type {
        ProjectType::Fullstack => {
            dependencies.push_str(r#"
oxidite-template = "0.1"
oxidite-auth = "0.1"
oxidite-db = "0.1"
oxidite-middleware = "0.1"
"#);
        },
        ProjectType::Api => {
            dependencies.push_str(r#"
oxidite-auth = "0.1"
oxidite-db = "0.1"
oxidite-middleware = "0.1"
"#);
        },
        ProjectType::Microservice => {
            dependencies.push_str(r#"
oxidite-queue = "0.1"
oxidite-middleware = "0.1"
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

    fs::write(path.join("config.toml"), content)
}

fn create_main_rs(path: &Path, p_type: ProjectType) -> std::io::Result<()> {
    let content = match p_type {
        ProjectType::Fullstack => r#"use oxidite_core::{Router, Server, Request, Response};
use oxidite_config::Config;
use oxidite_template::serve_static;

mod routes;
mod controllers;
mod models;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_file("config.toml")?;
    let addr = format!("{}:{}", config.get("server.host")?, config.get("server.port")?);

    let mut router = Router::new();
    
    // Register routes
    routes::register(&mut router);
    
    // Static files
    // Static files (fallback)
    router.get("/*", serve_static);

    let server = Server::new(addr.parse()?, router);
    println!("🚀 Server running on http://{}", addr);
    server.run().await?;

    Ok(())
}
"#,
        ProjectType::Api => r#"use oxidite_core::{Router, Server, Request, Response};
use oxidite_config::Config;

mod routes;
mod controllers;
mod models;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_file("config.toml")?;
    let addr = format!("{}:{}", config.get("server.host")?, config.get("server.port")?);

    let mut router = Router::new();
    
    // Register routes
    routes::register(&mut router);

    let server = Server::new(addr.parse()?, router);
    println!("🚀 API Server running on http://{}", addr);
    server.run().await?;

    Ok(())
}
"#,
        ProjectType::Microservice => r#"use oxidite_core::{Router, Server};
use oxidite_config::Config;

mod routes;
mod services;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_file("config.toml")?;
    let addr = format!("{}:{}", config.get("server.host")?, config.get("server.port")?);

    let mut router = Router::new();
    routes::register(&mut router);

    let server = Server::new(addr.parse()?, router);
    println!("🚀 Microservice running on http://{}", addr);
    server.run().await?;

    Ok(())
}
"#,
        ProjectType::Serverless => r#"use oxidite_core::{Request, Response};

pub async fn handler(req: Request) -> Result<Response, oxidite_core::Error> {
    Ok(Response::json(serde_json::json!({
        "message": "Hello from Serverless Function!"
    })))
}

// Local dev server
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use oxidite_core::{Router, Server};
    
    let mut router = Router::new();
    router.get("/", handler);
    
    let server = Server::new("127.0.0.1:8080".parse()?, router);
    println!("🚀 Function running locally on http://127.0.0.1:8080");
    server.run().await?;
    
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
        ProjectType::Fullstack => r#"use oxidite_core::{Router, Request, Response};

pub fn register(router: &mut Router) {
    router.get("/", index);
}

async fn index(_req: Request) -> Result<Response, oxidite_core::Error> {
    Ok(Response::html("<h1>Welcome to Oxidite Fullstack!</h1>"))
}
"#,
        ProjectType::Api => r#"use oxidite_core::{Router, Request, Response};

pub fn register(router: &mut Router) {
    router.get("/api/health", health);
}

async fn health(_req: Request) -> Result<Response, oxidite_core::Error> {
    Ok(Response::json(serde_json::json!({"status": "ok"})))
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
        fs::write(path.join("public/css/style.css"), "body { font-family: sans-serif; }")?;
        // JS
        fs::write(path.join("public/js/app.js"), "console.log('App loaded');")?;
        // Templates
        fs::write(path.join("templates/base.html"), "<html><body>{% block content %}{% endblock %}</body></html>")?;
        fs::write(path.join("templates/index.html"), "{% extends \"base.html\" %}{% block content %}<h1>Hello</h1>{% endblock %}")?;
    }

    Ok(())
}
