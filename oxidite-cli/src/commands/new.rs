use colored::*;
use dialoguer::{theme::ColorfulTheme, Select};
use std::fs;
use std::path::Path;

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
            "web" => Some(Self::Fullstack),
            "api" => Some(Self::Api),
            "minimal" => Some(Self::Api),
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

pub fn create_project(
    name: &str,
    project_type: Option<String>,
    template: Option<String>,
    requested_features: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        format!("🚀 Initializing new Oxidite project: {}", name)
            .green()
            .bold()
    );

    let explicit_type = project_type
        .as_deref()
        .or(template.as_deref())
        .map(|value| {
            ProjectType::from_str(value).ok_or(
                "Invalid project type/template. Options: fullstack, web, api, minimal, microservice, serverless",
            )
        })
        .transpose()?;

    let p_type = if let Some(p_type) = explicit_type {
        p_type
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
            .items(selections)
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

    fs::create_dir(name)?;
    let project_path = Path::new(name);

    let src_path = project_path.join("src");
    fs::create_dir(&src_path)?;

    fs::create_dir(src_path.join("models"))?;
    fs::create_dir(src_path.join("routes"))?;
    fs::create_dir(src_path.join("controllers"))?;
    fs::create_dir(src_path.join("services"))?;
    fs::create_dir(src_path.join("middleware"))?;
    fs::create_dir(src_path.join("validators"))?;
    fs::create_dir(src_path.join("jobs"))?;
    fs::create_dir(src_path.join("policies"))?;
    fs::create_dir(src_path.join("events"))?;
    fs::create_dir(src_path.join("utils"))?;
    fs::create_dir(src_path.join("config"))?;
    fs::create_dir(project_path.join("tests"))?;
    fs::create_dir(project_path.join("migrations"))?;
    fs::create_dir(project_path.join("seeds"))?;

    match p_type {
        ProjectType::Fullstack => {
            fs::create_dir(project_path.join("templates"))?;
            fs::create_dir(project_path.join("public"))?;
            fs::create_dir(project_path.join("public/css"))?;
            fs::create_dir(project_path.join("public/js"))?;
            fs::create_dir(project_path.join("public/images"))?;
        }
        ProjectType::Microservice => {
            fs::create_dir(src_path.join("queues"))?;
        }
        _ => {}
    }

    create_cargo_toml(project_path, name)?;
    create_config_toml(project_path, p_type)?;
    create_main_rs(project_path, p_type)?;
    create_boilerplate(project_path, p_type)?;
    create_readme(project_path, name, p_type)?;

    let gitignore = r#"/target
Cargo.lock
*.db
*.log
.env
"#;
    fs::write(project_path.join(".gitignore"), gitignore)?;

    if !requested_features.is_empty() {
        println!(
            "{} {}",
            "ℹ️  Requested feature flags recorded but not scaffolded automatically:".yellow(),
            requested_features.join(", ")
        );
    }

    println!("\n{}", "✅ Project created successfully!".green().bold());
    println!("\n🚀 Next steps:");
    println!("  cd {}", name);
    println!("  oxidite migrate");
    println!("  oxidite dev");

    Ok(())
}

fn create_cargo_toml(path: &Path, name: &str) -> std::io::Result<()> {
    let content = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
oxidite = "2.1.0"
tokio = {{ version = "1", features = ["full"] }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
"#,
        name
    );

    fs::write(path.join("Cargo.toml"), content)
}

fn create_readme(path: &Path, name: &str, p_type: ProjectType) -> std::io::Result<()> {
    let project_kind = match p_type {
        ProjectType::Fullstack => "fullstack application",
        ProjectType::Api => "API service",
        ProjectType::Microservice => "microservice",
        ProjectType::Serverless => "serverless function",
    };

    let content = format!(
        r#"# {name}

Generated by `oxidite new` as an Oxidite {project_kind}.

## Development

```bash
oxidite dev
```

## Database

```bash
oxidite migrate
oxidite seed run
```
"#
    );

    fs::write(path.join("README.md"), content)
}

fn create_config_toml(path: &Path, p_type: ProjectType) -> std::io::Result<()> {
    let mut content = String::from(
        r#"# Oxidite Configuration

[server]
host = "127.0.0.1"
port = 8080
"#,
    );

    match p_type {
        ProjectType::Fullstack | ProjectType::Api => {
            content.push_str(
                r#"
[database]
url = "sqlite://./data.db"
"#,
            );
        }
        ProjectType::Microservice => {
            content.push_str(
                r#"
[queue]
redis_url = "redis://localhost"
"#,
            );
        }
        ProjectType::Serverless => {}
    }

    fs::write(path.join("oxidite.toml"), content)
}

fn create_main_rs(path: &Path, p_type: ProjectType) -> std::io::Result<()> {
    let content = match p_type {
        ProjectType::Fullstack => {
            r#"use oxidite::prelude::*;
use oxidite::template::serve_static;

mod routes;
mod controllers;
mod middleware;
mod models;
mod services;
mod validators;
mod jobs;
mod policies;
mod events;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::load()
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    let addr = format!("{}:{}", config.server.host, config.server.port);

    let mut router = Router::new();
    routes::register(&mut router);

    // Static files fallback
    router.get("/*", serve_static);

    let server = Server::new(router);
    println!("🚀 Server running on http://{}", addr);
    server.listen(addr.parse().unwrap()).await
}
"#
        }
        ProjectType::Api | ProjectType::Microservice => {
            r#"use oxidite::prelude::*;

mod routes;
mod controllers;
mod middleware;
mod models;
mod services;
mod validators;
mod jobs;
mod policies;
mod events;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::load()
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    let addr = format!("{}:{}", config.server.host, config.server.port);

    let mut router = Router::new();
    routes::register(&mut router);

    let server = Server::new(router);
    println!("🚀 Server running on http://{}", addr);
    server.listen(addr.parse().unwrap()).await
}
"#
        }
        ProjectType::Serverless => {
            r#"use oxidite::prelude::*;

async fn handler(_req: Request) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "message": "Hello from Serverless Function!"
    })))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    router.get("/", handler);

    let server = Server::new(router);
    println!("🚀 Function running locally on http://127.0.0.1:8080");
    server.listen("127.0.0.1:8080".parse().unwrap()).await
}
"#
        }
    };

    fs::write(path.join("src/main.rs"), content)
}

fn create_boilerplate(path: &Path, p_type: ProjectType) -> std::io::Result<()> {
    fs::write(
        path.join("src/models/mod.rs"),
        "// Generated model modules are exported here.\n",
    )?;
    fs::write(
        path.join("src/controllers/mod.rs"),
        "// Generated controller modules are exported here.\n",
    )?;
    fs::write(
        path.join("src/services/mod.rs"),
        "// Generated service modules are exported here.\n",
    )?;
    fs::write(
        path.join("src/middleware/mod.rs"),
        "// Generated middleware modules are exported here.\n",
    )?;
    fs::write(
        path.join("src/validators/mod.rs"),
        "// Generated validator modules are exported here.\n",
    )?;
    fs::write(
        path.join("src/jobs/mod.rs"),
        "// Generated job modules are exported here.\n",
    )?;
    fs::write(
        path.join("src/policies/mod.rs"),
        "// Generated policy modules are exported here.\n",
    )?;
    fs::write(
        path.join("src/events/mod.rs"),
        "// Generated event modules are exported here.\n",
    )?;

    let routes_content = match p_type {
        ProjectType::Fullstack => {
            r#"use oxidite::prelude::*;
use oxidite::template::{Context, TemplateEngine};

pub fn register(router: &mut Router) {
    router.get("/", index);
    register_generated(router);
}

fn register_generated(router: &mut Router) {
    let _ = router;
    // Generated route modules are registered here automatically.
}

async fn index(_req: Request) -> Result<Response> {
    let mut engine = TemplateEngine::new();
    engine
        .load_dir("templates")
        .map_err(|e| Error::InternalServerError(e.to_string()))?;

    let mut context = Context::new();
    context.set("name", "Oxidite");

    let body = engine
        .render("index.html", &context)
        .map_err(|e| Error::InternalServerError(e.to_string()))?;

    Ok(Response::html(body))
}
"#
        }
        ProjectType::Api => {
            r#"use oxidite::prelude::*;

pub fn register(router: &mut Router) {
    router.get("/api/health", health);
    register_generated(router);
}

fn register_generated(router: &mut Router) {
    let _ = router;
    // Generated route modules are registered here automatically.
}

async fn health(_req: Request) -> Result<Response> {
    Ok(Response::json(serde_json::json!({"status": "ok"})))
}
"#
        }
        _ => {
            r#"use oxidite::prelude::*;

pub fn register(router: &mut Router) {
    register_generated(router);
}

fn register_generated(router: &mut Router) {
    let _ = router;
    // Generated route modules are registered here automatically.
}
"#
        }
    };
    fs::write(path.join("src/routes/mod.rs"), routes_content)?;

    if let ProjectType::Fullstack = p_type {
        let css_content = r#"
body {
    font-family: system-ui, -apple-system, Segoe UI, Roboto, sans-serif;
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 100vh;
    margin: 0;
    background-color: #0f172a;
    color: #e2e8f0;
}

.container {
    text-align: center;
    padding: 40px;
    background-color: #1e293b;
    border-radius: 12px;
}
"#;
        fs::write(path.join("public/css/style.css"), css_content)?;
        fs::write(path.join("public/js/app.js"), "console.log('App loaded');")?;

        let template_content = r#"<!doctype html>
<html>
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Oxidite</title>
  <link rel="stylesheet" href="/css/style.css">
</head>
<body>
  <div class="container">
    <h1>Hello, {{ name }}!</h1>
    <p>Your Oxidite app is running.</p>
  </div>
  <script src="/js/app.js"></script>
</body>
</html>
"#;
        fs::write(path.join("templates/index.html"), template_content)?;
    }

    Ok(())
}
