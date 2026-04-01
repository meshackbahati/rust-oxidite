# CLI Tool Guide

The CLI package is `oxidite-cli`. The installed binary is `oxidite`.

## Installation

```bash
cargo install oxidite-cli

# Install this generated CLI build explicitly
cargo install oxidite-cli --version 2.1.0-gen
```

## Project Creation

```bash
oxidite new myapp
oxidite new myapp --type api
oxidite new myapp --template web
```

Generated projects include:

- `oxidite.toml`
- `migrations/`
- `seeds/`
- `src/controllers/`
- `src/events/`
- `src/jobs/`
- `src/middleware/`
- `src/models/`
- `src/policies/`
- `src/routes/`
- `src/services/`
- `src/validators/`

## Generators

Use `generate` for new workflows. `make` remains as a hidden compatibility alias.

```bash
oxidite generate model User
oxidite generate model User email:string age:integer
oxidite generate route users
oxidite generate controller UserController
oxidite generate middleware AuthMiddleware
oxidite generate service Billing
oxidite generate validator CreateUser
oxidite generate job SendDigest
oxidite generate policy Post
oxidite generate event UserSignedUp
oxidite generate migration create_users_table
oxidite generate seeder users_seed
```

## Migrations

```bash
oxidite migrate create create_users_table
oxidite migrate
oxidite migrate run
oxidite migrate status
oxidite migrate revert
oxidite migrate:rollback
```

Migration files use the SQL sections the runtime understands:

```sql
-- migrate:up
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    email TEXT NOT NULL
);

-- migrate:down
DROP TABLE users;
```

## Seeders

```bash
oxidite seed create users_seed
oxidite seed
oxidite seed run
oxidite db:seed
```

## Queue Commands

Canonical commands:

```bash
oxidite queue work --workers 4
oxidite queue list
oxidite queue dlq
oxidite queue clear
```

Compatibility aliases:

```bash
oxidite queue:work --workers 4
oxidite queue:list
oxidite queue:dlq
oxidite queue:clear
```

## Development

```bash
oxidite dev
oxidite dev --port 8080
oxidite dev --host 0.0.0.0 --env development
oxidite dev --watch src --watch templates
oxidite dev --ignore dist
oxidite dev --no-hot-reload
```

The CLI forwards host, port, and environment overrides through:

- `SERVER_HOST`
- `SERVER_PORT`
- `OXIDITE_ENV`

## Build And Serve

```bash
oxidite build
oxidite build --release
oxidite build --profile release
oxidite build --target x86_64-unknown-linux-musl
oxidite build --features "database,queue"
oxidite build --verbose

oxidite serve
oxidite serve --addr 0.0.0.0:8080
oxidite serve --env production
```

## Diagnostics

```bash
oxidite doctor
oxidite --help
oxidite --version
oxidite version
```
