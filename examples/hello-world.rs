// Example: Simple Hello World with Oxidite

use oxidite_core::{Router, Server, OxiditeRequest, OxiditeResponse, Result};
use http_body_util::Full;
use bytes::Bytes;

async fn hello(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    Ok(hyper::Response::new(Full::new(Bytes::from("Hello, Oxidite!"))))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    router.get("/", hello);

    let server = Server::new(router);
    println!("Listening on http://127.0.0.1:3000");
    server.listen("127.0.0.1:3000".parse().unwrap()).await
}
