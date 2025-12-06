use oxidite_db::{Model, sqlx, Database, DatabaseType, DbTransaction, Result, chrono};
use async_trait::async_trait;
use sqlx::any::AnyRow;
use chrono::Utc;

#[derive(Model, sqlx::FromRow, Clone)]
struct UserWithTimestamps {
    id: i64,
    username: String,
    created_at: i64,
    updated_at: i64,
}

#[derive(Debug)]
struct MockDb;

#[async_trait]
impl Database for MockDb {
    fn db_type(&self) -> DatabaseType { DatabaseType::Sqlite }
    async fn execute(&self, _query: &str) -> Result<u64> { Ok(1) }
    async fn query(&self, _query: &str) -> Result<Vec<AnyRow>> { Ok(vec![]) }
    async fn query_one(&self, _query: &str) -> Result<Option<AnyRow>> { Ok(None) }
    async fn ping(&self) -> Result<()> { Ok(()) }
    async fn begin_transaction(&self) -> Result<DbTransaction> { unimplemented!() }
    
    async fn execute_query<'q>(&self, _query: sqlx::query::Query<'q, sqlx::Any, sqlx::any::AnyArguments<'q>>) -> Result<u64> {
        Ok(1)
    }
    async fn fetch_all<'q>(&self, _query: sqlx::query::Query<'q, sqlx::Any, sqlx::any::AnyArguments<'q>>) -> Result<Vec<AnyRow>> { Ok(vec![]) }
    async fn fetch_one<'q>(&self, _query: sqlx::query::Query<'q, sqlx::Any, sqlx::any::AnyArguments<'q>>) -> Result<Option<AnyRow>> { Ok(None) }
}

#[tokio::test]
async fn test_timestamps_compilation() {
    let db = MockDb;
    let mut user = UserWithTimestamps { 
        id: 1, 
        username: "test".to_string(),
        created_at: 0,
        updated_at: 0,
    };
    
    // Test create (should update created_at and updated_at on the struct)
    let _ = user.create(&db).await;
    
    // Test update (should update updated_at on the struct)
    let _ = user.update(&db).await;
}
