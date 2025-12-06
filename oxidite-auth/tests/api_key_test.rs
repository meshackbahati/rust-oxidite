use oxidite_auth::{ApiKey, ApiKeyMiddleware};
use oxidite_db::{Database, DatabaseType, DbTransaction, Result};
use async_trait::async_trait;
use sqlx::any::AnyRow;
use std::sync::Arc;

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
fn test_api_key_generation() {
    let key1 = ApiKey::generate_key();
    let key2 = ApiKey::generate_key();
    
    // Keys should start with ox_ prefix
    assert!(key1.starts_with("ox_"));
    assert!(key2.starts_with("ox_"));
    
    // Keys should be unique
    assert_ne!(key1, key2);
    
    // Keys should be reasonably long
    assert!(key1.len() > 40);
}

#[test]
fn test_api_key_hashing() {
    let key = "ox_test_key_12345";
    let hash1 = ApiKey::hash_key(key);
    let hash2 = ApiKey::hash_key(key);
    
    // Same key should produce same hash
    assert_eq!(hash1, hash2);
    
    // Hash should be hex string
    assert!(hash1.chars().all(|c| c.is_ascii_hexdigit()));
    
    // Different keys should produce different hashes
    let different_key = "ox_different_key";
    let hash3 = ApiKey::hash_key(different_key);
    assert_ne!(hash1, hash3);
}

#[tokio::test]
async fn test_api_key_middleware() {
    let db = Arc::new(MockDb);
    let _middleware = ApiKeyMiddleware::new(db);
    
    // In a real integration test with database, we would test:
    // - Key extraction from headers
    // - Key verification
    // - User ID injection into request extensions
    
    // For now, just verify it compiles and instantiates
    assert!(true);
}
