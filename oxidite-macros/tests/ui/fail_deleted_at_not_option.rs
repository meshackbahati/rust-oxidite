use sqlx::FromRow;

#[derive(oxidite_macros::Model, FromRow)]
struct User {
    id: i64,
    email: String,
    deleted_at: i64,
}

fn main() {}
