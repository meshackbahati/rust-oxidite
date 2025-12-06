use oxidite_db::{Model, sqlx, Database, DatabaseType, DbTransaction, Result};
use async_trait::async_trait;
use sqlx::any::AnyRow;

#[derive(Model, sqlx::FromRow, Clone)]
struct UserWithSoftDelete {
    id: i64,
    username: String,
    deleted_at: Option<i64>,
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

#[test]
fn test_has_soft_delete() {
    assert!(UserWithSoftDelete::has_soft_delete());
}

#[tokio::test]
async fn test_soft_delete_compilation() {
    let db = MockDb;
    let user = UserWithSoftDelete { 
        id: 1, 
        username: "test".to_string(),
        deleted_at: None,
    };
    
    // Test delete (should be soft)
    let _ = user.delete(&db).await;
    
    // Test force delete (should be hard)
    let _ = user.force_delete(&db).await;
}
