use oxidite_db::Model;
use sqlx::FromRow;

#[derive(oxidite_macros::Model, FromRow)]
#[model(table = "accounts")]
struct Account {
    id: i64,
    email: String,
}

fn main() {
    assert_eq!(Account::table_name(), "accounts");
}
