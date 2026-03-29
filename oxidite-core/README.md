# oxidite-core

Core HTTP server, router, request/response types, and extractors for Oxidite.

## Installation

```toml
[dependencies]
oxidite-core = "2.1.0"
```

## Key Components

- `Router`: method/path routing with path params and wildcard support.
- `Server`: Hyper-based async server integration for Oxidite services.
- `OxiditeRequest` / `OxiditeResponse`: request/response core types.
- Extractors: `Path`, `Query`, `Json`, `Form`, `State`, `Cookies`, `Body`.

## Basic Example

```rust
use oxidite_core::{Router, Server, OxiditeResponse, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();

    router.get("/", || async {
        Ok(OxiditeResponse::text("Hello, Oxidite!"))
    });

    Server::new(router)
        .listen("127.0.0.1:3000".parse().unwrap())
        .await
}
```

## Notes

- `HEAD` requests automatically fall back to matching `GET` routes.
- If a path exists for another method, the router returns `MethodNotAllowed`.
