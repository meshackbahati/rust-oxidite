#[derive(oxidite_macros::Model)]
#[model(foo = "bar")]
struct Bad {
    id: i64,
}

fn main() {}
