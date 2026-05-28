# CLI Tools

The CLI package name is `oxidite-cli`. The installed executable is `oxidite`.

## Installation

```bash
# Install from crates.io
cargo install oxidite-cli

# Install from this repository
cargo install --path oxidite-cli
```

After installation, the first time you run `oxidite`, it will suggest adding an alias for shorter commands.

**Recommended**: Add this to your shell config:

```bash
# For bash
echo "alias oxi='oxidite'" >> ~/.bashrc
source ~/.bashrc

# For zsh
echo "alias oxi='oxidite'" >> ~/.zshrc
source ~/.zshrc
```

**Windows users:**

```powershell
# PowerShell - add to your $PROFILE
Set-Alias oxi oxidite

# CMD (temporary, per session)
doskey oxi=oxidite
```

After this, you can use `oxi` instead of `oxidite` for all commands.

## What the CLI Actually Does

The CLI is a **code generation and project management tool**. It:

- Creates project scaffolding with standard structure
- Generates boilerplate code (models, routes, controllers, migrations)
- Runs and manages database migrations
- Starts dev servers with file watching
- Manages background processes

It does **not**:

- Compile your code faster than `cargo build`
- Prevent you from writing bad code
- Replace understanding of Rust, cargo, or the underlying libraries

## Project Creation

```bash
oxi new my-project
oxi new my-api --project-type api
oxi new my-web --template web
```

Generated projects include a standard directory structure. You can reorganize it, but the CLI tools expect this layout.

```text
my-project/
├── Cargo.toml
├── oxidite.toml
├── migrations/
├── src/
│   ├── main.rs
│   ├── models/
│   ├── routes/
│   └── controllers/
└── tests/
```

## Code Generators

Generators create files with boilerplate code. They won't overwrite existing files without asking.

```bash
oxi generate model User
oxi generate model User email:string age:integer
oxi generate route users
oxi generate controller UserController
oxi generate middleware AuthMiddleware
```

**Supported generators:**

| Generator | Creates | Notes |
|-----------|---------|-------|
| `model` | Struct with `#[derive(Model)]` | Generates CRUD methods, queries, validation |
| `route` | Route module | Basic router with handlers |
| `controller` | Controller struct | REST-style endpoint handlers |
| `middleware` | Middleware impl | Tower-style middleware |
| `service` | Service struct | Business logic layer |
| `validator` | Validator struct | Request validation |
| `job` | Background job | Queue job handler |
| `policy` | Authorization policy | RBAC/PBAC rules |
| `event` | Domain event | Event struct + handlers |
| `migration` | SQL migration file | Template with up/down sections |
| `seeder` | Database seeder | Seed data script |

**Model field types:**

`string`, `text`, `integer`, `float`, `decimal`, `boolean`, `uuid`, `json`, `timestamp`

The generated code is yours to modify. The generators are one-time scaffolding tools.

## Migrations

```bash
# Create and run
oxi migrate create create_users_table
oxi migrate run

# Check status and rollback
oxi migrate status
oxi migrate revert

# Auto-generate from model changes
oxi make-migrations
oxi make-migrations add_email_field --dry-run
```

Migration files are plain SQL:

```sql
-- migrate:up
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    email TEXT NOT NULL
);

-- migrate:down
DROP TABLE users;
```

**Limitations:**
- Migrations are tracked by filename, not content hash
- No auto-detection of destructive changes (dropping columns)
- Rollback only reverts the last migration, not arbitrary ones
- `make-migrations` generates SQL based on model differences but may miss complex schema changes

## Database Seeders

```bash
oxi seed create users_seed
oxi seed run
```

Seeders run your code against the database. You write the insertion logic yourself.

## Queue Management

```bash
oxi queue work --workers 4
oxi queue list
oxi queue dlq
oxi queue clear
```

Queues require Redis for production. The in-memory backend is for development only and loses data on restart.

## Development Server

```bash
oxi dev
oxi dev --port 8080
oxi dev --watch src --watch templates
oxi dev --no-hot-reload
```

The dev server:
- Watches files and recompiles on changes
- Uses `cargo run` under the hood (not faster than cargo)
- Passes environment variables from `.env` and `oxidite.toml`
- Can be slow on large projects (full recompile each change)

## Run Single Files

Execute a Rust file directly without creating a full project:

```bash
# Standalone file (creates temp project, runs, cleans up)
oxi run hello.rs

# File inside project (copies to src/bin/)
oxi run src/bin/script.rs

# With extra dependencies
oxi run api.rs --deps serde,chrono
```

**How it works:**
- **Standalone mode**: Creates a temporary Cargo project in `/tmp`, compiles with oxidite dependencies, runs, then deletes the temp directory
- **Project mode**: Places the file in `src/bin/` and runs via `cargo run --bin`
- Compile errors display directly in your terminal
- Not meant for production - use `oxi build` for deployable binaries

## Process Management

Manage long-running oxidite processes:

```bash
oxi pm2 start
oxi pm2 start my-api --release
oxi pm2 stop my-api
oxi pm2 restart my-api
oxi pm2 list
oxi pm2 info my-api
oxi pm2 monitor
```

Process state is stored in `.oxidite_procs.json` in the current directory. This is **not** a replacement for systemd, Docker, or proper process managers in production.

## Debug Logging

Enable verbose output:

```bash
OXIDITE_DEBUG=1 oxi dev
OXIDITE_DEBUG=true oxi serve
```

Shows environment loading, configuration details, file paths, and internal operations.

## Colored Output

Errors display in red, success in green, warnings in yellow, info in blue. Error messages are categorized by type (compile errors, runtime errors, permission errors).

## Build and Deploy

```bash
oxi build --release
oxi serve --env production
```

`oxi build` runs `cargo build` with your specified flags. It doesn't optimize beyond what cargo already does.

## Diagnostics

```bash
oxi doctor
```

Checks for:
- Rust and Cargo installation
- Required project files (`Cargo.toml`, `oxidite.toml`)
- Migration directory
- Common environment variables

It reports what it finds but doesn't fix issues automatically.
