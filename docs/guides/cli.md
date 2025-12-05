# Oxidite CLI Guide

The Oxidite CLI (`oxidite`) is your primary tool for developing applications with the Oxidite framework. It helps you scaffold projects, generate code, manage database migrations, and run your development server.

## Installation

To install the CLI from the source:

```bash
cargo install --path oxidite-cli
```

## Commands

### `new`

Creates a new Oxidite project with a standard directory structure and configuration.

**Usage:**
```bash
oxidite new <project_name>
```

**Example:**
```bash
oxidite new my-awesome-app
```

This will create a directory `my-awesome-app` containing:
- `Cargo.toml`: Project dependencies
- `src/main.rs`: Entry point
- `oxidite.toml`: Configuration file
- `.env`: Environment variables

### `serve`

Starts the HTTP server.

**Usage:**
```bash
oxidite serve [OPTIONS]
```

**Options:**
- `--addr <ADDR>`: Address to bind to (default: `127.0.0.1:3000`)

**Example:**
```bash
oxidite serve --addr 0.0.0.0:8080
```

### `make`

Generates boilerplate code for various components.

**Usage:**
```bash
oxidite make <GENERATOR> <NAME>
```

#### Generators:

- **Model**: Generates a database model struct.
  ```bash
  oxidite make model User
  ```

- **Controller**: Generates a controller with handler functions.
  ```bash
  oxidite make controller AuthController
  ```

- **Middleware**: Generates a middleware struct.
  ```bash
  oxidite make middleware RateLimit
  ```

### `migrate`

Manages database migrations.

**Usage:**
```bash
oxidite migrate <COMMAND>
```

#### Commands:

- **create**: Creates a new migration file.
  ```bash
  oxidite migrate create add_users_table
  ```

- **run**: Runs all pending migrations.
  ```bash
  oxidite migrate run
  ```

- **revert**: Reverts the last applied migration.
  ```bash
  oxidite migrate revert
  ```

- **status**: Shows the status of migrations.
  ```bash
  oxidite migrate status
  ```

## Workflow Example

1.  **Create a project:**
    ```bash
    oxidite new blog
    cd blog
    ```

2.  **Generate a model:**
    ```bash
    oxidite make model Post
    ```

3.  **Create a migration:**
    ```bash
    oxidite migrate create create_posts_table
    ```

4.  **Run the server:**
    ```bash
    oxidite serve
    ```
