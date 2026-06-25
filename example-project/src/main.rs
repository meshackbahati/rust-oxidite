use oxidite::prelude::*;
use oxidite::template::serve_static;
use oxidite_middleware::logger::Logger;

mod routes;
mod controllers;
mod models;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Load Config
    let config = Config::load()
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    let host = config.server.host.clone();
    let port = config.server.port;

    // 2. Build Router via Application coordinator
    let mut app = Application::new(config);
    routes::register(app.router_mut());
    app.router_mut().get("/*", serve_static);
    let router = app.into_router();

    // 3. Apply middleware
    // 4. Start Server
    let addr = format!("{host}:{port}");
    println!("🚀 Server running on http://{addr}");
    Server::new(Logger::new(router))
        .listen(addr.parse().unwrap())
        .await
}
