# CLI Tools

The CLI package name is `oxidite-cli`. The installed executable is `oxidite`.

## Installation

```bash
# Install from crates.io
cargo install oxidite-cli

# Install this generated CLI build explicitly
cargo install oxidite-cli --version 2.1.0-gen

# Install from this repository
cargo install --path oxidite-cli
```

## Project Creation

```bash
oxidite new my-project
oxidite new my-api --project-type api
oxidite new my-api --type api
oxidite new my-web --template web
oxidite new my-minimal --template minimal
```

Generated projects include:

```text
my-project/
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

## Migrations

```bash
oxidite migrate create create_users_table
oxidite generate migration create_users_table

# Run pending migrations
oxidite migrate
oxidite migrate run

# Inspect or revert
oxidite migrate status
oxidite migrate revert
oxidite migrate:rollback
```

Migration files use the following format:

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
oxidite generate seeder users_seed
oxidite seed
oxidite seed run
oxidite db:seed
```

## Queue Management

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

The CLI passes host, port, and environment overrides to the app through:

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
oxidite migrate --help
oxidite --version
oxidite version
```
