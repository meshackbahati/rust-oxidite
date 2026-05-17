use serde::{Deserialize, Serialize};
use crate::DatabaseType;

/// Supported database column types in Oxidite
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ColumnType {
    BigInt,
    Int,
    Text,
    Varchar(usize),
    Boolean,
    Float,
    DateTime,
    Json,
    Uuid,
}

impl ColumnType {
    /// Convert the column type to its backend-specific SQL representation
    pub fn to_sql(&self, db_type: DatabaseType) -> String {
        match (self, db_type) {
            (ColumnType::BigInt, _) => "BIGINT".to_string(),
            (ColumnType::Int, _) => "INTEGER".to_string(),
            (ColumnType::Text, _) => "TEXT".to_string(),
            (ColumnType::Varchar(size), _) => format!("VARCHAR({})", size),
            (ColumnType::Boolean, DatabaseType::Sqlite) => "INTEGER".to_string(),
            (ColumnType::Boolean, _) => "BOOLEAN".to_string(),
            (ColumnType::Float, _) => "DOUBLE PRECISION".to_string(),
            (ColumnType::DateTime, DatabaseType::Sqlite) => "INTEGER".to_string(),
            (ColumnType::DateTime, _) => "TIMESTAMP".to_string(),
            (ColumnType::Json, DatabaseType::Sqlite) => "TEXT".to_string(),
            (ColumnType::Json, DatabaseType::Postgres) => "JSONB".to_string(),
            (ColumnType::Json, _) => "JSON".to_string(),
            (ColumnType::Uuid, DatabaseType::Sqlite) => "TEXT".to_string(),
            (ColumnType::Uuid, _) => "UUID".to_string(),
        }
    }
}

/// Metadata for a single database column
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ColumnSchema {
    pub name: String,
    pub ty: ColumnType,
    pub nullable: bool,
    pub primary_key: bool,
    pub default: Option<String>,
}

/// Metadata for a database table
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<ColumnSchema>,
}

impl TableSchema {
    /// Generate a CREATE TABLE SQL statement for this schema
    pub fn to_create_sql(&self, db_type: DatabaseType) -> String {
        let mut sql = format!("CREATE TABLE {} (\n", self.name);
        let mut cols = Vec::new();
        
        for col in &self.columns {
            let mut col_sql = format!("    {} {}", col.name, col.ty.to_sql(db_type));
            
            if col.primary_key {
                col_sql.push_str(" PRIMARY KEY");
                if db_type == DatabaseType::Sqlite && col.ty == ColumnType::BigInt {
                    // Auto-increment for SQLite primary keys
                    col_sql.push_str(" AUTOINCREMENT");
                }
            } else if !col.nullable {
                col_sql.push_str(" NOT NULL");
            }
            
            if let Some(default) = &col.default {
                col_sql.push_str(&format!(" DEFAULT {}", default));
            }
            
            cols.push(col_sql);
        }
        
        sql.push_str(&cols.join(",\n"));
        sql.push_str("\n);");
        sql
    }
}
