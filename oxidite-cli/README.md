# oxidite-cli

Command-line interface for the Oxidite web framework. Provides tools for project scaffolding, code generation, migrations, and development server management.

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/oxidite-cli.svg)](https://crates.io/crates/oxidite-cli)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

</div>

## Overview

`oxidite-cli` is the official command-line tool for the Oxidite web framework. It streamlines development workflows by providing commands for project creation, code generation, database migrations, and development server management.

## Installation

Install the CLI tool globally:

```bash
# Install from crates.io (when published)
cargo install oxidite-cli

# Or install from local source
cargo install --path .
```

## Features

- **Project scaffolding** - Create new Oxidite projects with a single command
- **Code generation** - Generate models, controllers, middleware, and other components
- **Migration management** - Create, run, and rollback database migrations
- **Development server** - Hot-reloading development server with file watching
- **Interactive setup** - Guided project creation with configuration options
- **Health checks** - Diagnose and troubleshoot common issues
- **Asset management** - Build and optimize static assets

## Usage

### Creating a New Project

Generate a new Oxidite project:

```bash
# Interactive project creation
oxidite new my-app

# The interactive wizard will guide you through:
# - Project type selection (Fullstack, API, Microservice, Serverless)
# - Database configuration
# - Feature selection
# - Directory structure setup
```

### Development Server

Start the development server with hot reloading:

```bash
# Navigate to your project directory
cd my-app

# Start the development server
oxidite dev

# The server will watch for file changes and automatically restart
# Available at http://127.0.0.1:8080 by default
```

### Code Generation

Generate common components:

```bash
# Generate a model
oxidite make model User

# Generate a controller
oxidite make controller Users

# Generate middleware
oxidite make middleware Auth

# Generate a migration
oxidite make migration create_users_table
```

### Database Migrations

Manage your database schema:

```bash
# Create a new migration
oxidite migrate create add_email_to_users

# Run pending migrations
oxidite migrate run

# Rollback the last migration
oxidite migrate rollback

# View migration status
oxidite migrate status
```

### Health Checks

Diagnose common issues:

```bash
# Run system diagnostics
oxidite doctor

# This checks:
# - Rust installation
# - Database connectivity
# - Configuration files
# - Dependency versions
# - Common setup issues
```

### Full Command Reference

```bash
# Show help
oxidite --help
oxidite new --help
oxidite dev --help

# Project commands
oxidite new <project-name>     # Create a new project
oxidite build                 # Build the project
oxidite serve                 # Start production server

# Development commands
oxidite dev                   # Start development server
oxidite watch                 # Watch files and run tests/builds

# Code generation
oxidite make model <name>     # Generate a model
oxidite make controller <name> # Generate a controller
oxidite make middleware <name> # Generate middleware
oxidite make migration <name> # Generate a migration

# Database commands
oxidite migrate create <name> # Create migration
oxidite migrate run          # Run migrations
oxidite migrate rollback     # Rollback migrations
oxidite migrate status       # Show migration status

# Utility commands
oxidite doctor              # Run health checks
oxidite clean               # Clean build artifacts
oxidite version             # Show version information
```

### Project Types

The CLI supports different project types:

- **Fullstack Application**: Complete setup with templates, static files, and database
- **REST API**: Optimized for backend services (JSON only)
- **Microservice**: Minimal setup for specialized services
- **Serverless Function**: Lightweight event handler

### Configuration

The CLI reads configuration from `config.toml` in your project root:

```toml
[app]
name = "my-app"
port = 3000
environment = "development"

[database]
url = "sqlite::memory:"
migrations_dir = "./migrations"

[server]
host = "127.0.0.1"
workers = 4
timeout = 30
```

### Environment Variables

The CLI respects common environment variables:

```bash
# Override the port
OXIDITE_PORT=8080 oxidite dev

# Use a different database
DATABASE_URL=postgresql://user:pass@localhost/db oxidite dev

# Set environment
OXIDITE_ENV=production oxidite serve
```

## Integration with Oxidite

The CLI is designed to work seamlessly with Oxidite projects:

- Automatically generates Oxidite-compatible code
- Sets up proper directory structure
- Configures dependencies in Cargo.toml
- Creates appropriate example code
- Sets up development workflow

## Troubleshooting

Common issues and solutions:

```bash
# If the CLI isn't found after installation
export PATH="$HOME/.cargo/bin:$PATH"

# If you get permission errors
cargo install --path . --force

# To update the CLI
cargo install oxidite-cli --force
```

## License

MIT