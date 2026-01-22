# Installation

This chapter covers how to install Oxidite and set up your development environment.

## Prerequisites

Before installing Oxidite, you'll need:

- Rust 1.75 or higher
- Cargo (comes with Rust)
- Git

You can install Rust using rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Installing the Oxidite CLI

The easiest way to get started is to install the Oxidite CLI tool:

```bash
# Install from source (recommended for development)
cargo install --path oxidite-cli

# Or install from crates.io when published
cargo install oxidite-cli
```

## Creating Your First Project

Once you have the CLI installed, create a new project:

```bash
oxidite new my-app
cd my-app
```

This will create a new Oxidite project with a basic structure and all necessary dependencies.

## Manual Installation

If you prefer to add Oxidite to an existing project manually, add it to your `Cargo.toml`:

```toml
[dependencies]
oxidite = { version = "2.0", features = ["full"] }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
```

## Development Dependencies

For testing and development, you may also want to add:

```toml
[dev-dependencies]
oxidite-testing = "2.0"
tokio-test = "0.4"
```

## Verifying Installation

To verify your installation, create a simple test file:

```rust
use oxidite::prelude::*;

async fn hello(_req: Request) -> Result<Response> {
    Ok(Response::text("Hello, Oxidite!"))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    router.get("/", hello);

    Server::new(router)
        .listen("127.0.0.1:3000".parse()?)
        .await
}
```

Run this with:

```bash
cargo run
```

You should see your server running on http://127.0.0.1:3000.

## Troubleshooting

If you encounter issues:

1. Ensure you have the latest version of Rust installed
2. Make sure your Cargo is up to date
3. Check that you have all required build tools for your platform
4. Verify that you're using the correct features for your use case

Common features include:
- `full`: All features enabled
- `database`: Database ORM capabilities
- `auth`: Authentication and authorization
- `queue`: Background job processing
- `cache`: Caching capabilities
- `realtime`: WebSocket and SSE support
- `templates`: Server-side template rendering
- `mail`: Email sending capabilities
- `storage`: File storage (local/S3)
- `graphql`: GraphQL support
- `plugin`: Plugin system support