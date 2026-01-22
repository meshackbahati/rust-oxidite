// Example: Simple Hello World with Oxidite

use oxidite::prelude::*;

async fn hello(_req: Request) -> Result<Response> {
    Ok(Response::text("Hello, Oxidite!"))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    router.get("/", hello);

    let server = Server::new(router);
    println!("Listening on http://127.0.0.1:3000");
    server.listen("127.0.0.1:3000".parse().unwrap()).await
}
