# Sequelize -> Oxidite ORM Cookbook

This cookbook maps common Sequelize model patterns to `oxidite-db` + `#[derive(Model)]`.

## Model mapping

Sequelize:

- `tableName` -> `#[model(table = "...")]`
- `timestamps: true` -> include `created_at: i64`, `updated_at: i64`
- `paranoid`/soft delete -> include `deleted_at: Option<i64>`

Oxidite example:

```rust,ignore
use oxidite_db::Model;

#[derive(Model, Debug, Clone)]
#[model(table = "ctf_events")]
pub struct CtfEvent {
    pub id: i64,
    pub title: String,
    pub state: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
}
```

## CRUD mapping

- `Model.create(...)` -> `MyModel::create(&db, model).await?`
- `Model.findByPk(id)` -> `MyModel::find_by_id(&db, id).await?`
- `instance.save()` -> `model.save_checked(&db).await?`
- `instance.destroy()` -> `model.delete(&db).await?`

## Query mapping

- `where` -> `MyModel::query().filter_eq("col", value)`
- `order` -> `.order_by("created_at", SortDirection::Desc)`
- `limit/offset` -> `.paginate(Pagination::from_page(page, per_page)?)`

## Associations

Use relation helpers in `oxidite_db::relations`:

- `HasMany`
- `HasOne`
- `BelongsTo`

Prefer eager loading helpers for N+1-sensitive paths.

## Keep raw SQL where needed

For complex analytics SQL, preserve SQL and execute via db query APIs first, then optimize later.

## Migration sequence

1. Port models without changing table schema.
2. Port read queries.
3. Port writes with transaction tests.
4. Port background-job database paths.
