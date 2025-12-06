# Authorization & Access Control Guide

This guide covers Role-Based Access Control (RBAC) and Permission-Based Access Control (PBAC) in Oxidite.

## Overview

Oxidite provides a flexible authorization system that supports:
- **Role-Based Access Control (RBAC)**: Assign roles to users
- **Permission-Based Access Control (PBAC)**: Grant granular permissions through roles
- **Middleware Protection**: Secure routes with role/permission checks

## Database Schema

The RBAC system uses four tables:

```sql
roles              -- Define roles (admin, user, guest)
permissions        -- Define permissions (users.create, posts.read)
role_permissions   -- Associate permissions with roles
user_roles         -- Assign roles to users
```

Run the migration:
```bash
oxidite migrate run
oxidite seed run  # Seeds default roles and permissions
```

## Default Roles & Permissions

The seed creates three default roles:

| Role    | Description                   | Permissions                           |
|---------|-------------------------------|---------------------------------------|
| `admin` | Full system access            | All permissions                       |
| `user`  | Regular authenticated user    | Read users, create/read/update posts  |
| `guest` | Read-only access              | Read users, read posts                |

Default permissions follow the `resource.action` pattern:
- `users.create`, `users.read`, `users.update`, `users.delete`
- `posts.create`, `posts.read`, `posts.update`, `posts.delete`

## Using the Authorization Service

### Setup

```rust
use oxidite_auth::AuthorizationService;
use oxidite_db::DbPool;
use std::sync::Arc;

let db = DbPool::connect("sqlite://data.db").await?;
let auth = AuthorizationService::new(Arc::new(db));
```

### Check User Roles

```rust
// Check if user has admin role
if auth.user_has_role(user_id, "admin").await? {
    println!("User is an administrator");
}
```

### Check User Permissions

```rust
// Check if user can create posts
if auth.user_can(user_id, "posts.create").await? {
    // Allow post creation
} else {
    return Err(Error::Forbidden("Permission denied".to_string()));
}
```

### Get User Roles & Permissions

```rust
// Get all roles for a user
let roles = auth.user_roles(user_id).await?;
for role in roles {
    println!("Role: {} - {}", role.name, role.description.unwrap_or_default());
}

// Get all permissions for a user
let permissions = auth.user_permissions(user_id).await?;
for perm in permissions {
    println!("Permission: {} ({}.{})", perm.name, perm.resource, perm.action);
}
```

### Assign/Remove Roles

```rust
// Assign admin role to user
auth.assign_role(user_id, 1).await?; // 1 = admin role ID

// Remove role from user
auth.remove_role(user_id, 1).await?;
```

## Middleware Protection

### Require Specific Role

```rust
use oxidite_auth::RequireRole;
use oxidite_core::{Router, OxiditeRequest, OxiditeResponse};

async fn admin_dashboard(req: OxiditeRequest) -> Result<OxiditeResponse> {
    Ok(Response::new(Full::new(Bytes::from("Admin Dashboard"))))
}

// In your route setup
let require_admin = RequireRole::new("admin", db.clone());

router.get("/admin", move |req| async move {
    if !require_admin.check(&req).await? {
        return Err(Error::Forbidden("Admin access required".to_string()));
    }
    admin_dashboard(req).await
});
```

### Require Specific Permission

```rust
use oxidite_auth::RequirePermission;

let require_create_users = RequirePermission::new("users.create", db.clone());

router.post("/users", move |req| async move {
    if !require_create_users.check(&req).await? {
        return Err(Error::Forbidden("Missing permission: users.create".to_string()));
    }
    create_user(req).await
});
```

## Authorization Patterns

### Handler-Level Authorization

```rust
async fn delete_post(req: OxiditeRequest) -> Result<OxiditeResponse> {
    let user_id = req.extensions().get::<i64>()
        .ok_or(Error::Unauthorized("Not authenticated".to_string()))?;
    
    let auth = AuthorizationService::new(db);
    
    // Check permission
    if !auth.user_can(*user_id, "posts.delete").await? {
        return Err(Error::Forbidden("Cannot delete posts".to_string()));
    }
    
    // Proceed with deletion
    Ok(Response::new(Full::new(Bytes::from("Post deleted"))))
}
```

### Resource Ownership Check

```rust
async fn update_post(req: OxiditeRequest, post_id: i64) -> Result<OxiditeResponse> {
    let user_id = *req.extensions().get::<i64>().unwrap();
    
    // Get post from database
    let post = Post::find(&db, post_id).await?
        .ok_or(Error::NotFound)?;
    
    // Check if user owns the post OR has admin role
    let auth = AuthorizationService::new(db);
    let is_owner = post.author_id == user_id;
    let is_admin = auth.user_has_role(user_id, "admin").await?;
    
    if !is_owner && !is_admin {
        return Err(Error::Forbidden("Not authorized to update this post".to_string()));
    }
    
    // Proceed with update
    Ok(Response::new(Full::new(Bytes::from("Post updated"))))
}
```

### Multiple Permission Check (OR)

```rust
async fn view_analytics(req: OxiditeRequest) -> Result<OxiditeResponse> {
    let user_id = *req.extensions().get::<i64>().unwrap();
    let auth = AuthorizationService::new(db);
    
    // User needs EITHER admin role OR analytics.view permission
    let is_admin = auth.user_has_role(user_id, "admin").await?;
    let can_view_analytics = auth.user_can(user_id, "analytics.view").await?;
    
    if !is_admin && !can_view_analytics {
        return Err(Error::Forbidden("Access denied".to_string()));
    }
    
    Ok(Response::new(Full::new(Bytes::from("Analytics data"))))
}
```

### Multiple Permission Check (AND)

```rust
async fn publish_critical_content(req: OxiditeRequest) -> Result<OxiditeResponse> {
    let user_id = *req.extensions().get::<i64>().unwrap();
    let auth = AuthorizationService::new(db);
    
    // User needs BOTH content.create AND content.publish permissions
    let can_create = auth.user_can(user_id, "content.create").await?;
    let can_publish = auth.user_can(user_id, "content.publish").await?;
    
    if !can_create || !can_publish {
        return Err(Error::Forbidden("Insufficient permissions".to_string()));
    }
    
    Ok(Response::new(Full::new(Bytes::from("Content published"))))
}
```

## Creating Custom Roles & Permissions

### Create a New Role

```rust
use oxidite_auth::Role;

let mut moderator = Role {
    id: 0,  // Will be auto-generated
    name: "moderator".to_string(),
    description: Some("Content moderator".to_string()),
    created_at: chrono::Utc::now().timestamp(),
    updated_at: chrono::Utc::now().timestamp(),
};

// Save to database using raw SQL (Model trait not needed for RBAC)
db.execute(&format!(
    "INSERT INTO roles (name, description, created_at, updated_at) VALUES ('{}', '{}', {}, {})",
    moderator.name, moderator.description.unwrap_or_default(),
    moderator.created_at, moderator.updated_at
)).await?;
```

### Create a New Permission

```rust
use oxidite_auth::Permission;

db.execute(&format!(
    "INSERT INTO permissions (name, resource, action, description, created_at, updated_at) 
     VALUES ('comments.moderate', 'comments', 'moderate', 'Moderate user comments', {}, {})",
    chrono::Utc::now().timestamp(), chrono::Utc::now().timestamp()
)).await?;
```

### Assign Permission to Role

```rust
// Assign comments.moderate permission to moderator role
db.execute(
    "INSERT INTO role_permissions (role_id, permission_id) 
     SELECT r.id, p.id FROM roles r, permissions p 
     WHERE r.name = 'moderator' AND p.name = 'comments.moderate'"
).await?;
```

## Best Practices

1. **Use Permissions for Fine-Grained Control**: Prefer checking permissions over roles when possible
   ```rust
   // Good
   if auth.user_can(user_id, "posts.delete").await? { }
   
   // Less flexible
   if auth.user_has_role(user_id, "admin").await? { }
   ```

2. **Follow Naming Conventions**: Use `resource.action` format for permissions
   - `users.create`, `posts.update`, `comments.moderate`

3. **Store User ID in Request Extensions**: Set this in your authentication middleware
   ```rust
   req.extensions_mut().insert(user_id);
   ```

4. **Cache Permission Checks**: For high-traffic routes, cache authorization results
   ```rust
   // Cache user permissions for the request lifecycle
   let permissions = auth.user_permissions(user_id).await?;
   req.extensions_mut().insert(permissions);
   ```

5. **Fail Secure**: Default to denying access
   ```rust
   let has_permission = auth.user_can(user_id, "admin.panel").await.unwrap_or(false);
   if !has_permission {
       return Err(Error::Forbidden("Access denied".to_string()));
   }
   ```

## Error Handling

```rust
match auth.user_can(user_id, "posts.create").await {
    Ok(true) => {
        // User has permission
    },
    Ok(false) => {
        return Err(Error::Forbidden("Missing permission: posts.create".to_string()));
    },
    Err(e) => {
        // Database error
        eprintln!("Authorization check failed: {:?}", e);
        return Err(Error::Server("Authorization error".to_string()));
    }
}
```

## Testing Authorization

```rust
#[tokio::test]
async fn test_user_authorization() {
    let db = setup_test_db().await;
    let auth = AuthorizationService::new(Arc::new(db));
    
    // Create test user and assign role
    auth.assign_role(user_id, admin_role_id).await.unwrap();
    
    // Test permission
    assert!(auth.user_can(user_id, "users.create").await.unwrap());
    assert!(!auth.user_can(user_id, "nonexistent.permission").await.unwrap());
}
```

## Next Steps

- Implement API key authentication with per-key permissions
- Add rate limiting based on user roles
- Explore attribute-based access control (ABAC) for complex scenarios
