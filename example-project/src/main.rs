use oxidite::prelude::*;
use oxidite::template::serve_static;
use oxidite_middleware::logger::Logger;

mod routes;
mod controllers;
mod models;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::load()
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    let addr = format!("{}:{}", config.server.host, config.server.port);

    let mut router = Router::new();
    routes::register(&mut router);

    // Static files fallback
    router.get("/*", serve_static);

    // Wrap router with request/response logging middleware.
    let server = Server::new(Logger::new(router));
    println!("🚀 Server running on http://{}", addr);
    server.listen(addr.parse().unwrap()).await
}
