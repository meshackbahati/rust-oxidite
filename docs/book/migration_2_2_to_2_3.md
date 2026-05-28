# Migration Guide: Oxidite 2.2.x to 2.3

Oxidite 2.3 is a major release focusing on production readiness and framework stabilization. It introduces several breaking changes to improve ergonomics and solve long-standing issues.

## 1. Handler Trait - Expanded Extractor Support
Handlers now support up to 12 extractors total (previously limited to 3). No changes are required to existing handlers, but you can now remove combined extractor workarounds.

```rust
// Now possible in 2.3
async fn complex_handler(
    State(state): State<AppState>,
    Auth(user): Auth,
    Path(id): Path<i64>,
    Query(q): Query<Params>,
    Json(body): Json<Data>,
    HeaderMap(headers): HeaderMap,
) -> Result<Response>
```

## 2. Router Layering
The `Router` now has a `.layer()` method to apply global middleware. This replaces manual per-route middleware application for common layers.

```rust
let mut app = Router::new();
app.layer(CorsLayer::permissive());
app.layer(LoggerLayer::new());
```

## 3. WebSocket Auth Preservation
The `WebSocketUpgrade::on_upgrade` method now passes request extensions to the callback. This allows preserving authentication state after the upgrade.

**Before:**
```rust
ws.on_upgrade(|socket| async move { ... })
```

**After:**
```rust
ws.on_upgrade(|socket, extensions| async move {
    let user = extensions.get::<AuthenticatedUser>();
    // ...
})
```

## 4. DbPool Accessors and Concrete Pool Support

`DbPool` now exposes the underlying `sqlx` pool through `.pool()`, `.inner()`, and `.as_sqlx_pool()` methods. Use these to execute raw SQL queries that require `sqlx` specific features.

```rust
let pool = db.pool();
let users = sqlx::query_as!(User, "SELECT * FROM users WHERE ...")
    .fetch_all(pool)
    .await?;
```

### Concrete Pool Access (New in 2.3.1)

For database-specific features like PostgreSQL JSONB, arrays, or `#[derive(FromRow)]` with complex types, you can now access the concrete pool directly:

```rust
// Get PostgreSQL-specific pool
let pg_pool = db.postgres_pool().expect("PostgreSQL required");

// Use with sqlx::query_as for complex types
let logs = sqlx::query_as::<_, AdminAuditLog>(
    "SELECT * FROM admin_audit_logs ORDER BY created_at DESC LIMIT $1"
)
.bind(20i64)
.fetch_all(pg_pool)
.await?;
```

Convenience methods are also available:
- `db.fetch_all_as(sql, |q| q.bind(...))`
- `db.fetch_optional_as(sql, |q| q.bind(...))`
- `db.fetch_one_as(sql, |q| q.bind(...))`

**Note**: Using concrete pools doubles the connection count (creates both `AnyPool` and `PgPool`). Configure `max_connections` accordingly.

## 5. Model ID Flexibility
The `Model` derive macro now supports `i64`, `Uuid`, and `String` for the `id` field. `OrmError::NotFound` now stores the ID as a `String`.

If you were manually matching on `OrmError::NotFound { id, .. }`, you will need to update it to expect a `String`.

## 6. Config System
`SecurityConfig` now includes `cors_allowed_methods` and `cors_allowed_headers` fields. Update your `oxidite.toml` if you need to configure these.
