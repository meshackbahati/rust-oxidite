//! Database migration system

use std::path::{Path, PathBuf};
use std::fs;
use chrono::Utc;
use sqlx::Row;

/// Migration file
#[derive(Debug)]
pub struct Migration {
    pub version: String,
    pub name: String,
    pub up_sql: String,
    pub down_sql: String,
}

impl Migration {
    pub fn new(name: &str) -> Self {
        let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
        let version = format!("{}_{}", timestamp, name);
        
        Self {
            version,
            name: name.to_string(),
            up_sql: String::new(),
            down_sql: String::new(),
        }
    }
    
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)?;
        
        let filename = path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid filename"))?;
        
        // Parse filename: 20240101120000_create_users
        let parts: Vec<&str> = filename.splitn(2, '_').collect();
        if parts.len() != 2 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid migration filename"));
        }
        
        let version = filename.to_string();
        let name = parts[1].to_string();
        
        // Split content into up/down SQL
        let sections: Vec<&str> = content.split("-- migrate:down").collect();
        let up_sql = sections.get(0)
            .unwrap_or(&"")
            .replace("-- migrate:up", "")
            .trim()
            .to_string();
        let down_sql = sections.get(1)
            .unwrap_or(&"")
            .trim()
            .to_string();
        
        Ok(Self {
            version,
            name,
            up_sql,
            down_sql,
        })
    }
    
    pub fn save(&self, migrations_dir: impl AsRef<Path>) -> Result<PathBuf, std::io::Error> {
        let migrations_dir = migrations_dir.as_ref();
        fs::create_dir_all(migrations_dir)?;
        
        let filename = format!("{}.sql", self.version);
        let path = migrations_dir.join(filename);
        
        let content = format!(
            "-- migrate:up\n{}\n\n-- migrate:down\n{}\n",
            self.up_sql,
            self.down_sql
        );
        
        fs::write(&path, content)?;
        Ok(path)
    }
}

/// Migration manager
pub struct MigrationManager {
    migrations_dir: PathBuf,
}

impl MigrationManager {
    pub fn new(migrations_dir: impl AsRef<Path>) -> Self {
        Self {
            migrations_dir: migrations_dir.as_ref().to_path_buf(),
        }
    }
    
    pub fn list_migrations(&self) -> Result<Vec<Migration>, std::io::Error> {
        let mut migrations = Vec::new();
        
        if !self.migrations_dir.exists() {
            return Ok(migrations);
        }
        
        for entry in fs::read_dir(&self.migrations_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("sql") {
                if let Ok(migration) = Migration::from_file(&path) {
                    migrations.push(migration);
                }
            }
        }
        
        // Sort by version
        migrations.sort_by(|a, b| a.version.cmp(&b.version));
        
        Ok(migrations)
    }
    
    pub fn create_migration(&self, name: &str) -> Result<PathBuf, std::io::Error> {
        let migration = Migration::new(name);
        migration.save(&self.migrations_dir)
    }
    
    /// Ensure migrations table exists
    pub async fn ensure_migrations_table(&self, db: &impl crate::Database) -> crate::Result<()> {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS _migrations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                version TEXT NOT NULL UNIQUE,
                applied_at INTEGER NOT NULL
            )
        "#;
        db.execute(sql).await?;
        Ok(())
    }
    
    /// Get list of applied migrations
    pub async fn get_applied_migrations(&self, db: &impl crate::Database) -> crate::Result<Vec<String>> {
        self.ensure_migrations_table(db).await?;
        
        let rows = db.query("SELECT version FROM _migrations ORDER BY version").await?;
        let mut versions = Vec::new();
        
        for row in rows {
            if let Ok(version) = row.try_get::<String, _>("version") {
                versions.push(version);
            }
        }
        
        Ok(versions)
    }
    
    /// Mark migration as applied
    pub async fn mark_migration_applied(&self, db: &impl crate::Database, version: &str) -> crate::Result<()> {
        self.ensure_migrations_table(db).await?;
        
        let timestamp = chrono::Utc::now().timestamp();
        let sql = format!(
            "INSERT INTO _migrations (version, applied_at) VALUES ('{}', {})",
            version, timestamp
        );
        db.execute(&sql).await?;
        Ok(())
    }
    
    /// Remove migration record (for rollback)
    pub async fn mark_migration_reverted(&self, db: &impl crate::Database, version: &str) -> crate::Result<()> {
        let sql = format!("DELETE FROM _migrations WHERE version = '{}'", version);
        db.execute(&sql).await?;
        Ok(())
    }
    
    /// Get pending migrations
    pub async fn get_pending_migrations(&self, db: &impl crate::Database) -> Result<Vec<Migration>, Box<dyn std::error::Error>> {
        let all_migrations = self.list_migrations()?;
        let applied = self.get_applied_migrations(db).await?;
        
        let pending: Vec<Migration> = all_migrations
            .into_iter()
            .filter(|m| !applied.contains(&m.version))
            .collect();
        
        Ok(pending)
    }
}
