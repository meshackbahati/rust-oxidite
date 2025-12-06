# Getting Started with Oxidite

Oxidite is a modern, high-performance web framework for Rust. This guide will help you set up your environment and create your first Oxidite application.

## Prerequisites

- **Rust**: You need Rust installed. If you haven't, install it via [rustup](https://rustup.rs/):
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

## Installation

Oxidite provides a CLI tool to scaffold projects. Install it from the source:

```bash
cargo install --path oxidite-cli
```

## Creating a New Project

The `oxidite new` command is interactive and helps you choose the right template for your needs.

```bash
oxidite new my-app
```

You will be prompted to select a project type:

1. **Fullstack Application**: Includes templates, static files, and a complete folder structure.
2. **REST API**: Optimized for backend-only services with JSON responses.
3. **Microservice**: Minimal setup for specialized services.
4. **Serverless Function**: Lightweight handler for event-driven architectures.

### Non-Interactive Mode

For CI/CD or scripts, you can pass the type directly:

```bash
oxidite new my-api --project-type api
```

## Project Structure

A standard Oxidite project (API/Fullstack) looks like this:

```
my-app/
├── src/
│   ├── controllers/   # Request handlers
│   ├── models/        # Database models
│   ├── routes/        # Route definitions
│   ├── services/      # Business logic
│   ├── middleware/    # Custom middleware
│   ├── utils/         # Helper functions
│   └── main.rs        # Entry point
├── config/            # Configuration files
├── tests/             # Integration tests
├── Cargo.toml         # Dependencies
└── config.toml        # App configuration
```

## Running the Server

Oxidite comes with a built-in development server that supports hot reloading.

```bash
cd my-app
oxidite dev
```

The server will start on `http://127.0.0.1:8080` (or the port defined in `config.toml`).
Any changes to your code will automatically restart the server.

## Next Steps

- [Database Guide](database.md) - Learn how to use the ORM.
- [CLI Guide](cli.md) - Explore all CLI commands.
