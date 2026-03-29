use oxidite_core::{OxiditeRequest, Result as OxiditeResult, Error};
use oxidite_db::Database;
use oxidite_db::sqlx::Row;
use std::sync::Arc;
use crate::rbac::{Role, Permission};

/// Middleware to require a specific role
pub struct RequireRole {
    role_name: String,
    db: Arc<dyn Database>,
}

impl RequireRole {
    pub fn new(role_name: impl Into<String>, db: Arc<dyn Database>) -> Self {
        Self {
            role_name: role_name.into(),
            db,
        }
    }
    
    pub async fn check(&self, req: &OxiditeRequest) -> OxiditeResult<bool> {
        // Get user_id from request extensions (set by auth middleware)
        let user_id = req.extensions()
            .get::<i64>()
            .ok_or_else(|| Error::Unauthorized("User not authenticated".to_string()))?;
        
        // Check if user has the required role
        let query = oxidite_db::sqlx::query(
            "SELECT 1 FROM roles r
             INNER JOIN user_roles ur ON r.id = ur.role_id
             WHERE ur.user_id = ? AND r.name = ?
             LIMIT 1"
        )
            .bind(*user_id)
            .bind(&self.role_name);

        let row = self.db.fetch_one(query).await
            .map_err(|_| Error::InternalServerError("Database error".to_string()))?;

        Ok(row.is_some())
    }
}

/// Middleware to require a specific permission
pub struct RequirePermission {
    permission_name: String,
    db: Arc<dyn Database>,
}

impl RequirePermission {
    pub fn new(permission_name: impl Into<String>, db: Arc<dyn Database>) -> Self {
        Self {
            permission_name: permission_name.into(),
            db,
        }
    }
    
    pub async fn check(&self, req: &OxiditeRequest) -> OxiditeResult<bool> {
        // Get user_id from request extensions
        let user_id = req.extensions()
            .get::<i64>()
            .ok_or_else(|| Error::Unauthorized("User not authenticated".to_string()))?;
        
        // Check if user has the required permission through any of their roles
        let query = oxidite_db::sqlx::query(
            "SELECT 1 FROM permissions p
             INNER JOIN role_permissions rp ON p.id = rp.permission_id
             INNER JOIN user_roles ur ON rp.role_id = ur.role_id
             WHERE ur.user_id = ? AND p.name = ?
             LIMIT 1"
        )
            .bind(*user_id)
            .bind(&self.permission_name);

        let row = self.db.fetch_one(query).await
            .map_err(|_| Error::InternalServerError("Database error".to_string()))?;

        Ok(row.is_some())
    }
}

/// Utility functions for authorization checks
pub struct AuthorizationService {
    db: Arc<dyn Database>,
}

impl AuthorizationService {
    pub fn new(db: Arc<dyn Database>) -> Self {
        Self { db }
    }
    
    /// Check if user has a specific role
    pub async fn user_has_role(&self, user_id: i64, role_name: &str) -> oxidite_db::Result<bool> {
        let query = oxidite_db::sqlx::query(
            "SELECT COUNT(*) as count FROM user_roles ur
             INNER JOIN roles r ON ur.role_id = r.id
             WHERE ur.user_id = ? AND r.name = ?"
        )
            .bind(user_id)
            .bind(role_name);

        let row = self.db.fetch_one(query).await?;
        Ok(row
            .and_then(|r| r.try_get::<i64, _>("count").ok())
            .unwrap_or(0) > 0)
    }
    
    /// Check if user has a specific permission
    pub async fn user_can(&self, user_id: i64, permission_name: &str) -> oxidite_db::Result<bool> {
        let query = oxidite_db::sqlx::query(
            "SELECT COUNT(*) as count FROM permissions p
             INNER JOIN role_permissions rp ON p.id = rp.permission_id
             INNER JOIN user_roles ur ON rp.role_id = ur.role_id
             WHERE ur.user_id = ? AND p.name = ?"
        )
            .bind(user_id)
            .bind(permission_name);

        let row = self.db.fetch_one(query).await?;
        Ok(row
            .and_then(|r| r.try_get::<i64, _>("count").ok())
            .unwrap_or(0) > 0)
    }
    
    /// Get all roles for a user
    pub async fn user_roles(&self, user_id: i64) -> oxidite_db::Result<Vec<Role>> {
        use oxidite_db::sqlx::FromRow;
        
        let query = oxidite_db::sqlx::query(
            "SELECT r.* FROM roles r
             INNER JOIN user_roles ur ON r.id = ur.role_id
             WHERE ur.user_id = ?"
        )
            .bind(user_id);

        let rows = self.db.fetch_all(query).await?;
        let mut roles = Vec::new();
        
        for row in rows {
            roles.push(Role::from_row(&row)?);
        }
        
        Ok(roles)
    }
    
    /// Get all permissions for a user (through their roles)
    pub async fn user_permissions(&self, user_id: i64) -> oxidite_db::Result<Vec<Permission>> {
        use oxidite_db::sqlx::FromRow;
        
        let query = oxidite_db::sqlx::query(
            "SELECT DISTINCT p.* FROM permissions p
             INNER JOIN role_permissions rp ON p.id = rp.permission_id
             INNER JOIN user_roles ur ON rp.role_id = ur.role_id
             WHERE ur.user_id = ?"
        )
            .bind(user_id);

        let rows = self.db.fetch_all(query).await?;
        let mut permissions = Vec::new();
        
        for row in rows {
            permissions.push(Permission::from_row(&row)?);
        }
        
        Ok(permissions)
    }
    
    /// Assign role to user
    pub async fn assign_role(&self, user_id: i64, role_id: i64) -> oxidite_db::Result<()> {
        // Use SELECT before INSERT for backend portability.
        let exists_query = oxidite_db::sqlx::query(
            "SELECT 1 FROM user_roles WHERE user_id = ? AND role_id = ? LIMIT 1"
        )
            .bind(user_id)
            .bind(role_id);

        if self.db.fetch_one(exists_query).await?.is_none() {
            let insert_query = oxidite_db::sqlx::query(
                "INSERT INTO user_roles (user_id, role_id) VALUES (?, ?)"
            )
                .bind(user_id)
                .bind(role_id);
            self.db.execute_query(insert_query).await?;
        }
        Ok(())
    }
    
    /// Remove role from user
    pub async fn remove_role(&self, user_id: i64, role_id: i64) -> oxidite_db::Result<()> {
        let query = oxidite_db::sqlx::query(
            "DELETE FROM user_roles WHERE user_id = ? AND role_id = ?"
        )
            .bind(user_id)
            .bind(role_id);
        self.db.execute_query(query).await?;
        Ok(())
    }
}
