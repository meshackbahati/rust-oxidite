# CLI Tools

The Oxidite CLI package is `oxidite-cli`, and the installed executable is `oxidite`.

## Installation

```bash
# Install from crates.io
cargo install oxidite-cli

# Install this generated build explicitly
cargo install oxidite-cli --version 2.2.0

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
    pub created_at: i64,
    pub updated_at: i64,
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

### Declarative Migrations (Auto-Generation)

Oxidite can automatically generate migration files by diffing your Rust models against the current database schema. This eliminates the need for manual SQL for common schema changes.

```bash
# Generate a migration based on model changes
oxidite migrate make

# Generate with a specific name
oxidite migrate make create_users_table

# Dry run: see the SQL without generating files
oxidite migrate make --dry-run
```

You can also use the top-level alias:
```bash
oxidite make-migrations
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

## Interactive Console (Tinker)

Oxidite ships with a built-in interactive console, similar to Laravel's Tinker or Rails' console. It lets you evaluate Rust expressions directly inside your project's context — with access to all your models, services, and the database.

```bash
oxidite tinker
```

**Example session:**

```
🧪 Oxidite Tinker v2.2.0
Type Rust expressions to evaluate them in your project's context.
Use `exit` or Ctrl-D to quit.

oxidite> User::all(&db).await
[User { id: 1, name: "Alice", ... }, User { id: 2, name: "Bob", ... }]

oxidite> User::find(&db, 1).await
Some(User { id: 1, name: "Alice", email: "alice@example.com" })

oxidite> 2 + 2
4

oxidite> exit
👋 Goodbye!
```

Under the hood, Tinker generates a temporary `src/bin/_tinker.rs` file, compiles it with `cargo run`, and displays the result. The temporary file is cleaned up automatically when you exit.

## Performance Profiler

```bash
# Profile a local HTTP endpoint with default concurrency (10) and total requests (100)
oxidite profile http://localhost:3000/

# Profile with custom concurrency and request count
oxidite profile http://localhost:3000/users -c 20 -r 500
```

The `profile` command spawns concurrent asynchronous HTTP workers using `reqwest` to measure:
- Total execution time
- Successful vs. failed requests (HTTP 2xx)
- Request throughput (Requests per second / RPS)
- Detailed latency percentiles (min, max, average, p50, p90, p99)

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
