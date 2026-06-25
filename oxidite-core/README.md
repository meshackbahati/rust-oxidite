# oxidite-core

Core HTTP server, router, request/response types, and extractors for Oxidite.

## Installation

```toml
[dependencies]
oxidite-core = "2.3.4"
```

## Key Components

- `Router`: method/path routing with path params and wildcard support.
- `Server`: Hyper-based async server integration for Oxidite services.
- `OxiditeRequest` / `OxiditeResponse`: request/response core types.
- Extractors: `Path`, `Query`, `Json`, `Form`, `State`, `Cookies`, `Body`, `WebSocketUpgrade`.

## Basic Example

```rust
use oxidite_core::{Application, Request, Response, Result};

async fn hello(_req: Request) -> Result<Response> {
    Ok(Response::text("Hello, Oxidite!"))
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = oxidite_config::Config::default();
    let mut app = Application::new(config);
    app.router_mut().get("/", hello);
    app.run().await
}
```

> **Alternative**: The manual approach using `Router::new()` + `Server::new(router).listen(addr)` works identically under the hood.

## Notes

- `HEAD` requests automatically fall back to matching `GET` routes.
- If a path exists for another method, the router returns `MethodNotAllowed`.
