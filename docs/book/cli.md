# CLI Tools

The Oxidite CLI provides powerful command-line tools for project scaffolding, code generation, and development workflows. This chapter covers all available CLI commands and their usage.

## Overview

The Oxidite CLI includes:
- Project creation and scaffolding
- Code generation tools
- Database migrations
- Development server with hot reload
- Build and deployment utilities

## Installation

Install the Oxidite CLI globally:

```bash
# Install via cargo
cargo install oxidite-cli

# Or install a specific version
cargo install oxidite-cli --version 2.0.0

# Verify installation
oxidite --version
```

## Project Scaffolding

Create new projects with the CLI:

```bash
# Create a new Oxidite project
oxidite new my_app

# Create with specific features
oxidite new my_app --features "db,auth,api"

# Create with template
oxidite new my_app --template api
oxidite new my_app --template web
oxidite new my_app --template fullstack
```

The CLI creates a complete project structure:

```
my_app/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── routes/
│   ├── models/
│   ├── middleware/
│   └── lib.rs
├── migrations/
├── templates/
├── public/
├── config/
│   └── default.toml
├── tests/
├── docs/
└── README.md
```

### Project Templates

Different templates for various use cases:

```bash
# API-only template
oxidite new api_project --template api

# Web application template
oxidite new web_project --template web

# Full-stack application template
oxidite new fullstack_project --template fullstack

# Minimal template
oxidite new minimal_project --template minimal
```

## Code Generation

Generate boilerplate code automatically:

```bash
# Generate a new route
oxidite generate route users

# Generate a model
oxidite generate model User name:string email:string age:integer

# Generate a controller
oxidite generate controller Posts

# Generate a middleware
oxidite generate middleware auth

# Generate a migration
oxidite generate migration create_users_table

# Generate a component (for full-stack apps)
oxidite generate component UserCard
```

### Model Generation

Create database models with the generator:

```bash
# Basic model
oxidite generate model User

# Model with fields
oxidite generate model Post title:string content:text author_id:integer published:boolean

# Model with relationships
oxidite generate model Comment content:text user_id:integer post_id:integer

# Model with custom options
oxidite generate model Product \
    name:string \
    price:decimal \
    description:text \
    --timestamps \
    --uuid-primary-key \
    --with-soft-delete
```

The generated model looks like:

```rust
use oxidite::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Model, Serialize, Deserialize)]
#[model(table = "products")]
pub struct Product {
    #[model(primary_key)]
    pub id: i32,
    #[model(not_null)]
    pub name: String,
    #[model(not_null)]
    pub price: f64,
    pub description: Option<String>,
    #[model(created_at)]
    pub created_at: String,
    #[model(updated_at)]
    pub updated_at: String,
}
```

### Route Generation

Generate route files:

```bash
# Generate basic CRUD routes
oxidite generate route users

# Generate routes for a specific model
oxidite generate route --model User

# Generate API routes
oxidite generate route --api posts

# Generate routes with custom template
oxidite generate route --template api-crud products
```

Generated route file:

```rust
use oxidite::prelude::*;
use crate::models::User;

pub fn configure_routes(router: &mut Router) {
    router.get("/users", get_all_users);
    router.get("/users/:id", get_user);
    router.post("/users", create_user);
    router.put("/users/:id", update_user);
    router.delete("/users/:id", delete_user);
}

async fn get_all_users(_req: Request) -> Result<Response> {
    // Implementation here
    Ok(Response::json(serde_json::json!({})))
}

async fn get_user(Path(id): Path<i32>) -> Result<Response> {
    // Implementation here
    Ok(Response::json(serde_json::json!({})))
}

async fn create_user(_req: Request) -> Result<Response> {
    // Implementation here
    Ok(Response::json(serde_json::json!({})))
}

async fn update_user(Path(id): Path<i32>, _req: Request) -> Result<Response> {
    // Implementation here
    Ok(Response::json(serde_json::json!({})))
}

async fn delete_user(Path(id): Path<i32>) -> Result<Response> {
    // Implementation here
    Ok(Response::json(serde_json::json!({})))
}
```

## Database Commands

Manage your database with CLI tools:

```bash
# Run pending migrations
oxidite migrate

# Create a new migration
oxidite generate migration add_email_to_users

# Rollback last migration
oxidite migrate:rollback

# Drop all tables (dangerous!)
oxidite migrate:reset

# Seed the database
oxidite db:seed

# Generate migration from model changes
oxidite migrate:generate
```

### Migration Generation

Automatically generate migrations from model changes:

```bash
# Generate migration based on model differences
oxidite generate migration --auto

# Generate migration for a specific model
oxidite generate migration --model User --add-field phone:string

# Generate migration for model deletion
oxidite generate migration --model OldTable --delete-table
```

Example generated migration:

```rust
use oxidite_db::Migration;

pub struct AddPhoneToUsers;

impl Migration for AddPhoneToUsers {
    fn version(&self) -> i64 {
        20240121120000
    }
    
    fn name(&self) -> &'static str {
        "add_phone_to_users"
    }
    
    fn up(&self) -> &'static str {
        r#"
        ALTER TABLE users ADD COLUMN phone VARCHAR(20);
        "#
    }
    
    fn down(&self) -> &'static str {
        r#"
        ALTER TABLE users DROP COLUMN phone;
        "#
    }
}
```

## Development Server

Run your application with the development server:

```bash
# Start development server
oxidite dev

# Start with specific port
oxidite dev --port 8080

# Start with hot reload disabled
oxidite dev --no-hot-reload

# Start with specific environment
oxidite dev --env development

# Watch specific directories
oxidite dev --watch src --watch templates
```

### Hot Reload Configuration

The development server supports hot reload for rapid development:

```bash
# Enable hot reload (default)
oxidite dev --hot-reload

# Specify files/directories to watch
oxidite dev --watch src/**/*.rs --watch templates/**/*.html

# Ignore specific files
oxidite dev --ignore target --ignore node_modules
```

## Build and Production Commands

Prepare your application for production:

```bash
# Build for release
oxidite build

# Build with specific profile
oxidite build --profile release

# Build for specific target
oxidite build --target x86_64-unknown-linux-musl

# Build with features
oxidite build --features "postgres,redis"

# Create optimized build
oxidite build --release
```

### Production Deployment

Deploy your application:

```bash
# Deploy to production
oxidite deploy

# Deploy to specific environment
oxidite deploy --env production

# Deploy with specific configuration
oxidite deploy --config prod.toml

# Dry run deployment
oxidite deploy --dry-run
```

## Configuration Management

Manage application configuration:

```bash
# Generate configuration file
oxidite config:generate

# Validate configuration
oxidite config:validate

# Show current configuration
oxidite config:show

# Set configuration values
oxidite config:set database.url postgresql://localhost/myapp

# Encrypt sensitive configuration
oxidite config:encrypt --key SECRET_KEY
```

Configuration file structure:

```toml
[server]
port = 3000
host = "127.0.0.1"
workers = 4
timeout = 30

[database]
url = "postgresql://localhost/myapp"
pool_size = 10
timeout = 30

[authentication]
jwt_secret = "your-secret-key"
token_expiration = 86400

[logging]
level = "info"
format = "json"
output = "stdout"

[cache]
backend = "redis"
url = "redis://127.0.0.1:6379"
ttl = 3600
```

## Testing Commands

Run tests with the CLI:

```bash
# Run all tests
oxidite test

# Run specific test
oxidite test test_name

# Run tests with specific features
oxidite test --features integration

# Run tests in watch mode
oxidite test --watch

# Run tests with coverage
oxidite test --coverage

# Run only unit tests
oxidite test --unit

# Run only integration tests
oxidite test --integration
```

## Plugin Management

Manage Oxidite plugins:

```bash
# List installed plugins
oxidite plugin:list

# Install a plugin
oxidite plugin:install plugin-name

# Install plugin from git
oxidite plugin:install git+https://github.com/user/plugin.git

# Uninstall plugin
oxidite plugin:uninstall plugin-name

# Update plugins
oxidite plugin:update

# Search for plugins
oxidite plugin:search keyword
```

## Custom Commands

Create custom CLI commands:

```bash
# Generate a custom command
oxidite generate command backup

# Create command with arguments
oxidite generate command import --args "source:string destination:string"
```

Custom command template:

```rust
use clap::Parser;
use oxidite_cli::Command;

#[derive(Parser)]
#[clap(name = "backup")]
pub struct BackupCommand {
    #[clap(short, long)]
    pub output: Option<String>,
    
    #[clap(short, long)]
    pub compress: bool,
}

#[async_trait::async_trait]
impl Command for BackupCommand {
    async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Running backup command");
        
        if let Some(output) = &self.output {
            println!("Output to: {}", output);
        }
        
        if self.compress {
            println!("Compression enabled");
        }
        
        // Your backup logic here
        
        Ok(())
    }
}
```

Register the command in your CLI:

```rust
use oxidite_cli::{Cli, Command};

pub struct MyAppCli {
    commands: Vec<Box<dyn Command>>,
}

impl MyAppCli {
    pub fn new() -> Self {
        let mut cli = Self {
            commands: vec![],
        };
        
        cli.register_command(Box::new(BackupCommand::parse_from(["backup"])));
        
        cli
    }
    
    pub fn register_command(&mut self, command: Box<dyn Command>) {
        self.commands.push(command);
    }
    
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Execute the command
        Ok(())
    }
}
```

## Help and Documentation

Get help and documentation:

```bash
# Show help
oxidite --help
oxidite help
oxidite -h

# Show help for specific command
oxidite migrate --help
oxidite generate --help

# Show version
oxidite --version
oxidite -V

# Show verbose output
oxidite migrate --verbose
oxidite build -v
```

## Environment-Specific Commands

Handle different environments:

```bash
# Development environment
oxidite dev --env development

# Staging environment
oxidite deploy --env staging

# Production environment
oxidite deploy --env production

# Load environment-specific configuration
oxidite --config config/staging.toml
```

## Task Automation

Run automated tasks:

```bash
# Run predefined tasks
oxidite run setup
oxidite run cleanup
oxidite run seed-users

# List available tasks
oxidite run --list

# Run task with arguments
oxidite run backup -- --output /tmp/backup.sql
```

Task definition in `tasks.rs`:

```rust
use oxidite_cli::Task;

pub struct SetupTask;

impl Task for SetupTask {
    fn name(&self) -> &str { "setup" }
    
    fn description(&self) -> &str { "Set up the application" }
    
    async fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Setting up application...");
        
        // Run migrations
        // Create directories
        // Set up initial data
        
        println!("Setup complete!");
        Ok(())
    }
}
```

## Performance Profiling

Profile application performance:

```bash
# Profile application
oxidite profile

# Profile with specific duration
oxidite profile --duration 30s

# Profile specific endpoint
oxidite profile --endpoint /api/users

# Generate flame graph
oxidite profile --flamegraph
```

## Security Scanning

Scan for security vulnerabilities:

```bash
# Scan dependencies
oxidite security:audit

# Scan for common vulnerabilities
oxidite security:scan

# Generate security report
oxidite security:report

# Fix known vulnerabilities
oxidite security:fix
```

## Docker Integration

Work with Docker containers:

```bash
# Generate Dockerfile
oxidite docker:generate

# Build Docker image
oxidite docker:build

# Run in Docker
oxidite docker:run

# Push to registry
oxidite docker:push

# Generate docker-compose
oxidite docker:compose
```

Dockerfile template:

```dockerfile
FROM rust:1.92 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/my_app /usr/local/bin/my_app
EXPOSE 3000
CMD ["my_app"]
```

## Summary

The Oxidite CLI provides comprehensive tools for:

- **Project scaffolding**: Quick project creation with templates
- **Code generation**: Automate repetitive coding tasks
- **Database management**: Migrations and seeding
- **Development workflow**: Hot reload and testing
- **Building and deployment**: Production-ready builds
- **Configuration management**: Environment-specific configs
- **Task automation**: Custom command creation
- **Performance profiling**: Application optimization
- **Security scanning**: Vulnerability detection

The CLI streamlines development workflows and enforces best practices across your Oxidite projects.