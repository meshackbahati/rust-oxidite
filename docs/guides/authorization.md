# Authorization Guide

Role-based access control (RBAC) and permission management in Oxidite.

## Installation

```toml
[dependencies]
oxidite = { version = "1.0", features = ["auth", "database"] }
```

## Quick Start

```rust
use oxidite::prelude::*;
use oxidite_auth::{RequirePermission, RequireRole};

// Protect routes with permissions
app.delete("/users/:id", delete_user)
    .layer(RequirePermission::new("users.delete"));

app.get("/admin", admin_panel)
    .layer(RequireRole::new("admin"));
```

## Roles and Permissions

The `oxidite-auth` crate provides database models for `Role` and `Permission`. You can manage these in your application logic.

```rust
// Example: Creating a role and assigning it to a user
// (This is a simplified example, your actual implementation may vary)

// Create role
let admin_role = Role::create("admin", &db).await?;

// Create permission
let create_users_perm = Permission::create("users.create", &db).await?;

// Add permission to role
admin_role.add_permission(&create_users_perm, &db).await?;

// Assign role to user
user.assign_role(&admin_role, &db).await?;

// Check permission
if user.has_permission("users.create", &db).await? {
    // Allow action
}

// Check role
if user.has_role("admin", &db).await? {
    // Allow access
}
```

## Middleware

```rust
use oxidite_auth::{RequirePermission, RequireRole, RequireAnyPermission};

// Require specific permission
app.post("/posts", create_post)
    .layer(RequirePermission::new("posts.create"));

// Require specific role
app.get("/admin/dashboard", dashboard)
    .layer(RequireRole::new("admin"));

// Require any of multiple permissions
app.put("/posts/:id", update_post)
    .layer(RequireAnyPermission::new(vec![
        "posts.update.own".to_string(),
        "posts.update.all".to_string()
    ]));
```

## In Handlers

You can also perform authorization checks within your handlers.

```rust
use oxidite_auth::Auth;

async fn delete_user(
    auth: Auth,
    Path(id): Path<i64>,
    State(db): State<Arc<Database>>,
) -> Result<OxiditeResponse> {
    let user = User::find(auth.user_id, &db).await?;

    if !user.has_permission("users.delete", &db).await? {
        return Err(Error::Forbidden("You do not have permission to delete users.".to_string()));
    }
    
    User::delete(id, &db).await?;
    
    Ok(OxiditeResponse::json(json!({ "status": "ok" })))
}
```

## Dynamic Permissions

For more complex scenarios, you can implement custom logic in your handlers.

```rust
// Check at runtime
async fn can_edit_post(user: &User, post: &Post, db: &Database) -> Result<bool> {
    Ok(user.has_permission("posts.update.all", db).await? ||
    (user.has_permission("posts.update.own", db).await? && post.user_id == user.id))
}
```
