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
use oxidite::auth::rbac::*;

// Protect routes with permissions
app.delete("/users/:id", delete_user)
    .middleware(RequirePermission::new("users.delete"));

app.get("/secure", secure_page)
    .middleware(RequireRole::new("admin"));
```

## Roles and Permissions

```rust
// Create role
let admin_role = Role {
    name: "admin".to_string(),
    permissions: vec![
        "users.create".to_string(),
        "users.read".to_string(),
        "users.update".to_string(),
        "users.delete".to_string(),
    ],
};

// Assign to user
user.assign_role(&db, "admin").await?;

// Check permission
if user.has_permission("users.delete") {
    // Allow action
}

// Check role
if user.has_role("admin") {
    // Allow access
}
```

## Middleware

```rust
use oxidite::auth::rbac::*;

// Require specific permission
app.post("/posts", create_post)
    .middleware(RequirePermission::new("posts.create"));

// Require specific role
app.get("/analytics", analytics_page)
    .middleware(RequireRole::new("admin"));

// Require any of multiple permissions
app.put("/posts/:id", update_post)
    .middleware(RequireAnyPermission::new(vec![
        "posts.update.own",
        "posts.update.all"
    ]));
```

## In Handlers

```rust
async fn delete_user(
    auth: Auth,
    Path(params): Path<HashMap<String, String>>,
) -> Result<Response> {
    // Auth automatically injected by middleware
    if !auth.user.has_permission("users.delete") {
        return Err(Error::Forbidden);
    }
    
    let user_id = params.get("id").unwrap().parse()?;
    User::delete(&db, user_id).await?;
    
    Ok(Response::ok())
}
```

## Permission Hierarchy

```rust
// users.* grants all user permissions
let super_admin = Role {
    name: "super_admin".to_string(),
    permissions: vec!["*".to_string()], // All permissions
};
```

## Dynamic Permissions

```rust
// Check at runtime
async fn can_edit_post(user: &User, post: &Post) -> bool {
    user.has_permission("posts.update.all") ||
    (user.has_permission("posts.update.own") && post.user_id == user.id)
}
```

Complete examples at [docs.rs/oxidite](https://docs.rs/oxidite)
