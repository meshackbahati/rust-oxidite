# CLI Tools

The Oxidite CLI package is `oxidite-cli`, and the installed executable is `oxidite`.

## Installation

```bash
# Install from crates.io
cargo install oxidite-cli

# Install this generated build explicitly
cargo install oxidite-cli --version 2.1.0-gen

# Install from the workspace checkout
cargo install --path oxidite-cli
```

Verify the binary:

```bash
oxidite --version
oxidite version
```

## Project Scaffolding

Create a new project:

```bash
# Interactive project creation
oxidite new my_app

# Explicit project type
oxidite new my_api --project-type api
oxidite new my_api --type api

# Template aliases
oxidite new my_web --template web
oxidite new my_fullstack --template fullstack
oxidite new my_minimal --template minimal
```

Supported project kinds:

- `api`
- `fullstack`
- `web` as an alias for `fullstack`
- `microservice`
- `minimal` as an alias for `api`
- `serverless`

The generated project includes the directories the CLI expects for development:

```text
my_app/
├── Cargo.toml
├── README.md
├── oxidite.toml
├── migrations/
├── seeds/
├── src/
│   ├── main.rs
│   ├── controllers/
│   ├── events/
│   ├── jobs/
│   ├── middleware/
│   ├── models/
│   ├── policies/
│   ├── routes/
│   ├── services/
│   └── validators/
└── tests/
```

## Code Generation

Use `generate` for new workflows. `make` remains as a hidden compatibility alias.

```bash
# Models
oxidite generate model User
oxidite generate model User email:string age:integer

# Route modules
oxidite generate route users

# Controllers and middleware
oxidite generate controller UserController
oxidite generate middleware AuthMiddleware

# Other supported generators
oxidite generate service Billing
oxidite generate validator CreateUser
oxidite generate job SendDigest
oxidite generate policy Post
oxidite generate event UserSignedUp

# File-based database artifacts
oxidite generate migration create_users_table
oxidite generate seeder users_seed
```

Supported model field types:

- `string`
- `text`
- `integer`
- `float`
- `decimal`
- `boolean`
- `uuid`
- `json`
- `timestamp`

Example generated model:

```rust,ignore
use serde::{Deserialize, Serialize};
use oxidite::db::{Model, sqlx};

#[derive(Debug, Clone, Serialize, Deserialize, Model, sqlx::FromRow)]
#[model(table = "users")]
pub struct User {
    pub id: i64,
    pub email: String,
    pub age: i64,
}
```

## Database Migrations

Create a migration file:

```bash
oxidite migrate create create_users_table
oxidite generate migration create_users_table
```

The generated file uses file-based SQL sections:

```sql
-- migrate:up
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    email TEXT NOT NULL
);

-- migrate:down
DROP TABLE users;
```

Run migrations:

```bash
# Canonical command
oxidite migrate run

# Bare command also runs pending migrations
oxidite migrate
```

Check or revert migrations:

```bash
oxidite migrate status
oxidite migrate revert

# Compatibility alias retained by the CLI
oxidite migrate:rollback
```

## Seeders

```bash
# Create a seeder file
oxidite seed create users_seed
oxidite generate seeder users_seed

# Run seeders
oxidite seed run
oxidite seed

# Compatibility alias
oxidite db:seed
```

## Queue Commands

Canonical queue commands:

```bash
oxidite queue work --workers 4
oxidite queue list
oxidite queue dlq
oxidite queue clear
```

Compatibility aliases that still work:

```bash
oxidite queue:work --workers 4
oxidite queue:list
oxidite queue:dlq
oxidite queue:clear
```

## Development Workflow

Start the development server with hot reload:

```bash
oxidite dev
oxidite dev --port 8080
oxidite dev --host 0.0.0.0 --env development
oxidite dev --watch src --watch templates
oxidite dev --ignore dist
oxidite dev --no-hot-reload
```

The CLI forwards these overrides to the generated app via:

- `SERVER_HOST`
- `SERVER_PORT`
- `OXIDITE_ENV`

Start the current project in release mode:

```bash
oxidite serve
oxidite serve --addr 0.0.0.0:8080
oxidite serve --env production
```

Build the current project:

```bash
oxidite build
oxidite build --release
oxidite build --profile release
oxidite build --target x86_64-unknown-linux-musl
oxidite build --features "database,queue"
oxidite build --verbose
```

## Diagnostics

```bash
oxidite doctor
```

The doctor command checks:

- Rust and Cargo availability
- project files
- migration directory presence
- common environment variables

## Help

```bash
oxidite --help
oxidite migrate --help
oxidite generate --help
```
