use oxidite_core::{OxiditeRequest, Result as OxiditeResult, Error};
use oxidite_db::Database;
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
        let query = format!(
            "SELECT r.* FROM roles r 
             INNER JOIN user_roles ur ON r.id = ur.role_id 
             WHERE ur.user_id = {} AND r.name = '{}'",
            user_id, self.role_name
        );
        
        let rows = self.db.query(&query).await
            .map_err(|_| Error::Server("Database error".to_string()))?;
        
        Ok(!rows.is_empty())
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
        let query = format!(
            "SELECT p.* FROM permissions p 
             INNER JOIN role_permissions rp ON p.id = rp.permission_id 
             INNER JOIN user_roles ur ON rp.role_id = ur.role_id 
             WHERE ur.user_id = {} AND p.name = '{}'",
            user_id, self.permission_name
        );
        
        let rows = self.db.query(&query).await
            .map_err(|_| Error::Server("Database error".to_string()))?;
        
        Ok(!rows.is_empty())
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
        let query = format!(
            "SELECT COUNT(*) as count FROM user_roles ur 
             INNER JOIN roles r ON ur.role_id = r.id 
             WHERE ur.user_id = {} AND r.name = '{}'",
            user_id, role_name
        );
        
        let rows = self.db.query(&query).await?;
        Ok(!rows.is_empty())
    }
    
    /// Check if user has a specific permission
    pub async fn user_can(&self, user_id: i64, permission_name: &str) -> oxidite_db::Result<bool> {
        let query = format!(
            "SELECT COUNT(*) as count FROM permissions p 
             INNER JOIN role_permissions rp ON p.id = rp.permission_id 
             INNER JOIN user_roles ur ON rp.role_id = ur.role_id 
             WHERE ur.user_id = {} AND p.name = '{}'",
            user_id, permission_name
        );
        
        let rows = self.db.query(&query).await?;
        Ok(!rows.is_empty())
    }
    
    /// Get all roles for a user
    pub async fn user_roles(&self, user_id: i64) -> oxidite_db::Result<Vec<Role>> {
        use oxidite_db::sqlx::FromRow;
        
        let query = format!(
            "SELECT r.* FROM roles r 
             INNER JOIN user_roles ur ON r.id = ur.role_id 
             WHERE ur.user_id = {}",
            user_id
        );
        
        let rows = self.db.query(&query).await?;
        let mut roles = Vec::new();
        
        for row in rows {
            roles.push(Role::from_row(&row)?);
        }
        
        Ok(roles)
    }
    
    /// Get all permissions for a user (through their roles)
    pub async fn user_permissions(&self, user_id: i64) -> oxidite_db::Result<Vec<Permission>> {
        use oxidite_db::sqlx::FromRow;
        
        let query = format!(
            "SELECT DISTINCT p.* FROM permissions p 
             INNER JOIN role_permissions rp ON p.id = rp.permission_id 
             INNER JOIN user_roles ur ON rp.role_id = ur.role_id 
             WHERE ur.user_id = {}",
            user_id
        );
        
        let rows = self.db.query(&query).await?;
        let mut permissions = Vec::new();
        
        for row in rows {
            permissions.push(Permission::from_row(&row)?);
        }
        
        Ok(permissions)
    }
    
    /// Assign role to user
    pub async fn assign_role(&self, user_id: i64, role_id: i64) -> oxidite_db::Result<()> {
        let query = format!(
            "INSERT OR IGNORE INTO user_roles (user_id, role_id) VALUES ({}, {})",
            user_id, role_id
        );
        self.db.execute(&query).await?;
        Ok(())
    }
    
    /// Remove role from user
    pub async fn remove_role(&self, user_id: i64, role_id: i64) -> oxidite_db::Result<()> {
        let query = format!(
            "DELETE FROM user_roles WHERE user_id = {} AND role_id = {}",
            user_id, role_id
        );
        self.db.execute(&query).await?;
        Ok(())
    }
}
