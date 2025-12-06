use oxidite_db::{Model, sqlx, Database, DatabaseType, DbTransaction, Result, HasMany, HasOne, BelongsTo};
use async_trait::async_trait;
use sqlx::any::AnyRow;

#[derive(Model, sqlx::FromRow, Clone)]
struct User {
    id: i64,
    username: String,
}

#[derive(Model, sqlx::FromRow, Clone)]
struct Post {
    id: i64,
    user_id: i64,
    title: String,
}

#[derive(Debug)]
struct MockDb;

#[async_trait]
impl Database for MockDb {
    fn db_type(&self) -> DatabaseType { DatabaseType::Sqlite }
    async fn execute(&self, _query: &str) -> Result<u64> { Ok(1) }
    async fn query(&self, _query: &str) -> Result<Vec<AnyRow>> { Ok(vec![]) } // Mock empty result
    async fn query_one(&self, _query: &str) -> Result<Option<AnyRow>> { Ok(None) }
    async fn ping(&self) -> Result<()> { Ok(()) }
    async fn begin_transaction(&self) -> Result<DbTransaction> { unimplemented!() }
    async fn execute_query<'q>(&self, _query: sqlx::query::Query<'q, sqlx::Any, sqlx::any::AnyArguments<'q>>) -> Result<u64> { Ok(1) }
    async fn fetch_all<'q>(&self, _query: sqlx::query::Query<'q, sqlx::Any, sqlx::any::AnyArguments<'q>>) -> Result<Vec<AnyRow>> { Ok(vec![]) }
    async fn fetch_one<'q>(&self, _query: sqlx::query::Query<'q, sqlx::Any, sqlx::any::AnyArguments<'q>>) -> Result<Option<AnyRow>> { Ok(None) }
}

impl User {
    fn posts(&self) -> HasMany<User, Post> {
        HasMany::new(self.id, "user_id")
    }
    
    fn profile(&self) -> HasOne<User, Post> { // Just for testing HasOne
        HasOne::new(self.id, "user_id")
    }
}

impl Post {
    fn user(&self) -> BelongsTo<Post, User> {
        BelongsTo::new(self.user_id)
    }
}

#[tokio::test]
async fn test_relationships_compilation() {
    let db = MockDb;
    let user = User { id: 1, username: "test".to_string() };
    let post = Post { id: 1, user_id: 1, title: "test".to_string() };
    
    // Test HasMany
    let _posts = user.posts().get(&db).await;
    
    // Test HasOne
    let _profile = user.profile().get(&db).await;
    
    // Test BelongsTo
    let _user = post.user().get(&db).await;
}
