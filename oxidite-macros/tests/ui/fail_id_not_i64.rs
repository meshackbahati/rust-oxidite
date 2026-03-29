use sqlx::FromRow;

#[derive(oxidite_macros::Model, FromRow)]
struct User {
    id: i32,
    email: String,
}

fn main() {}
