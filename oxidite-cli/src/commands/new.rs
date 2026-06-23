use colored::*;
use dialoguer::{theme::ColorfulTheme, Select};
use std::fs;
use std::path::Path;

// Embed the Oxidite SVG logo at compile time so `oxidite new` can write it
// to the generated project's `public/images/` directory.
const OXIDITE_LOGO_SVG: &str = include_str!("../templates/oxidite.svg");

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
        format!("Initializing new Oxidite project: {}", name)
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

    // Create the project directory structure.
    fs::create_dir(name)?;
    let project_path = Path::new(name);

    let src_path = project_path.join("src");
    fs::create_dir(&src_path)?;

    // Standard application module directories.
    // Each one has a mod.rs so you can add submodules immediately.
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
    create_test_file(project_path, p_type)?;
    fs::create_dir(project_path.join("migrations"))?;
    fs::create_dir(project_path.join("seeds"))?;

    match p_type {
        ProjectType::Fullstack => {
            // Fullstack projects serve HTML templates and static assets.
            // Files in `public/` are served at the root URL path, e.g.:
            //   public/css/style.css   →  /css/style.css
            //   public/images/logo.svg  →  /images/logo.svg
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

    // .gitignore — keep secrets and build artefacts out of version control.
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
            "Requested feature flags recorded but not scaffolded automatically:".yellow(),
            requested_features.join(", ")
        );
    }

    println!("\n{}", "Project created successfully!".green().bold());
    println!("\nNext steps:");
    println!("  cd {}", name);
    println!("  oxidite migrate");
    println!("  oxidite dev");

    Ok(())
}

// ---------------------------------------------------------------------------
// Test files (scaffolded for all project types)
// ---------------------------------------------------------------------------

fn create_test_file(path: &Path, p_type: ProjectType) -> std::io::Result<()> {
    let test_content = match p_type {
        ProjectType::Serverless => r#"use oxidite_testing::{TestRequest, test_router};

#[tokio::test]
async fn test_health_endpoint() {
    let mut router = oxidite::Router::new();
    router.get("/", handler);

    let mut server = test_router(router);
    let req = TestRequest::get("/").build_oxidite();
    let resp = server.call(req).await.unwrap();
    assert!(resp.status().is_success());
}
"#,
        _ => r#"use oxidite_testing::{TestRequest, TestResponse, test_router};
use oxidite::prelude::*;

/// Integration test for the health-check endpoint.
#[tokio::test]
async fn test_health_endpoint() {
    let mut router = Router::new();
    router.get("/", || async { Ok(Response::json_val(json!({"status": "ok"}))) });

    let mut server = test_router(router);
    let req = TestRequest::get("/").build_oxidite();
    let resp = server.call(req).await.unwrap();

    let test_resp = TestResponse::from_oxidite_response(resp).await;
    assert!(test_resp.is_success());
}
"#,
    };

    fs::write(path.join("tests/integration_test.rs"), test_content)
}

// ---------------------------------------------------------------------------
// Cargo.toml
// ---------------------------------------------------------------------------

fn create_cargo_toml(path: &Path, name: &str) -> std::io::Result<()> {
    let content = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
# Oxidite — a modern, batteries-included Rust web framework.
# The "full" feature flag enables all sub-crates (auth, db, cache, etc.).
oxidite = {{ version = "2.3.3-1", features = ["full"] }}

# Async runtime.
tokio = {{ version = "1", features = ["full"] }}

# Serialization / deserialization.
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"

[dev-dependencies]
# Testing utilities for integration tests.
oxidite-testing = "2.3.3-1"
"#,
        name
    );

    fs::write(path.join("Cargo.toml"), content)
}

// ---------------------------------------------------------------------------
// README
// ---------------------------------------------------------------------------

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

## Quick Start

```bash
# Install the CLI (if you haven't already)
cargo install oxidite-cli

# Run database migrations
oxidite migrate

# Start the development server with hot-reload
oxidite dev
```

## Project Structure

```
{name}/
  src/
    main.rs            # Application entry point
    routes/mod.rs      # Route registration
    controllers/       # Request handlers
    models/            # Data models and ORM structs
    services/          # Business logic
    middleware/         # Custom middleware
    validators/        # Input validation
    jobs/              # Background jobs
    policies/          # Authorization policies
    events/            # Event handlers
  oxidite.toml         # Framework configuration
  migrations/          # SQL migration files
  seeds/               # Database seed files
  tests/               # Integration tests
```

## Useful Commands

| Command              | Description                          |
|----------------------|--------------------------------------|
| `oxidite dev`        | Start dev server with hot-reload     |
| `oxidite migrate`    | Run pending migrations               |
| `oxidite seed run`   | Seed the database                    |
| `oxidite doctor`     | Health check for your project        |
| `oxidite make model` | Scaffold a new model                 |
"#
    );

    fs::write(path.join("README.md"), content)
}

// ---------------------------------------------------------------------------
// oxidite.toml
// ---------------------------------------------------------------------------

fn create_config_toml(path: &Path, p_type: ProjectType) -> std::io::Result<()> {
    let mut content = String::from(
        r#"# Oxidite Configuration
#
# Environment Variables — three strategies (all produce the same env::var() results):
#
# 1. [env] flat table — keys map directly to env vars:
#        [env]
#        DATABASE_URL = "postgres://..."
#        API_KEY = "secret"
#
# 2. Namespaced tables — table name becomes an UPPERCASE prefix:
#        [google]
#        client_id = "abc"        -> GOOGLE_CLIENT_ID
#        client_secret = "xyz"    -> GOOGLE_CLIENT_SECRET
#
# 3. Nested tables — flattened recursively:
#        [google.oauth]
#        client_id = "abc"        -> GOOGLE_OAUTH_CLIENT_ID
#
# .env files also work and take priority over oxidite.toml values.
# Real OS environment variables always take highest priority.
#
# All strategies are interchangeable — use whichever fits best.

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

# Define env vars here or in a .env file. Examples:
#
# [env]
# JWT_SECRET = "change-me"
#
# [platform]
# name = "my-app"              -> PLATFORM_NAME
"#,
            );
        }
        ProjectType::Microservice => {
            content.push_str(
                r#"
[queue]
redis_url = "redis://localhost"

# Define env vars here or in a .env file. Examples:
#
# [env]
# JWT_SECRET = "change-me"
#
# [broker]
# url = "redis://localhost"    -> BROKER_URL
"#,
            );
        }
        ProjectType::Serverless => {
            content.push_str(
                r#"
# Define env vars here or in a .env file. Examples:
#
# [env]
# API_KEY = "your-key"
#
# [provider]
# region = "us-east-1"         -> PROVIDER_REGION
"#,
            );
        }
    }

    fs::write(path.join("oxidite.toml"), content)
}

// ---------------------------------------------------------------------------
// src/main.rs
// ---------------------------------------------------------------------------

fn create_main_rs(path: &Path, p_type: ProjectType) -> std::io::Result<()> {
    let content = match p_type {
        ProjectType::Fullstack => {
            r#"// Application entry point.
//
// This sets up the Oxidite server, loads configuration, registers routes,
// and serves static files from the `public/` directory.

use oxidite::prelude::*;
use oxidite::template::serve_static;

// Each module maps to a directory under `src/`.
// Use `oxidite make <kind> <name>` to scaffold new modules.
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
    // Load configuration from oxidite.toml, .env, and OS env vars.
    let config = Config::load()
        .map_err(|e| Error::InternalServerError(e.to_string()))?;

    let addr = format!("{}:{}", config.server.host, config.server.port);

    // Create the application router and register all routes.
    let mut router = Router::new();
    routes::register(&mut router);

    // Serve static files from `public/` as a catch-all fallback.
    // e.g. GET /css/style.css  ->  public/css/style.css
    //      GET /images/logo.svg ->  public/images/logo.svg
    router.get("/*", serve_static);

    let server = Server::new(router);
    println!("Server running on http://{}", addr);
    server.listen(addr.parse().unwrap()).await
}
"#
        }
        ProjectType::Api | ProjectType::Microservice => {
            r#"// Application entry point.
//
// This sets up the Oxidite API server, loads configuration, and registers
// all route handlers.

use oxidite::prelude::*;

// Each module maps to a directory under `src/`.
// Use `oxidite make <kind> <name>` to scaffold new modules.
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
    // Load configuration from oxidite.toml, .env, and OS env vars.
    let config = Config::load()
        .map_err(|e| Error::InternalServerError(e.to_string()))?;

    let addr = format!("{}:{}", config.server.host, config.server.port);

    // Create the application router and register all routes.
    let mut router = Router::new();
    routes::register(&mut router);

    let server = Server::new(router);
    println!("Server running on http://{}", addr);
    server.listen(addr.parse().unwrap()).await
}
"#
        }
        ProjectType::Serverless => {
            r#"// Serverless function entry point.
//
// This defines a single HTTP handler that responds to requests.
// Deploy it behind any HTTP trigger (AWS Lambda, Cloudflare Workers, etc.).

use oxidite::prelude::*;

// The request handler — replace this with your own logic.
async fn handler(_req: Request) -> Result<Response> {
    Ok(Response::json_val(json!({
        "message": "Hello from Serverless Function!"
    })))
}

#[tokio::main]
async fn main() -> Result<()> {
    // For local development, mount the handler on a router.
    let mut router = Router::new();
    router.get("/", handler);

    let server = Server::new(router);
    println!("Function running locally on http://127.0.0.1:8080");
    server.listen("127.0.0.1:8080".parse().unwrap()).await
}
"#
        }
    };

    fs::write(path.join("src/main.rs"), content)
}

// ---------------------------------------------------------------------------
// Boilerplate files (routes, modules, static assets, templates)
// ---------------------------------------------------------------------------

fn create_boilerplate(path: &Path, p_type: ProjectType) -> std::io::Result<()> {
    // Module barrel files — each one re-exports submodules in its directory.
    // Run `oxidite make <kind> <name>` to create new files inside them.
    let module_files: &[(&str, &str)] = &[
        ("src/models/mod.rs",
         "// Data models and ORM structs are defined here.\n\
          // Use `oxidite make model User name:string email:string` to scaffold.\n"),
        ("src/controllers/mod.rs",
         "// Controllers handle HTTP requests and return responses.\n\
          // Use `oxidite make controller Auth` to scaffold.\n"),
        ("src/services/mod.rs",
         "// Services contain business logic, keeping controllers thin.\n\
          // Use `oxidite make service Payment` to scaffold.\n"),
        ("src/middleware/mod.rs",
         "// Middleware runs before/after request handlers.\n\
          // Use `oxidite make middleware RequireAuth` to scaffold.\n"),
        ("src/validators/mod.rs",
         "// Validators check and sanitize user input.\n\
          // Use `oxidite make validator CreateUser` to scaffold.\n"),
        ("src/jobs/mod.rs",
         "// Background jobs for async processing (emails, reports, etc.).\n\
          // Use `oxidite make job SendWelcomeEmail` to scaffold.\n"),
        ("src/policies/mod.rs",
         "// Authorization policies control access to resources.\n\
          // Use `oxidite make policy PostPolicy` to scaffold.\n"),
        ("src/events/mod.rs",
         "// Event handlers react to domain events.\n\
          // Use `oxidite make event UserRegistered` to scaffold.\n"),
    ];

    for (file_path, content) in module_files {
        fs::write(path.join(file_path), content)?;
    }

    // Routes — register all HTTP handlers here.
    let routes_content = match p_type {
        ProjectType::Fullstack => {
            r#"// Route registration.
//
// Add your routes inside `register()`. The `register_generated()` function
// is where `oxidite make route` inserts new route bindings automatically.

use oxidite::prelude::*;
use oxidite::template::{Context, TemplateEngine};

/// Register all application routes on the router.
pub fn register(router: &mut Router) {
    // The root path renders the welcome page.
    router.get("/", index);

    // Auto-generated routes are inserted here by the CLI.
    register_generated(router);
}

fn register_generated(router: &mut Router) {
    let _ = router;
    // `oxidite make route` adds new route lines below this comment.
}

/// Welcome page handler — renders `templates/index.html`.
///
/// The template engine loads HTML files from the `templates/` directory
/// and fills in `{{ variable }}` placeholders using the context.
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
            r#"// Route registration.
//
// Add your API routes inside `register()`. The `register_generated()` function
// is where `oxidite make route` inserts new route bindings automatically.

use oxidite::prelude::*;

/// Register all API routes on the router.
pub fn register(router: &mut Router) {
    // Health-check endpoint — useful for load balancers and monitoring.
    router.get("/api/health", health);

    // Auto-generated routes are inserted here by the CLI.
    register_generated(router);
}

fn register_generated(router: &mut Router) {
    let _ = router;
    // `oxidite make route` adds new route lines below this comment.
}

/// Returns a simple JSON health-check response.
async fn health(_req: Request) -> Result<Response> {
    Ok(Response::json_val(json!({"status": "ok"})))
}
"#
        }
        _ => {
            r#"// Route registration.
//
// Add your routes inside `register()`. The `register_generated()` function
// is where `oxidite make route` inserts new route bindings automatically.

use oxidite::prelude::*;

/// Register all routes on the router.
pub fn register(router: &mut Router) {
    // Auto-generated routes are inserted here by the CLI.
    register_generated(router);
}

fn register_generated(router: &mut Router) {
    let _ = router;
    // `oxidite make route` adds new route lines below this comment.
}
"#
        }
    };
    fs::write(path.join("src/routes/mod.rs"), routes_content)?;

    // Fullstack-specific: static assets, logo, and HTML template.
    if let ProjectType::Fullstack = p_type {
        // ---- CSS (with comments for beginners) ----
        let css_content = r#"/* Oxidite default styles.
 *
 * These styles create a simple centred card layout for the welcome page.
 * Replace or extend them to match your design.
 */

body {
    /* System font stack for fast, native-looking text. */
    font-family: system-ui, -apple-system, Segoe UI, Roboto, sans-serif;

    /* Centre the card both horizontally and vertically. */
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 100vh;
    margin: 0;

    /* Dark background. */
    background-color: #0f172a;
    color: #e2e8f0;
}

.container {
    text-align: center;
    padding: 40px;
    background-color: #1e293b;
    border-radius: 12px;
}

/* Logo sizing — adjust width/height to taste. */
.logo {
    width: 200px;
    height: auto;
    margin-bottom: 24px;
}

h1 {
    margin: 0 0 8px;
    font-size: 2rem;
}

p {
    margin: 0;
    opacity: 0.8;
}
"#;
        fs::write(path.join("public/css/style.css"), css_content)?;

        // ---- JS placeholder ----
        let js_content = r#"// Client-side JavaScript.
// This file is served at /js/app.js from the public/ directory.
console.log('Oxidite app loaded');
"#;
        fs::write(path.join("public/js/app.js"), js_content)?;

        // ---- Oxidite SVG logo ----
        // Written as a static file in public/images/.
        // Referenced in HTML as <img src="/images/oxidite.svg">.
        // Developers can delete or replace this file freely.
        fs::write(path.join("public/images/oxidite.svg"), OXIDITE_LOGO_SVG)?;

        // ---- HTML welcome template ----
        // Uses Handlebars-style {{ variable }} placeholders.
        // The logo and favicon both reference files in public/.
        let template_content = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Hello, Oxidite</title>

  <!-- Favicon — the Oxidite logo from public/images/.
       Delete this line or point it to your own icon. -->
  <link rel="icon" type="image/svg+xml" href="/images/oxidite.svg">

  <!-- Styles from public/css/style.css -->
  <link rel="stylesheet" href="/css/style.css">
</head>
<body>
  <div class="container">
    <!-- Logo — served from public/images/oxidite.svg.
         Remove this <img> or replace the src with your own logo. -->
    <img class="logo" src="/images/oxidite.svg" alt="Oxidite logo">

    <h1>Hello, {{ name }}!</h1>
    <p>Your Oxidite app is running. Start building something great.</p>
  </div>

  <!-- Client-side JS from public/js/app.js -->
  <script src="/js/app.js"></script>
</body>
</html>
"#;
        fs::write(path.join("templates/index.html"), template_content)?;
    }

    Ok(())
}
