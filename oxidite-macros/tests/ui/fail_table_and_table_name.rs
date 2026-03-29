use sqlx::FromRow;

#[derive(oxidite_macros::Model, FromRow)]
#[model(table = "users", table_name = "users")]
struct User {
    id: i64,
    email: String,
}

fn main() {}
