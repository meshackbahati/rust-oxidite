//! # Oxidite Web Framework
//!
//! Oxidite is a modern, batteries-included web framework for Rust, inspired by Laravel and Rails.
//!
//! ## Quick Start
//!
//! ```toml
//! [dependencies]
//! oxidite = "1.0"
//! tokio = { version = "1", features = ["full"] }
//! serde = { version = "1", features = ["derive"] }
//! ```
//!
//! ```rust,no_run
//! use oxidite::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let mut app = Router::new();
//!     
//!     app.get("/", |_req| async {
//!         Ok(Response::text("Hello, Oxidite!"))
//!     });
//!     
//!     Server::new(app)
//!         .listen("127.0.0.1:3000".parse().unwrap())
//!         .await
//! }
//! ```
//!
//! ## Features
//!
//! - **HTTP Server**: HTTP/1.1, HTTP/2, and WebSocket support
//! - **Routing**: Path parameters, query parsing, API versioning
//! - **Middleware**: CORS, logging, compression, rate limiting
//! - **Database**: ORM with relationships, migrations, soft deletes
//! - **Authentication**: RBAC, JWT, OAuth2, 2FA, API keys
//! - **Background Jobs**: Cron scheduling, retry logic, dead letter queue
//! - **Caching**: Memory and Redis backends
//! - **Real-time**: WebSocket support with pub/sub
//! - **Templates**: Server-side rendering
//! - **Email**: SMTP support
//! - **File Storage**: Local and S3 backends

// Re-export core types
pub use oxidite_core::*;
pub use oxidite_middleware;
pub use oxidite_config;

#[cfg(feature = "database")]
pub use oxidite_db as db;

#[cfg(feature = "auth")]
pub use oxidite_auth as auth;

#[cfg(feature = "queue")]
pub use oxidite_queue as queue;

#[cfg(feature = "cache")]
pub use oxidite_cache as cache;

#[cfg(feature = "realtime")]
pub use oxidite_realtime as realtime;

#[cfg(feature = "templates")]
pub use oxidite_template as template;

#[cfg(feature = "mail")]
pub use oxidite_mail as mail;

#[cfg(feature = "storage")]
pub use oxidite_storage as storage;

#[cfg(feature = "security")]
pub use oxidite_security as security;

#[cfg(feature = "utils")]
pub use oxidite_utils as utils;

/// Prelude module for common imports
pub mod prelude {
    pub use oxidite_core::{
        Router, Server, Handler,
        Error, Result,
        Request, Response,
        extract::{Json, Path, Query, State, FromRequest, Form, Cookies, Body},
    };
    
    pub use oxidite_middleware::{
        ServiceBuilder, LoggerLayer, CorsLayer, CompressionLayer,
        CacheLayer, CacheMiddleware, CacheConfig, CacheLayerBuilder,
    };
    
    pub use oxidite_config::Config;
    
    #[cfg(feature = "database")]
    pub use oxidite_db::{Database, Model, Migration};
    
    #[cfg(feature = "auth")]
    pub use oxidite_auth::{Permission, Role};
    
    #[cfg(feature = "queue")]
    pub use oxidite_queue::{Queue, Job, PostgresBackend};
    
    #[cfg(feature = "cache")]
    pub use oxidite_cache::Cache;
    
    #[cfg(feature = "realtime")]
    pub use oxidite_realtime::WebSocketManager;
    
    #[cfg(feature = "graphql")]
    pub use oxidite_graphql::{GraphQLHandler, GraphQLSchema};
    
    #[cfg(feature = "plugin")]
    pub use oxidite_plugin::{PluginManager, Plugin, PluginInfo};
    
    pub use serde::{Serialize, Deserialize};
}