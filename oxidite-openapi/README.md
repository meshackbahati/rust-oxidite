# oxidite-openapi

OpenAPI 3.0 document structures and helpers for Oxidite.

## Installation

```toml
[dependencies]
oxidite-openapi = "2.1.0"
```

## What This Crate Provides

- OpenAPI spec data structures (`OpenApiSpec`, `PathItem`, `Operation`, etc.)
- `OpenApiBuilder` for assembling a spec
- Helper constructors for common operations and schemas
- Swagger UI HTML generation via `generate_docs_html`

## Quick Example

```rust
use oxidite_openapi::{
    get_operation, generate_docs_html, AutoDocs, OpenApiBuilder, Parameter, PathItem, Response, Schema,
};

let list_users = get_operation("List users")
    .with_description("Returns paginated users")
    .add_parameter(Parameter::query("page", Schema::integer()))
    .add_response("200", Response::json("ok", Schema::array(Schema::object(std::collections::HashMap::new()))));

let spec = OpenApiBuilder::new("Users API", "1.0.0")
    .description("Public API")
    .path("/users", PathItem::default().with_get(list_users))
    .build();

let html = generate_docs_html(&spec);
assert!(html.contains("SwaggerUIBundle"));
```

## Router Integration

If you use `oxidite_core::Router`, you can register docs endpoints in one call:

```rust
# use oxidite_core::Router;
# use oxidite_openapi::{AutoDocs, OpenApiBuilder};
let router = Router::new();
let spec = OpenApiBuilder::new("My API", "1.0.0").build();
let _router = router.with_auto_docs(spec); // mounts /openapi.json and /api/docs
```

## Notes

- This crate does not currently include proc-macro annotations or automatic route introspection.
- Integrate by exposing the generated spec at an endpoint (for example `/openapi.json`) and serving `generate_docs_html` output.
