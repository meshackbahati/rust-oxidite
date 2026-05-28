# CLI Tools

The Oxidite CLI package is `oxidite-cli`, and the installed executable is `oxidite`.

## Installation

```bash
cargo install oxidite-cli
```

**After installation**, add this alias for shorter commands:

```bash
# Bash
echo "alias oxi='oxidite'" >> ~/.bashrc
source ~/.bashrc

# Zsh
echo "alias oxi='oxidite'" >> ~/.zshrc
source ~/.zshrc
```

**Windows PowerShell:**
```powershell
Set-Alias oxi oxidite  # Add to $PROFILE for persistence
```

The CLI will also suggest this alias on first run.

## What the CLI Does

The CLI generates code, manages projects, and runs development tools. It saves typing but doesn't replace understanding of the underlying technologies.

**It does:**
- Create project scaffolding
- Generate boilerplate models, routes, controllers
- Run database migrations
- Start development servers with file watching
- Execute single Rust files without full project setup
- Manage background processes

**It doesn't:**
- Speed up compilation beyond cargo's capabilities
- Prevent architectural mistakes
- Replace knowledge of Rust, sqlx, or hyper

## Project Scaffolding

```bash
oxi new my_app
oxi new my_api --project-type api
oxi new my_web --template web
```

Generated structure includes standard directories. The CLI tools expect this layout for commands like `oxi generate` and `oxi migrate`.

## Code Generation

```bash
oxi generate model User email:string age:integer
oxi generate route users
oxi generate controller UserController
```

Generated code is starting point boilerplate. You modify it to fit your needs. Generators don't overwrite existing files.

## Database Migrations

```bash
oxi migrate create create_users_table
oxi migrate run
oxi migrate status
oxi migrate revert
oxi make-migrations  # Auto-generate from model changes
```

Migrations are SQL files with `-- migrate:up` and `-- migrate:down` sections. The migration tracker stores which migrations have run, not their content hashes.

## Development Server

```bash
oxi dev
oxi dev --port 8080 --watch src
```

Runs `cargo run` and restarts on file changes. Compile times remain the same as manual `cargo build`.

## Single File Execution

```bash
oxi run script.rs
oxi run script.rs --deps serde,chrono
```

Creates temp projects for standalone files. Useful for scripts and prototypes, not for production code.

## Process Management

```bash
oxi pm2 start --release
oxi pm2 list
oxi pm2 stop <name>
```

Stores process state locally. For production deployment, use systemd, Docker, or Kubernetes instead.

## Debug and Output

```bash
OXIDITE_DEBUG=1 oxi dev
```

Colored output: red for errors, green for success, yellow for warnings, blue for info. Errors are categorized by type automatically.
