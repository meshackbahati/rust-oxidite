# Oxidite CLI Reference

The `oxidite` command-line tool is your companion for building Oxidite applications.

## Installation

```bash
cargo install --path oxidite-cli
```

## Commands

### `new` - Create a Project

Scaffolds a new Oxidite project.

```bash
oxidite new <project-name> [options]
```

**Options:**
- `-t, --type <type>`: Specify project type (`fullstack`, `api`, `microservice`, `serverless`). If omitted, an interactive prompt is shown.

**Example:**
```bash
oxidite new my-blog --type fullstack
```

### `dev` - Development Server

Starts the application in development mode with hot reloading.

```bash
oxidite dev
```

This command watches for changes in `.rs`, `.toml`, `.html`, `.css`, and `.js` files and automatically restarts the server.

### `make` - Code Generation

Generates boilerplate code for various components.

#### `make model`
Creates a new database model in `src/models/`.

```bash
oxidite make model User
```

#### `make controller`
Creates a new controller with CRUD handlers in `src/controllers/`.

```bash
oxidite make controller Users
```

#### `make middleware`
Creates a new middleware in `src/middleware/`.

```bash
oxidite make middleware Auth
```

### `migrate` - Database Migrations

Manages database schema changes.

#### `migrate create`
Creates a new migration file.

```bash
oxidite migrate create create_users_table
```

#### `migrate run`
Applies pending migrations.

```bash
oxidite migrate run
```

#### `migrate revert`
Reverts the last applied migration.

```bash
oxidite migrate revert
```

#### `migrate status`
Shows the status of all migrations.

```bash
oxidite migrate status
```
