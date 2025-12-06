use sqlx::{any::{AnyPoolOptions, AnyRow}, AnyPool, Transaction};
use std::fmt::Debug;

pub use sqlx;

pub mod migrations;
pub use migrations::{Migration, MigrationManager};

pub mod relations;
pub use relations::{HasMany, HasOne, BelongsTo};

pub type Result<T> = std::result::Result<T, sqlx::Error>;

pub use oxidite_macros::Model;
pub use async_trait::async_trait;
pub use chrono;
pub use regex;
pub use once_cell;

/// Database backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    Postgres,
    MySql,
    Sqlite,
}

/// Connection pool configuration
#[derive(Debug, Clone)]
pub struct PoolOptions {
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: std::time::Duration,
    pub idle_timeout: Option<std::time::Duration>,
}

impl Default for PoolOptions {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 0,
            connect_timeout: std::time::Duration::from_secs(30),
            idle_timeout: Some(std::time::Duration::from_secs(600)), // 10 minutes
        }
    }
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
    
    /// Begin a transaction
    async fn begin_transaction(&self) -> Result<DbTransaction>;

    /// Execute a sqlx Query
    async fn execute_query<'q>(&self, query: sqlx::query::Query<'q, sqlx::Any, sqlx::any::AnyArguments<'q>>) -> Result<u64>;

    /// Fetch all from a sqlx Query
    async fn fetch_all<'q>(&self, query: sqlx::query::Query<'q, sqlx::Any, sqlx::any::AnyArguments<'q>>) -> Result<Vec<AnyRow>>;

    /// Fetch one from a sqlx Query
    async fn fetch_one<'q>(&self, query: sqlx::query::Query<'q, sqlx::Any, sqlx::any::AnyArguments<'q>>) -> Result<Option<AnyRow>>;
}

/// Database connection pool wrapper
#[derive(Clone, Debug)]
pub struct DbPool {
    pool: AnyPool,
    db_type: DatabaseType,
}

impl DbPool {
    pub async fn connect(url: &str) -> Result<Self> {
        Self::connect_with_options(url, PoolOptions::default()).await
    }
    
    pub async fn connect_with_options(url: &str, options: PoolOptions) -> Result<Self> {
        sqlx::any::install_default_drivers();
        let max_conns = if url.contains(":memory:") { 1 } else { options.max_connections };
        
        let mut pool_options = AnyPoolOptions::new()
            .max_connections(max_conns)
            .min_connections(options.min_connections)
            .acquire_timeout(options.connect_timeout);
        
        if let Some(idle_timeout) = options.idle_timeout {
            pool_options = pool_options.idle_timeout(idle_timeout);
        }
        
        let pool = pool_options.connect(url).await?;
        
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
    
    async fn begin_transaction(&self) -> Result<DbTransaction> {
        let tx = self.pool.begin().await?;
        Ok(DbTransaction { tx: Arc::new(Mutex::new(Some(tx))) })
    }

    async fn execute_query<'q>(&self, query: sqlx::query::Query<'q, sqlx::Any, sqlx::any::AnyArguments<'q>>) -> Result<u64> {
        let result = query.execute(&self.pool).await?;
        Ok(result.rows_affected())
    }

    async fn fetch_all<'q>(&self, query: sqlx::query::Query<'q, sqlx::Any, sqlx::any::AnyArguments<'q>>) -> Result<Vec<AnyRow>> {
        let rows = query.fetch_all(&self.pool).await?;
        Ok(rows)
    }

    async fn fetch_one<'q>(&self, query: sqlx::query::Query<'q, sqlx::Any, sqlx::any::AnyArguments<'q>>) -> Result<Option<AnyRow>> {
        let row = query.fetch_optional(&self.pool).await?;
        Ok(row)
    }
}

use std::sync::Arc;
use tokio::sync::Mutex;

/// Database transaction
#[derive(Clone, Debug)]
pub struct DbTransaction {
    tx: Arc<Mutex<Option<Transaction<'static, sqlx::Any>>>>,
}

impl DbTransaction {
    /// Execute a query within the transaction
    pub async fn execute(&self, query: &str) -> Result<u64> {
        let mut lock = self.tx.lock().await;
        if let Some(ref mut tx) = *lock {
            let result = sqlx::query(query).execute(&mut **tx).await?;
            Ok(result.rows_affected())
        } else {
            Err(sqlx::Error::PoolClosed)
        }
    }

    /// Query multiple rows within the transaction
    pub async fn query(&self, query: &str) -> Result<Vec<AnyRow>> {
        let mut lock = self.tx.lock().await;
        if let Some(ref mut tx) = *lock {
            let rows = sqlx::query(query).fetch_all(&mut **tx).await?;
            Ok(rows)
        } else {
            Err(sqlx::Error::PoolClosed)
        }
    }

    /// Query one row within the transaction
    pub async fn query_one(&self, query: &str) -> Result<Option<AnyRow>> {
        let mut lock = self.tx.lock().await;
        if let Some(ref mut tx) = *lock {
            let row = sqlx::query(query).fetch_optional(&mut **tx).await?;
            Ok(row)
        } else {
            Err(sqlx::Error::PoolClosed)
        }
    }

    /// Commit the transaction
    pub async fn commit(self) -> Result<()> {
        let mut lock = self.tx.lock().await;
        if let Some(tx) = lock.take() {
            tx.commit().await?;
        }
        Ok(())
    }

    /// Rollback the transaction
    pub async fn rollback(self) -> Result<()> {
        let mut lock = self.tx.lock().await;
        if let Some(tx) = lock.take() {
            tx.rollback().await?;
        }
        Ok(())
    }
}

#[async_trait]
impl Database for DbTransaction {
    fn db_type(&self) -> DatabaseType {
        // Still a placeholder, but consistent
        DatabaseType::Postgres 
    }

    async fn execute(&self, query: &str) -> Result<u64> {
        self.execute(query).await
    }

    async fn query(&self, query: &str) -> Result<Vec<AnyRow>> {
        self.query(query).await
    }

    async fn query_one(&self, query: &str) -> Result<Option<AnyRow>> {
        self.query_one(query).await
    }

    async fn ping(&self) -> Result<()> {
        self.execute("SELECT 1").await?;
        Ok(())
    }
    
    async fn begin_transaction(&self) -> Result<DbTransaction> {
        // Nested transactions not supported by this simple wrapper yet
        // Could use savepoints if needed.
        Err(sqlx::Error::Configuration("Nested transactions not supported".into()))
    }

    async fn execute_query<'q>(&self, query: sqlx::query::Query<'q, sqlx::Any, sqlx::any::AnyArguments<'q>>) -> Result<u64> {
        let mut lock = self.tx.lock().await;
        if let Some(ref mut tx) = *lock {
            let result = query.execute(&mut **tx).await?;
            Ok(result.rows_affected())
        } else {
            Err(sqlx::Error::PoolClosed)
        }
    }

    async fn fetch_all<'q>(&self, query: sqlx::query::Query<'q, sqlx::Any, sqlx::any::AnyArguments<'q>>) -> Result<Vec<AnyRow>> {
        let mut lock = self.tx.lock().await;
        if let Some(ref mut tx) = *lock {
            let rows = query.fetch_all(&mut **tx).await?;
            Ok(rows)
        } else {
            Err(sqlx::Error::PoolClosed)
        }
    }

    async fn fetch_one<'q>(&self, query: sqlx::query::Query<'q, sqlx::Any, sqlx::any::AnyArguments<'q>>) -> Result<Option<AnyRow>> {
        let mut lock = self.tx.lock().await;
        if let Some(ref mut tx) = *lock {
            let row = query.fetch_optional(&mut **tx).await?;
            Ok(row)
        } else {
            Err(sqlx::Error::PoolClosed)
        }
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
/// Model trait for database entities
#[async_trait]
pub trait Model: Sized + Send + Sync + Unpin + for<'r> sqlx::FromRow<'r, AnyRow> {
    /// Get the table name
    fn table_name() -> &'static str;

    /// Get the list of fields (columns)
    fn fields() -> &'static [&'static str];

    /// Check if the model supports soft deletes
    fn has_soft_delete() -> bool {
        false
    }

    /// Find a record by ID
    async fn find(db: &impl Database, id: i64) -> Result<Option<Self>> {
        let mut query = format!("SELECT * FROM {} WHERE id = {}", Self::table_name(), id);
        if Self::has_soft_delete() {
            query.push_str(" AND deleted_at IS NULL");
        }
        let row = db.query_one(&query).await?;
        
        match row {
            Some(row) => Ok(Some(Self::from_row(&row)?)),
            None => Ok(None),
        }
    }

    /// Find all records
    async fn all(db: &impl Database) -> Result<Vec<Self>> {
        let mut query = format!("SELECT * FROM {}", Self::table_name());
        if Self::has_soft_delete() {
            query.push_str(" WHERE deleted_at IS NULL");
        }
        let rows = db.query(&query).await?;
        
        let mut models = Vec::new();
        for row in rows {
            models.push(Self::from_row(&row)?);
        }
        Ok(models)
    }
    
    /// Create a new record
    async fn create(&mut self, db: &impl Database) -> Result<()>;

    /// Update an existing record
    async fn update(&mut self, db: &impl Database) -> Result<()>;

    /// Delete the record (soft delete if supported, otherwise hard delete)
    async fn delete(&self, db: &impl Database) -> Result<()>;
    
    /// Force delete the record (hard delete)
    async fn force_delete(&self, db: &impl Database) -> Result<()>;
    
    /// Validate the model fields
    fn validate(&self) -> std::result::Result<(), String> {
        Ok(())
    }

    /// Save (create or update)
    async fn save(&mut self, db: &impl Database) -> Result<()> {
        if let Err(e) = self.validate() {
            return Err(sqlx::Error::Protocol(e.into()));
        }
        // This default impl is tricky without knowing if it's new.
        // For now, let's leave it to the user or macro to decide.
        // But we can provide a helper if we had an `is_new()` method.
        self.create(db).await
    }
}
