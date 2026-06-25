use std::net::SocketAddr;

use oxidite_config::Config;

use crate::error::{Error, Result};
use crate::router::Router;
use crate::server::Server;

/// Application boot coordinator.
///
/// Orchestrates a predictable startup sequence:
/// **Config ──> Router ──> Middleware ──> Server**
///
/// Routes are registered macro-free via explicit `router_mut()` calls.
/// Middleware is applied by wrapping the router before passing to `Server`.
///
/// # Example
///
/// ```rust,no_run
/// use oxidite_core::{Application, Request, Response, Error, Result};
///
/// async fn hello(_req: Request) -> Result<Response> {
///     Ok(Response::text("Hello, World!"))
/// }
///
/// # async fn run() -> Result<()> {
/// let config = oxidite_config::Config::default();
/// let mut app = Application::new(config);
/// app.router_mut().get("/", hello);
/// app.run().await
/// # }
/// ```
pub struct Application {
    config: Config,
    router: Router,
}

impl Application {
    /// Create a new Application with the given configuration.
    pub fn new(config: Config) -> Self {
        Application {
            config,
            router: Router::new(),
        }
    }

    /// Borrow the inner [`Router`] immutably.
    ///
    /// Useful for inspecting registered routes without modifying them.
    pub fn router(&self) -> &Router {
        &self.router
    }

    /// Mutable borrow the inner [`Router`] for registering routes and middleware.
    pub fn router_mut(&mut self) -> &mut Router {
        &mut self.router
    }

    /// Borrow the application configuration immutably.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Consume the application and return the configured Router.
    ///
    /// Use this when you need to apply middleware before starting the server:
    ///
    /// ```rust,no_run
    /// # use oxidite_core::{Application, Router, Response, Server};
    /// # use oxidite_core::Error;
    /// # async fn run() -> Result<(), Error> {
    /// # let config = oxidite_config::Config::default();
    /// # let mut app = Application::new(config);
    /// let router = app.into_router();
    /// // apply middleware, then serve
    /// # Ok(())
    /// # }
    /// ```
    pub fn into_router(self) -> Router {
        self.router
    }

    /// Start the server using the configured host and port from `Config`.
    ///
    /// This is the terminal step in the boot sequence:
    /// Config ──> Router ──> Middleware ──> Server
    ///
    /// If you need to apply middleware, call `into_router()` first,
    /// wrap the router, and then use `Server::new(wrapped).listen(addr)` directly.
    pub async fn run(self) -> Result<()> {
        let host = &self.config.server.host;
        let port = self.config.server.port;
        let addr_str = format!("{host}:{port}");
        let addr: SocketAddr = addr_str.parse().map_err(|e| {
            Error::InternalServerError(format!("invalid server address `{addr_str}`: {e}"))
        })?;
        Server::new(self.router).listen(addr).await
    }
}
