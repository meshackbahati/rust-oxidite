use sqlx::{any::{AnyPoolOptions, AnyRow}, AnyPool, Transaction};
use std::{fmt::Debug, future::Future};
use thiserror::Error;

pub use sqlx;

pub mod migrations;
pub use migrations::{Migration, MigrationManager};

pub mod relations;
pub use relations::{HasMany, HasOne, BelongsTo};

pub type Result<T> = std::result::Result<T, sqlx::Error>;
pub type OrmResult<T> = std::result::Result<T, OrmError>;

#[derive(Debug, Error)]
pub enum OrmError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("model `{model}` with id `{id}` was not found")]
    NotFound { model: &'static str, id: i64 },
    #[error("invalid SQL identifier `{value}` for {kind}")]
    InvalidIdentifier {
        kind: &'static str,
        value: String,
    },
    #[error("invalid pagination: {0}")]
    InvalidPagination(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pagination {
    pub limit: usize,
    pub offset: usize,
}

impl Pagination {
    pub fn new(limit: usize, offset: usize) -> OrmResult<Self> {
        if limit == 0 {
            return Err(OrmError::InvalidPagination("limit must be greater than 0"));
        }
        Ok(Self { limit, offset })
    }

    pub fn from_page(page: usize, per_page: usize) -> OrmResult<Self> {
        if page == 0 {
            return Err(OrmError::InvalidPagination("page must be 1 or greater"));
        }
        if per_page == 0 {
            return Err(OrmError::InvalidPagination("per_page must be greater than 0"));
        }

        Ok(Self {
            limit: per_page,
            offset: (page - 1) * per_page,
        })
    }
}

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
        
        let db_type = parse_database_type(url)?;

        Ok(Self { pool, db_type })
    }

    /// Execute a closure within a transaction and automatically commit or rollback.
    pub async fn with_transaction<T, F, Fut>(&self, operation: F) -> Result<T>
    where
        F: FnOnce(&DbTransaction) -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let tx = self.begin_transaction().await?;
        match operation(&tx).await {
            Ok(value) => {
                tx.commit().await?;
                Ok(value)
            }
            Err(err) => {
                let _ = tx.rollback().await;
                Err(err)
            }
        }
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
        Ok(DbTransaction {
            tx: Arc::new(Mutex::new(Some(tx))),
            db_type: self.db_type,
        })
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
    db_type: DatabaseType,
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
        self.db_type
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl SortDirection {
    fn as_sql(self) -> &'static str {
        match self {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        }
    }
}

#[derive(Debug, Clone)]
#[doc(hidden)]
pub enum QueryValue {
    I64(i64),
    String(String),
    Bool(bool),
    F64(f64),
    Uuid(String),
    DateTimeUtc(i64),
    Json(String),
}

impl From<i64> for QueryValue {
    fn from(value: i64) -> Self {
        Self::I64(value)
    }
}

impl From<i32> for QueryValue {
    fn from(value: i32) -> Self {
        Self::I64(value as i64)
    }
}

impl From<&str> for QueryValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<String> for QueryValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<bool> for QueryValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<f64> for QueryValue {
    fn from(value: f64) -> Self {
        Self::F64(value)
    }
}

impl From<uuid::Uuid> for QueryValue {
    fn from(value: uuid::Uuid) -> Self {
        Self::Uuid(value.to_string())
    }
}

impl From<chrono::DateTime<chrono::Utc>> for QueryValue {
    fn from(value: chrono::DateTime<chrono::Utc>) -> Self {
        Self::DateTimeUtc(value.timestamp())
    }
}

impl From<serde_json::Value> for QueryValue {
    fn from(value: serde_json::Value) -> Self {
        Self::Json(value.to_string())
    }
}

#[derive(Debug, Clone)]
enum Filter {
    Eq { column: String, value: QueryValue },
    Like { column: String, value: String },
    IsNull { column: String },
    IsNotNull { column: String },
}

#[derive(Debug, Clone)]
enum QueryBuildError {
    InvalidIdentifier {
        kind: &'static str,
        value: String,
    },
    EmptySelectFields,
}

impl QueryBuildError {
    fn into_orm_error(self) -> OrmError {
        match self {
            QueryBuildError::InvalidIdentifier { kind, value } => {
                OrmError::InvalidIdentifier { kind, value }
            }
            QueryBuildError::EmptySelectFields => {
                OrmError::InvalidPagination("select fields cannot be empty")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModelQuery<M: Model> {
    select_fields: Vec<String>,
    filters: Vec<Filter>,
    order_by: Vec<(String, SortDirection)>,
    limit: Option<usize>,
    offset: Option<usize>,
    include_soft_deleted: bool,
    build_error: Option<QueryBuildError>,
    _phantom: std::marker::PhantomData<M>,
}

impl<M: Model> Default for ModelQuery<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: Model> ModelQuery<M> {
    pub fn new() -> Self {
        Self {
            select_fields: vec!["*".to_string()],
            filters: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
            include_soft_deleted: false,
            build_error: None,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn select(mut self, fields: &[&str]) -> Self {
        if fields.is_empty() {
            self.build_error = Some(QueryBuildError::EmptySelectFields);
            return self;
        }

        let mut projected = Vec::with_capacity(fields.len());
        for field in fields {
            if !is_valid_identifier(field) {
                self.build_error = Some(QueryBuildError::InvalidIdentifier {
                    kind: "column",
                    value: (*field).to_string(),
                });
                return self;
            }
            projected.push((*field).to_string());
        }

        self.select_fields = projected;
        self
    }

    pub fn filter_eq(mut self, column: &str, value: impl Into<QueryValue>) -> Self {
        if !is_valid_identifier(column) {
            self.build_error = Some(QueryBuildError::InvalidIdentifier {
                kind: "column",
                value: column.to_string(),
            });
            return self;
        }

        self.filters.push(Filter::Eq {
            column: column.to_string(),
            value: value.into(),
        });
        self
    }

    pub fn filter_like(mut self, column: &str, value: impl Into<String>) -> Self {
        if !is_valid_identifier(column) {
            self.build_error = Some(QueryBuildError::InvalidIdentifier {
                kind: "column",
                value: column.to_string(),
            });
            return self;
        }

        self.filters.push(Filter::Like {
            column: column.to_string(),
            value: value.into(),
        });
        self
    }

    pub fn filter_is_null(mut self, column: &str) -> Self {
        if !is_valid_identifier(column) {
            self.build_error = Some(QueryBuildError::InvalidIdentifier {
                kind: "column",
                value: column.to_string(),
            });
            return self;
        }

        self.filters.push(Filter::IsNull {
            column: column.to_string(),
        });
        self
    }

    pub fn filter_is_not_null(mut self, column: &str) -> Self {
        if !is_valid_identifier(column) {
            self.build_error = Some(QueryBuildError::InvalidIdentifier {
                kind: "column",
                value: column.to_string(),
            });
            return self;
        }

        self.filters.push(Filter::IsNotNull {
            column: column.to_string(),
        });
        self
    }

    pub fn order_by(mut self, column: &str, direction: SortDirection) -> Self {
        if !is_valid_identifier(column) {
            self.build_error = Some(QueryBuildError::InvalidIdentifier {
                kind: "column",
                value: column.to_string(),
            });
            return self;
        }
        self.order_by.push((column.to_string(), direction));
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

    pub fn paginate(mut self, pagination: Pagination) -> Self {
        self.limit = Some(pagination.limit);
        self.offset = Some(pagination.offset);
        self
    }

    pub fn with_deleted(mut self) -> Self {
        self.include_soft_deleted = true;
        self
    }

    fn build_sql(&self, count_only: bool) -> OrmResult<(String, Vec<QueryValue>)> {
        if let Some(err) = self.build_error.as_ref() {
            return Err(err.clone().into_orm_error());
        }

        if !is_valid_identifier(M::table_name()) {
            return Err(OrmError::InvalidIdentifier {
                kind: "table name",
                value: M::table_name().to_string(),
            });
        }

        let select = if count_only {
            "COUNT(*) as count".to_string()
        } else {
            self.select_fields.join(", ")
        };

        let mut sql = format!("SELECT {} FROM {}", select, M::table_name());
        let mut clauses = Vec::new();
        let mut binds = Vec::new();

        if M::has_soft_delete() && !self.include_soft_deleted {
            clauses.push("deleted_at IS NULL".to_string());
        }

        for filter in &self.filters {
            match filter {
                Filter::Eq { column, value } => {
                    clauses.push(format!("{column} = ?"));
                    binds.push(value.clone());
                }
                Filter::Like { column, value } => {
                    clauses.push(format!("{column} LIKE ?"));
                    binds.push(QueryValue::String(value.clone()));
                }
                Filter::IsNull { column } => clauses.push(format!("{column} IS NULL")),
                Filter::IsNotNull { column } => clauses.push(format!("{column} IS NOT NULL")),
            }
        }

        if !clauses.is_empty() {
            sql.push_str(&format!(" WHERE {}", clauses.join(" AND ")));
        }

        if !count_only && !self.order_by.is_empty() {
            let order = self
                .order_by
                .iter()
                .map(|(column, direction)| format!("{} {}", column, direction.as_sql()))
                .collect::<Vec<_>>()
                .join(", ");
            sql.push_str(&format!(" ORDER BY {order}"));
        }

        if !count_only {
            if let Some(limit) = self.limit {
                sql.push_str(&format!(" LIMIT {limit}"));
            }
            if let Some(offset) = self.offset {
                sql.push_str(&format!(" OFFSET {offset}"));
            }
        }

        Ok((sql, binds))
    }

    pub async fn fetch_all(self, db: &impl Database) -> OrmResult<Vec<M>> {
        let (sql, binds) = self.build_sql(false)?;
        let mut query = sqlx::query(&sql);
        for bind in binds {
            query = match bind {
                QueryValue::I64(value) => query.bind(value),
                QueryValue::String(value) => query.bind(value),
                QueryValue::Bool(value) => query.bind(value),
                QueryValue::F64(value) => query.bind(value),
                QueryValue::Uuid(value) => query.bind(value),
                QueryValue::DateTimeUtc(value) => query.bind(value),
                QueryValue::Json(value) => query.bind(value),
            };
        }

        let rows = db.fetch_all(query).await?;
        let mut models = Vec::with_capacity(rows.len());
        for row in rows {
            models.push(M::from_row(&row)?);
        }
        Ok(models)
    }

    pub async fn fetch_one(self, db: &impl Database) -> OrmResult<Option<M>> {
        let (sql, binds) = self.limit(1).build_sql(false)?;
        let mut query = sqlx::query(&sql);
        for bind in binds {
            query = match bind {
                QueryValue::I64(value) => query.bind(value),
                QueryValue::String(value) => query.bind(value),
                QueryValue::Bool(value) => query.bind(value),
                QueryValue::F64(value) => query.bind(value),
                QueryValue::Uuid(value) => query.bind(value),
                QueryValue::DateTimeUtc(value) => query.bind(value),
                QueryValue::Json(value) => query.bind(value),
            };
        }

        let row = db.fetch_one(query).await?;
        match row {
            Some(row) => Ok(Some(M::from_row(&row)?)),
            None => Ok(None),
        }
    }

    pub async fn count(self, db: &impl Database) -> OrmResult<i64> {
        use sqlx::Row;

        let (sql, binds) = self.build_sql(true)?;
        let mut query = sqlx::query(&sql);
        for bind in binds {
            query = match bind {
                QueryValue::I64(value) => query.bind(value),
                QueryValue::String(value) => query.bind(value),
                QueryValue::Bool(value) => query.bind(value),
                QueryValue::F64(value) => query.bind(value),
                QueryValue::Uuid(value) => query.bind(value),
                QueryValue::DateTimeUtc(value) => query.bind(value),
                QueryValue::Json(value) => query.bind(value),
            };
        }

        let row = db.fetch_one(query).await?;
        let row = row.ok_or(OrmError::NotFound {
            model: M::table_name(),
            id: 0,
        })?;
        Ok(row.try_get::<i64, _>("count")?)
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
        self.where_clauses
            .push(format!("{} = '{}'", column, escape_sql_literal(value)));
        self
    }

    pub fn order_by(mut self, column: &str, direction: &str) -> Self {
        let normalized = if direction.eq_ignore_ascii_case("desc") {
            "DESC"
        } else {
            "ASC"
        };
        self.order_by.push(format!("{} {}", column, normalized));
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

    /// Start a typed query for this model.
    fn query() -> ModelQuery<Self> {
        ModelQuery::new()
    }

    /// Find a record by ID
    async fn find(db: &impl Database, id: i64) -> Result<Option<Self>> {
        let mut query = format!("SELECT * FROM {} WHERE id = ?", Self::table_name());
        if Self::has_soft_delete() {
            query.push_str(" AND deleted_at IS NULL");
        }
        let row = db.fetch_one(sqlx::query(&query).bind(id)).await?;
        
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
        let rows = db.fetch_all(sqlx::query(&query)).await?;
        
        let mut models = Vec::new();
        for row in rows {
            models.push(Self::from_row(&row)?);
        }
        Ok(models)
    }

    /// Find a record by ID and return a typed not-found error when missing.
    async fn find_or_fail(db: &impl Database, id: i64) -> OrmResult<Self> {
        Self::find(db, id)
            .await?
            .ok_or(OrmError::NotFound {
                model: Self::table_name(),
                id,
            })
    }

    /// Fetch a page of records with optional soft-delete filtering.
    async fn all_paginated(db: &impl Database, pagination: Pagination) -> OrmResult<Vec<Self>> {
        Self::query().paginate(pagination).fetch_all(db).await
    }

    /// Find multiple rows by id.
    async fn find_many(db: &impl Database, ids: &[i64]) -> Result<Vec<Self>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders = std::iter::repeat("?")
            .take(ids.len())
            .collect::<Vec<_>>()
            .join(", ");
        let mut query = format!(
            "SELECT * FROM {} WHERE id IN ({})",
            Self::table_name(),
            placeholders
        );

        if Self::has_soft_delete() {
            query.push_str(" AND deleted_at IS NULL");
        }

        let mut sql_query = sqlx::query(&query);
        for id in ids {
            sql_query = sql_query.bind(*id);
        }

        let rows = db.fetch_all(sql_query).await?;
        let mut models = Vec::with_capacity(rows.len());
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

    /// Whether this model represents an already-persisted row.
    /// Override for custom primary-key strategies.
    fn is_persisted(&self) -> bool {
        false
    }

    /// Save (create or update)
    async fn save(&mut self, db: &impl Database) -> Result<()> {
        if let Err(e) = self.validate() {
            return Err(sqlx::Error::Protocol(e.into()));
        }

        if self.is_persisted() {
            self.update(db).await
        } else {
            self.create(db).await
        }
    }

    /// Validate and save using a typed ORM error surface.
    async fn save_checked(&mut self, db: &impl Database) -> OrmResult<()> {
        if let Err(err) = self.validate() {
            return Err(OrmError::Validation(err));
        }
        self.save(db).await?;
        Ok(())
    }

    /// Insert many models in sequence.
    async fn insert_many(db: &impl Database, models: &mut [Self]) -> Result<()> {
        for model in models {
            model.create(db).await?;
        }
        Ok(())
    }

    /// Update many models in sequence.
    async fn update_many(db: &impl Database, models: &mut [Self]) -> Result<()> {
        for model in models {
            model.update(db).await?;
        }
        Ok(())
    }
}

fn parse_database_type(url: &str) -> Result<DatabaseType> {
    if url.starts_with("postgres://") || url.starts_with("postgresql://") {
        return Ok(DatabaseType::Postgres);
    }
    if url.starts_with("mysql://") {
        return Ok(DatabaseType::MySql);
    }
    if url.starts_with("sqlite://") {
        return Ok(DatabaseType::Sqlite);
    }

    Err(sqlx::Error::Configuration(
        format!(
            "unsupported database URL scheme for `{url}`; expected postgres://, postgresql://, mysql://, or sqlite://"
        )
        .into(),
    ))
}

fn escape_sql_literal(value: &str) -> String {
    value.replace('\'', "''")
}

pub(crate) fn is_valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use crate as oxidite_db;
    use super::{parse_database_type, DatabaseType, Model, Pagination, QueryBuilder, QueryValue, SortDirection};
    use crate::sqlx;

    #[allow(dead_code)]
    #[derive(Model, sqlx::FromRow)]
    struct TestModel {
        id: i64,
        name: String,
        deleted_at: Option<i64>,
    }

    #[test]
    fn query_builder_escapes_single_quotes() {
        let query = QueryBuilder::new("users")
            .where_eq("name", "O'Reilly")
            .build();
        assert_eq!(query, "SELECT * FROM users WHERE name = 'O''Reilly'");
    }

    #[test]
    fn query_builder_normalizes_order_direction() {
        let asc_query = QueryBuilder::new("users").order_by("id", "something").build();
        assert_eq!(asc_query, "SELECT * FROM users ORDER BY id ASC");

        let desc_query = QueryBuilder::new("users").order_by("id", "DESC").build();
        assert_eq!(desc_query, "SELECT * FROM users ORDER BY id DESC");
    }

    #[test]
    fn parse_database_type_rejects_unknown_scheme() {
        let err = parse_database_type("unknown://localhost").expect_err("expected failure");
        let msg = err.to_string();
        assert!(msg.contains("unsupported database URL scheme"));
    }

    #[test]
    fn parse_database_type_accepts_supported_schemes() {
        assert_eq!(
            parse_database_type("postgres://localhost/db").unwrap(),
            DatabaseType::Postgres
        );
        assert_eq!(
            parse_database_type("mysql://localhost/db").unwrap(),
            DatabaseType::MySql
        );
        assert_eq!(
            parse_database_type("sqlite://db.sqlite").unwrap(),
            DatabaseType::Sqlite
        );
    }

    #[test]
    fn pagination_validation() {
        assert!(Pagination::new(0, 0).is_err());
        assert!(Pagination::from_page(0, 10).is_err());
        assert!(Pagination::from_page(1, 0).is_err());

        let pagination = Pagination::from_page(2, 10).unwrap();
        assert_eq!(pagination.limit, 10);
        assert_eq!(pagination.offset, 10);
    }

    #[test]
    fn model_query_builds_safe_sql_with_soft_delete_and_pagination() {
        let pagination = Pagination::from_page(2, 25).unwrap();
        let (sql, _binds) = TestModel::query()
            .filter_eq("name", "alice")
            .order_by("id", SortDirection::Desc)
            .paginate(pagination)
            .build_sql(false)
            .unwrap();

        assert_eq!(
            sql,
            "SELECT * FROM testmodels WHERE deleted_at IS NULL AND name = ? ORDER BY id DESC LIMIT 25 OFFSET 25"
        );
    }

    #[test]
    fn model_query_rejects_invalid_identifier() {
        let result = TestModel::query()
            .filter_eq("name;drop_table", "alice")
            .build_sql(false);
        assert!(result.is_err());
    }

    #[test]
    fn query_value_supports_uuid_datetime_and_json() {
        let uuid = uuid::Uuid::new_v4();
        let now = chrono::Utc::now();
        let json = serde_json::json!({"k":"v"});

        assert!(matches!(QueryValue::from(uuid), QueryValue::Uuid(_)));
        assert!(matches!(
            QueryValue::from(now),
            QueryValue::DateTimeUtc(_)
        ));
        assert!(matches!(QueryValue::from(json), QueryValue::Json(_)));
    }

    #[test]
    fn derived_model_persisted_state_uses_id() {
        let persisted = TestModel {
            id: 1,
            name: "alice".to_string(),
            deleted_at: None,
        };
        let new_model = TestModel {
            id: 0,
            name: "bob".to_string(),
            deleted_at: None,
        };

        assert!(persisted.is_persisted());
        assert!(!new_model.is_persisted());
    }
}
