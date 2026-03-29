# oxidite-macros

Procedural macros used by Oxidite crates.

This crate currently provides:

- `#[derive(Model)]` for `oxidite-db` models.

## `derive(Model)`

```rust
use oxidite_db::Model;
use sqlx::FromRow;

#[derive(Model, FromRow)]
#[model(table = "users")]
struct User {
    id: i64,
    name: String,
    #[validate(email)]
    email: String,
    created_at: i64,
    updated_at: i64,
    deleted_at: Option<i64>,
}
```

Supported container attributes:

- `#[model(table_name = "...")]`
- `#[model(table = "...")]` (alias)

Supported field attributes:

- `#[validate(email)]` for `String` fields.

Generated behavior:

- Implements `oxidite_db::Model`.
- Generates CRUD SQL for the model table.
- Enables soft delete when `deleted_at: Option<i64>` exists.
- Maintains `created_at`/`updated_at` when those fields are `i64`.
- Implements `is_persisted()` as `id > 0` (used by `Model::save()`).

Compile-time diagnostics include:

- derive only allowed on named structs
- `id` field required and must be `i64`
- `created_at`/`updated_at` must be `i64` when present
- `deleted_at` must be `Option<i64>` when present
- invalid or duplicate `#[model(...)]` attributes
- invalid `#[validate(email)]` usage

## Testing

Macro diagnostics are covered with `trybuild` UI tests under `tests/ui`.
