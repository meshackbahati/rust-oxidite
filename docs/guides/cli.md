# CLI Tool Guide

The Oxidite CLI helps you scaffold projects, generate code, and manage your application.

## Installation

```bash
cargo install --path oxidite-cli
```

## Commands

### Create New Project

```bash
# Basic project
oxidite new myapp

# With specific type
oxidite new myapp --project-type api
oxidite new myapp --project-type fullstack
oxidite new myapp --project-type microservice
```

### Code Generation

```bash
# Generate model
oxidite make model User

# Generate controller
oxidite make controller UserController

# Generate middleware
oxidite make middleware AuthMiddleware
```

### Database Migrations

```bash
# Create migration
oxidite migrate create create_users_table

# Run migrations
oxidite migrate run

# Rollback last migration
oxidite migrate revert

# Check status
oxidite migrate status
```

### Database Seeders

```bash
# Create seeder
oxidite seed create UserSeeder

# Run seeders
oxidite seed run
```

### Queue Management

```bash
# Start workers
oxidite queue work --workers 4

# View statistics
oxidite queue list

# View dead letter queue
oxidite queue dlq

# Clear pending jobs
oxidite queue clear
```

### Health Check

```bash
# System diagnostics
oxidite doctor
```

### Build

```bash
# Development build
oxidite build

# Production build  
oxidite build --release
```

### Development Server

```bash
# Start with hot reload
oxidite dev
```

## Project Structure

After `oxidite new myapp`, you get:

```
myapp/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── models/
│   ├── controllers/
│   └── middleware/
├── migrations/
├── seeders/
├── templates/
└── config.toml
```

## Configuration

Oxidite projects use `config.toml`:

```toml
[server]
host = "127.0.0.1"
port = 8080

[database]
url = "postgresql://localhost/myapp"

[cache]
url = "redis://127.0.0.1"

[queue]
url = "redis://127.0.0.1"
```

## Tips

- Use `oxidite doctor` to debug issues
- Run `oxidite migrate status` before deploying
- Use `oxidite dev` for automatic reloading during development
