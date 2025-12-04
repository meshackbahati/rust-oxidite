use async_trait::async_trait;
use sqlx::{any::{AnyPoolOptions, AnyRow}, AnyPool};
use std::fmt::Debug;

pub use sqlx;

pub type Result<T> = std::result::Result<T, sqlx::Error>;

/// Database backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    Postgres,
    MySql,
    Sqlite,
}

/// Common database trait
#[async_trait]
pub trait Database: Send + Sync + Debug {
    /// Get the database type
    fn db_type(&self) -> DatabaseType;

    /// Execute a query
    async fn execute(&self, query: &str) -> Result<u64>;

    /// Query multiple rows
    async fn query(&self, query: &str) -> Result<Vec<AnyRow>>;

    /// Query one row
    async fn query_one(&self, query: &str) -> Result<Option<AnyRow>>;

    /// Check health
    async fn ping(&self) -> Result<()>;
}

/// Database connection pool wrapper
#[derive(Clone, Debug)]
pub struct DbPool {
    pool: AnyPool,
    db_type: DatabaseType,
}

impl DbPool {
    pub async fn connect(url: &str) -> Result<Self> {
        sqlx::any::install_default_drivers();
        let max_conns = if url.contains(":memory:") { 1 } else { 5 };
        let pool = AnyPoolOptions::new()
            .max_connections(max_conns)
            .connect(url)
            .await?;
        
        let db_type = if url.starts_with("postgres://") || url.starts_with("postgresql://") {
            DatabaseType::Postgres
        } else if url.starts_with("mysql://") {
            DatabaseType::MySql
        } else if url.starts_with("sqlite://") {
            DatabaseType::Sqlite
        } else {
            // Default or unknown, maybe panic or error? 
            // For AnyPool, the scheme matters.
            // Let's assume sqlite if not specified? No, AnyPool needs scheme.
            // We can try to infer from the pool kind but AnyPool hides it well.
            // Let's just rely on the URL scheme for now.
            DatabaseType::Sqlite 
        };

        Ok(Self { pool, db_type })
    }
}

#[async_trait]
impl Database for DbPool {
    fn db_type(&self) -> DatabaseType {
        self.db_type
    }

    async fn execute(&self, query: &str) -> Result<u64> {
        let result = sqlx::query(query).execute(&self.pool).await?;
        Ok(result.rows_affected())
    }

    async fn query(&self, query: &str) -> Result<Vec<AnyRow>> {
        let rows = sqlx::query(query).fetch_all(&self.pool).await?;
        Ok(rows)
    }

    async fn query_one(&self, query: &str) -> Result<Option<AnyRow>> {
        let row = sqlx::query(query).fetch_optional(&self.pool).await?;
        Ok(row)
    }

    async fn ping(&self) -> Result<()> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }
}

/// Query builder (simplified for now)
pub struct QueryBuilder {
    table: String,
    select_fields: Vec<String>,
    where_clauses: Vec<String>,
    order_by: Vec<String>,
    limit: Option<usize>,
    offset: Option<usize>,
}

impl QueryBuilder {
    pub fn new(table: &str) -> Self {
        Self {
            table: table.to_string(),
            select_fields: vec!["*".to_string()],
            where_clauses: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
        }
    }

    pub fn select(mut self, fields: &[&str]) -> Self {
        self.select_fields = fields.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn where_eq(mut self, column: &str, value: &str) -> Self {
        self.where_clauses.push(format!("{} = '{}'", column, value));
        self
    }

    pub fn order_by(mut self, column: &str, direction: &str) -> Self {
        self.order_by.push(format!("{} {}", column, direction));
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn build(&self) -> String {
        let mut query = format!("SELECT {} FROM {}", self.select_fields.join(", "), self.table);

        if !self.where_clauses.is_empty() {
            query.push_str(&format!(" WHERE {}", self.where_clauses.join(" AND ")));
        }

        if !self.order_by.is_empty() {
            query.push_str(&format!(" ORDER BY {}", self.order_by.join(", ")));
        }

        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        query
    }
}
