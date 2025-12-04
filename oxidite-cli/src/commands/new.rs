use std::fs;
use std::path::Path;

pub fn create_project(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Creating new Oxidite project: {}", name);
    
    // Create project directory
    fs::create_dir(name)?;
    let project_path = Path::new(name);
    
    // Createsubdirectories
    fs::create_dir(project_path.join("src"))?;
    fs::create_dir(project_path.join("src/models"))?;
    fs::create_dir(project_path.join("tests"))?;
    fs::create_dir(project_path.join("migrations"))?;
    
    // Create Cargo.toml
    let cargo_toml = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
oxidite-core = {{ git = "https://github.com/yourusername/oxidite", branch = "main" }}
oxidite-middleware = {{ git = "https://github.com/yourusername/oxidite", branch = "main" }}
tokio = {{ version = "1", features = ["full"] }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
"#, name);
    
    fs::write(project_path.join("Cargo.toml"), cargo_toml)?;
    
    // Create src/main.rs
    let main_rs = r#"use oxidite_core::{Router, Server, OxiditeRequest, OxiditeResponse, Result};
use oxid ite_middleware::{ServiceBuilder, LoggerLayer};
use http_body_util::Full;
use bytes::Bytes;

async fn index(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    Ok(hyper::Response::new(Full::new(Bytes::from("Hello from Oxidite!"))))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    router.get("/", index);
    
    let service = ServiceBuilder::new()
        .layer(LoggerLayer)
        .service(router);
    
    let server = Server::new(service);
    println!("Server running on http://127.0.0.1:3000");
    server.listen("127.0.0.1:3000".parse().unwrap()).await
}
"#;
    
    fs::write(project_path.join("src/main.rs"), main_rs)?;
    
    // Create README.md
    let readme = format!(r#"# {}

A new Oxidite web application.

## Getting Started

```bash
cargo run
```

## Database Migrations

Create a new migration:
```bash
oxidite migrate create create_users_table
```

Run pending migrations:
```bash
DATABASE_URL=postgres://localhost/mydb oxidite migrate run
```
"#, name);
    
    fs::write(project_path.join("README.md"), readme)?;
    
    // Create example migration
    let example_migration = r#"-- Migration: Create initial schema
-- Created at: 2025-01-01T00:00:00Z

-- Add migration
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Add rollback (optional)
-- DROP TABLE users;
"#;
    fs::write(project_path.join("migrations/20250101000000_create_initial_schema.sql"), example_migration)?;
    
    println!("✅ Project created successfully!");
    println!("  cd {}", name);
    println!("  cargo run");
    
    Ok(())
}
