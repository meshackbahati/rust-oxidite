use oxidite_db::{DbPool, Database, sqlx::Row};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Starting Database Demo");

    // Use SQLite for this example as it's self-contained
    let url = "sqlite::memory:?cache=shared";
    println!("Connecting to {}", url);

    let db = DbPool::connect(url).await?;
    println!("Connected to {:?}", db.db_type());

    // Create a table
    let setup_sql = "
        CREATE TABLE users (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL
        );
    ";
    db.execute(setup_sql).await?;
    println!("Created table 'users'");

    // Insert some data
    let insert_sql = "INSERT INTO users (name, email) VALUES ('Alice', 'alice@example.com')";
    let rows = db.execute(insert_sql).await?;
    println!("Inserted {} row(s)", rows);

    let insert_sql2 = "INSERT INTO users (name, email) VALUES ('Bob', 'bob@example.com')";
    let rows2 = db.execute(insert_sql2).await?;
    println!("Inserted {} row(s)", rows2);

    // Query data using the new query method
    let select_sql = "SELECT * FROM users";
    let results = db.query(select_sql).await?;
    println!("Found {} user(s)", results.len());

    for row in results {
        let name: String = row.try_get("name")?;
        let email: String = row.try_get("email")?;
        println!("User: {} ({})", name, email);
    }

    // Query one
    let one_sql = "SELECT * FROM users WHERE name = 'Alice'";
    if let Some(row) = db.query_one(one_sql).await? {
        let name: String = row.try_get("name")?;
        println!("Found specific user: {}", name);
    } else {
        println!("User not found");
    }

    db.ping().await?;
    println!("Ping successful");

    Ok(())
}
