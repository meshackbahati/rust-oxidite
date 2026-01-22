# CLI Tools

Oxidite provides a comprehensive command-line interface (CLI) for project management and development tasks.

## Installation

Install the Oxidite CLI globally:

```bash
cargo install --path oxidite-cli
```

Or build and install from the repository:

```bash
cd oxidite-cli
cargo install --path .
```

## Commands

### `oxidite new`

Create a new Oxidite project:

```bash
# Create a new project with default settings
oxidite new my-project

# Create a project with specific type (api, fullstack, microservice)
oxidite new my-api --project-type api
oxidite new my-fullstack --project-type fullstack
oxidite new my-microservice --project-type microservice
```

The `new` command creates a project with the following structure:

```
my-project/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── routes/
│   ├── controllers/
│   └── models/
├── templates/          # For fullstack projects
├── public/             # For static files
├── migrations/         # For database migrations
└── README.md
```

### `oxidite serve`

Start the HTTP server:

```bash
# Serve on default address (127.0.0.1:3000)
oxidite serve

# Serve on custom address
oxidite serve --addr 0.0.0.0:8080
```

### `oxidite dev`

Start the development server with hot reloading:

```bash
# Start development server
oxidite dev

# This watches for file changes and automatically restarts the server
```

### `oxidite make`

Generate code templates for common components:

#### `oxidite make model`

Create a new model:

```bash
# Generate a User model
oxidite make model User

# This creates a file src/models/user.rs with a basic model structure
```

The generated model file will look like:

```rust
use oxidite::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Model, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub created_at: i64,
    pub updated_at: i64,
}
```

#### `oxidite make controller`

Create a new controller:

```bash
# Generate a UserController
oxidite make controller UserController

# This creates a file src/controllers/user_controller.rs
```

The generated controller file will include basic CRUD operations:

```rust
use oxidite::prelude::*;
use serde_json::json;

pub async fn index(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    Ok(response::json(json!({"message": "List all resources"})))
}

pub async fn show(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    Ok(response::json(json!({"message": "Show a specific resource"})))
}

pub async fn create(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    Ok(response::json(json!({"message": "Create a new resource"})))
}

pub async fn update(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    Ok(response::json(json!({"message": "Update an existing resource"})))
}

pub async fn delete(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    Ok(response::json(json!({"message": "Delete a resource"})))
}
```

#### `oxidite make middleware`

Create new middleware:

```bash
# Generate authentication middleware
oxidite make middleware AuthMiddleware
```

### `oxidite migrate`

Manage database migrations:

#### `oxidite migrate create`

Create a new migration file:

```bash
# Create a migration for adding users table
oxidite migrate create create_users_table

# This creates a file in migrations/ directory with timestamp prefix
# e.g., migrations/20241206000001_create_users_table.sql
```

The generated migration file will look like:

```sql
-- Migration: create_users_table
-- Created at: 2024-12-06 00:00:01

-- Up
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Down (optional - for rollback)
-- DROP TABLE users;
```

#### `oxidite migrate run`

Run pending migrations:

```bash
# Run all pending migrations
oxidite migrate run
```

#### `oxidite migrate revert`

Rollback the last migration:

```bash
# Revert the last migration
oxidite migrate revert
```

#### `oxidite migrate status`

Check migration status:

```bash
# Show current migration status
oxidite migrate status
```

### `oxidite seed`

Manage database seeders:

#### `oxidite seed run`

Run database seeders:

```bash
# Run all seeders
oxidite seed run
```

#### `oxidite seed create`

Create a new seeder:

```bash
# Create a seeder for initial users
oxidite seed create CreateInitialUsers
```

### `oxidite queue`

Manage background job queues:

#### `oxidite queue work`

Start queue workers:

```bash
# Start 4 workers
oxidite queue work --workers 4

# Or with short flag
oxidite queue work -w 4
```

#### `oxidite queue list`

List queue statistics:

```bash
# Show queue statistics
oxidite queue list
```

#### `oxidite queue dlq`

List dead letter queue:

```bash
# Show failed jobs in dead letter queue
oxidite queue dlq
```

#### `oxidite queue clear`

Clear all pending jobs:

```bash
# Clear all jobs from the queue
oxidite queue clear
```

### `oxidite doctor`

Run system diagnostics:

```bash
# Check if all dependencies and configurations are correct
oxidite doctor
```

This command verifies:
- Rust installation
- Cargo availability
- Required dependencies
- Database connectivity (if configured)
- Environment variables

### `oxidite build`

Build the project:

```bash
# Build in debug mode
oxidite build

# Build in release mode
oxidite build --release
```

## Project Structure

When you create a new project with `oxidite new`, you get the following structure:

```
my-project/
├── Cargo.toml              # Project dependencies and metadata
├── .env                   # Environment variables (not committed)
├── .gitignore             # Git ignore file
├── README.md              # Project documentation
├── src/
│   ├── main.rs           # Entry point
│   ├── routes/
│   │   ├── mod.rs        # Route registration
│   │   └── api.rs        # API routes
│   ├── controllers/      # Request handlers
│   │   └── mod.rs
│   ├── models/           # Data models
│   │   └── mod.rs
│   ├── middleware/       # Custom middleware
│   │   └── mod.rs
│   └── services/         # Business logic
│       └── mod.rs
├── migrations/           # Database migration files
├── seeds/               # Database seed files
├── templates/           # HTML templates (for fullstack)
├── public/              # Static assets
│   ├── css/
│   ├── js/
│   └── images/
└── tests/               # Test files
```

## Configuration

The CLI reads configuration from:

1. Command line arguments
2. Environment variables
3. `.env` file in the project root
4. `Cargo.toml` in the project root

## Development Workflow

A typical development workflow with the Oxidite CLI:

```bash
# 1. Create a new project
oxidite new my-awesome-app

# 2. Navigate to project
cd my-awesome-app

# 3. Generate models and controllers
oxidite make model User
oxidite make controller UserController

# 4. Create database migrations
oxidite migrate create create_users_table

# 5. Run migrations
oxidite migrate run

# 6. Start development server
oxidite dev

# 7. In another terminal, create more components as needed
oxidite make middleware AuthMiddleware
```

## Environment Variables

The CLI supports common environment variables:

- `DATABASE_URL`: Database connection string
- `PORT`: Port to serve on (defaults to 3000)
- `HOST`: Host to bind to (defaults to 127.0.0.1)
- `RUST_LOG`: Logging level (e.g., `info`, `debug`, `warn`)

## Troubleshooting

### Common Issues

**Command not found**: If you get "command not found" error, make sure the cargo bin directory is in your PATH:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

**Permission denied**: If you get permission errors, try installing with `--root`:

```bash
cargo install --path oxidite-cli --root ~/.cargo
```

**Build failures**: If the CLI doesn't build, ensure you have the latest Rust version:

```bash
rustup update
```

### Debugging

For verbose output, use the `RUST_LOG` environment variable:

```bash
RUST_LOG=debug oxidite serve
```

## Customization

You can customize the CLI behavior by creating a `.oxiditerc` file in your project root:

```json
{
  "port": 8080,
  "host": "0.0.0.0",
  "database_url": "postgresql://localhost/myapp",
  "features": ["database", "auth", "templates"]
}
```

This file allows you to set default values for common options across all CLI commands in your project.