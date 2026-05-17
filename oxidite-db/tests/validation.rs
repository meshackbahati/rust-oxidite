extern crate oxidite_db as oxidite;

use oxidite_db::{Model, sqlx, Database, DatabaseType, DbTransaction, Result, DbInspector, TableSchema};
use async_trait::async_trait;
use sqlx::any::AnyRow;

#[derive(Model, sqlx::FromRow, Clone)]
struct UserWithValidation {
    id: i64,
    username: String,
    #[validate(email)]
    email: String,
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
    fn inspector(&self) -> Box<dyn DbInspector> {
        struct MockInspector;
        #[async_trait]
        impl DbInspector for MockInspector {
            async fn get_table_schema(&self, _table_name: &str) -> Result<Option<TableSchema>> { Ok(None) }
            async fn list_tables(&self) -> Result<Vec<String>> { Ok(vec![]) }
        }
        Box::new(MockInspector)
    }
}

#[tokio::test]
async fn test_email_validation_valid() {
    let db = MockDb;
    let user = UserWithValidation {
        id: 1,
        username: "test".to_string(),
        email: "test@example.com".to_string(),
    };
    
    assert!(user.validate(&db).await.is_ok());
}

#[tokio::test]
async fn test_email_validation_invalid() {
    let db = MockDb;
    let user = UserWithValidation {
        id: 1,
        username: "test".to_string(),
        email: "invalid-email".to_string(),
    };
    
    let result = user.validate(&db).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid email format"));
}

#[tokio::test]
async fn test_email_validation_missing_at() {
    let db = MockDb;
    let user = UserWithValidation {
        id: 1,
        username: "test".to_string(),
        email: "testexample.com".to_string(),
    };
    
    assert!(user.validate(&db).await.is_err());
}

#[tokio::test]
async fn test_email_validation_missing_domain() {
    let db = MockDb;
    let user = UserWithValidation {
        id: 1,
        username: "test".to_string(),
        email: "test@".to_string(),
    };
    
    assert!(user.validate(&db).await.is_err());
}
