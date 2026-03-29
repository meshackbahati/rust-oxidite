# Oxidite Testing

Testing utilities for Oxidite handlers and routers.

## Installation

```toml
[dev-dependencies]
oxidite-testing = "2.1.0"
```

## Available API

- `TestRequest`: fluent request builder for GET/POST/PUT/DELETE.
- `TestRequest::build_oxidite()`: convert test request to `OxiditeRequest`.
- `TestResponse`: response wrapper helpers (`status`, `text`, `json`, assertions).
- `TestServer`: wraps a `tower::Service` (including `Router`) for request execution.
- `test_router(router)`: convenience constructor for `TestServer<Router>`.

## Example

```rust
use oxidite::prelude::*;
use oxidite_testing::{test_router, TestRequest};

async fn ping(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    Ok(OxiditeResponse::text("pong"))
}

#[tokio::test]
async fn ping_route_responds() {
    let mut router = Router::new();
    router.get("/ping", ping);

    let mut server = test_router(router);
    let req = TestRequest::get("/ping").build_oxidite();

    let resp = server.call(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
}
```

## Notes

- `TestRequest::header/json/build` are convenience methods and may panic on invalid input.
- Use `try_header`, `try_json`, and `try_build` for non-panicking test setup.
