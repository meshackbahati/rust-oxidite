use oxidite_db::Model;
use sqlx::FromRow;

#[derive(oxidite_macros::Model, FromRow)]
#[model(table_name = "people")]
struct Person {
    id: i64,
    email: String,
}

fn main() {
    assert_eq!(Person::table_name(), "people");
}
