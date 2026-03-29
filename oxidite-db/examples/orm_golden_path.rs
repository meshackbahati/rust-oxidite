use oxidite_db::{DbPool, Database, Model, Pagination, SortDirection, sqlx};

#[allow(dead_code)]
#[derive(Model, sqlx::FromRow, Debug)]
#[model(table = "users")]
struct User {
    id: i64,
    name: String,
    email: String,
    created_at: i64,
    updated_at: i64,
    deleted_at: Option<i64>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = DbPool::connect("sqlite::memory:").await?;

    db.execute(
        r#"
        CREATE TABLE users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            deleted_at INTEGER NULL
        )
        "#,
    )
    .await?;

    let mut user = User {
        id: 0,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        created_at: 0,
        updated_at: 0,
        deleted_at: None,
    };

    user.create(&db).await?;

    let users = User::query()
        .filter_like("email", "%@example.com")
        .order_by("id", SortDirection::Asc)
        .paginate(Pagination::from_page(1, 20)?)
        .fetch_all(&db)
        .await?;

    println!("loaded {} user(s)", users.len());

    // Raw SQL remains available for advanced or vendor-specific queries.
    let row = db
        .query_one("SELECT COUNT(*) AS count FROM users")
        .await?
        .ok_or("count row missing")?;
    let count: i64 = sqlx::Row::try_get(&row, "count")?;
    println!("raw count: {count}");

    Ok(())
}
