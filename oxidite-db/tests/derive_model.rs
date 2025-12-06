use oxidite_db::{Model, sqlx, Database, DatabaseType, DbTransaction, Result};
use async_trait::async_trait;
use sqlx::any::AnyRow;

#[derive(Model, sqlx::FromRow)]
struct User {
    id: i64,
    username: String,
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
        // In a real mock we would inspect the query string here
        Ok(1)
    }
    async fn fetch_all<'q>(&self, _query: sqlx::query::Query<'q, sqlx::Any, sqlx::any::AnyArguments<'q>>) -> Result<Vec<AnyRow>> { Ok(vec![]) }
    async fn fetch_one<'q>(&self, _query: sqlx::query::Query<'q, sqlx::Any, sqlx::any::AnyArguments<'q>>) -> Result<Option<AnyRow>> { Ok(None) }
}

#[test]
fn test_metadata() {
    assert_eq!(User::table_name(), "users");
    let fields = User::fields();
    assert!(fields.contains(&"id"));
    assert!(fields.contains(&"username"));
}

#[tokio::test]
async fn test_crud_compilation() {
    let db = MockDb;
    let mut user = User { id: 1, username: "test".to_string() };
    
    // Test that methods exist and compile
    let _ = user.create(&db).await;
    let _ = user.update(&db).await;
    let _ = user.delete(&db).await;
}
