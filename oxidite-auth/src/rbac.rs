use oxidite_db::sqlx;
use oxidite_db::sqlx::FromRow;

#[derive(FromRow, Clone, Debug)]
pub struct Role {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(FromRow, Clone, Debug)]
pub struct Permission {
    pub id: i64,
    pub name: String,
    pub resource: String,
    pub action: String,
    pub description: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Role {
    /// Get all permissions for this role
    pub async fn permissions(&self, db: &impl oxidite_db::Database) -> oxidite_db::Result<Vec<Permission>> {
        let query = format!(
            "SELECT p.* FROM permissions p 
             INNER JOIN role_permissions rp ON p.id = rp.permission_id 
             WHERE rp.role_id = {}",
            self.id
        );
        
        let rows = db.query(&query).await?;
        let mut permissions = Vec::new();
        
        for row in rows {
            permissions.push(sqlx::FromRow::from_row(&row)?);
        }
        
        Ok(permissions)
    }
    
    /// Check if role has a specific permission
    pub async fn has_permission(&self, db: &impl oxidite_db::Database, permission_name: &str) -> oxidite_db::Result<bool> {
        let permissions = self.permissions(db).await?;
        Ok(permissions.iter().any(|p| p.name == permission_name))
    }
}

impl Permission {
    /// Check if permission matches resource and action
    pub fn matches(&self, resource: &str, action: &str) -> bool {
        self.resource == resource && self.action == action
    }
}
