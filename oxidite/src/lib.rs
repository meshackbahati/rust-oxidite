//! # Oxidite Web Framework
//!
//! Oxidite is a modular web framework for Rust built on `hyper` and `tokio`.
//! It provides an integrated stack covering routing, ORM, authentication, real-time
//! communication, background jobs, caching, storage, templates, email, and OpenAPI
//! generation — all wired through a single CLI for scaffolding and development.
//!
//! ## Quick Start
//!
//! ```toml
//! [dependencies]
//! oxidite = { version = "2.3.4", features = ["full"] }
//! tokio = { version = "1", features = ["full"] }
//! serde = { version = "1", features = ["derive"] }
//! ```
//!
//! ```rust,no_run
//! use oxidite::prelude::*;
//!
//! async fn hello(_req: Request) -> Result<Response> {
//!     Ok(Response::text("Hello, Oxidite!"))
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let mut app = Application::new(Config::load().unwrap());
//!     app.router_mut().get("/", hello);
//!     app.run().await
//! }
//! ```
//!
//! ## Features
//!
//! - **HTTP Server**: HTTP/1.1, HTTP/2, and WebSocket support
//! - **Routing**: Path parameters, query parsing, API versioning
//! - **Middleware**: CORS, logging, compression, rate limiting
//! - **Database**: ORM with relationships, migrations, soft deletes, eager loading
//! - **Authentication**: RBAC, JWT, OAuth2, 2FA, API keys
//! - **Background Jobs**: Cron scheduling, retry logic, dead letter queue
//! - **Caching**: Memory and Redis backends
//! - **Real-time**: WebSocket and SSE with event broadcasting
//! - **Templates**: Server-side rendering
//! - **Email**: SMTP support
//! - **File Storage**: Local and S3 backends
//! - **Config**: TOML-based config with namespaced env variables

// Re-export core types
/// Re-export all items from [`oxidite_core`]
pub use oxidite_core::*;
/// Re-export the `extract` module from [`oxidite_core`]
pub use oxidite_core::extract;
/// Re-export [`oxidite_middleware`] as `middleware`
pub use oxidite_middleware as middleware;
/// Re-export [`oxidite_config`] as `config`
pub use oxidite_config as config;
/// Re-export [`OxiditeRequest`] and [`OxiditeResponse`]
pub use oxidite_core::types::{OxiditeRequest, OxiditeResponse};

#[cfg(feature = "database")]
/// Re-export [`oxidite_db`] as `db`
pub use oxidite_db as db;

#[cfg(feature = "auth")]
/// Re-export [`oxidite_auth`] as `auth`
pub use oxidite_auth as auth;

#[cfg(feature = "queue")]
/// Re-export [`oxidite_queue`] as `queue`
pub use oxidite_queue as queue;

#[cfg(feature = "cache")]
/// Re-export [`oxidite_cache`] as `cache`
pub use oxidite_cache as cache;

#[cfg(feature = "realtime")]
/// Re-export [`oxidite_realtime`] as `realtime`
pub use oxidite_realtime as realtime;

#[cfg(feature = "templates")]
/// Re-export [`oxidite_template`] as `template`
pub use oxidite_template as template;

#[cfg(feature = "mail")]
/// Re-export [`oxidite_mail`] as `mail`
pub use oxidite_mail as mail;

#[cfg(feature = "storage")]
/// Re-export [`oxidite_storage`] as `storage`
pub use oxidite_storage as storage;

#[cfg(feature = "security")]
/// Re-export [`oxidite_security`] as `security`
pub use oxidite_security as security;

#[cfg(feature = "utils")]
/// Re-export [`oxidite_utils`] as `utils`
pub use oxidite_utils as utils;

/// Prelude module for common imports
pub mod prelude {
    /// Core framework types
    pub use oxidite_core::{
        Application, Router, Server, Handler, IntoHandler, handler_fn,
        Error, Result,
        Request, Response,
        StatusCode, mpsc, BodyExt,
        CorsConfig,
        extract::{Json, Path, Query, State, FromRequest, Form, Cookies, Body, WebSocketUpgrade, PathParams},
    };
    
    /// Middleware types
    pub use oxidite_middleware::{
        ServiceBuilder, LoggerLayer, CorsLayer, CompressionLayer,
        CacheLayer, CacheMiddleware, CacheConfig, CacheLayerBuilder,
        MetricsLayer,
    };
    
    /// Config type
    pub use oxidite_config::Config;
    
    // Response helpers for cleaner syntax: json!({...}), text("..."), html("...")
    /// Response helper functions
    pub use oxidite_core::response::helpers::{json, text, html, ok, no_content};
    
    #[cfg(feature = "database")]
    /// Database types
    pub use oxidite_db::{Database, Model, Migration, DbPool, DbTransaction, Result as DbResult, OrmError, Pagination, SortDirection};
    
    // Re-export commonly used sqlx items so users don't need to know about sqlx
    #[cfg(feature = "database")]
    /// Commonly used sqlx types
    pub use oxidite_db::sqlx::{
        query, query_as, query_scalar,
        FromRow, Row,
        Encode, Decode, Type,
        AnyPool, Any,
        postgres::{PgPool, PgRow},
        mysql::{MySqlPool, MySqlRow},
        sqlite::{SqlitePool, SqliteRow},
        Column,
        Acquire, Executor,
    };
    
    #[cfg(feature = "auth")]
    /// Auth types
    pub use oxidite_auth::{Permission, Role};
    
    #[cfg(feature = "queue")]
    /// Queue types
    pub use oxidite_queue::{Queue, Job, PostgresBackend};
    
    #[cfg(feature = "cache")]
    /// Cache trait
    pub use oxidite_cache::Cache;
    
    #[cfg(feature = "realtime")]
    /// WebSocket types
    pub use oxidite_realtime::WebSocketManager;
    
    #[cfg(feature = "graphql")]
    /// GraphQL types
    pub use oxidite_graphql::{GraphQLHandler, GraphQLSchema};
    
    #[cfg(feature = "plugin")]
    /// Plugin types
    pub use oxidite_plugin::{PluginManager, Plugin, PluginInfo};
    
    /// Serde derive macros
    pub use serde::{Serialize, Deserialize};
    /// Serde JSON value
    pub use serde_json::json;
    /// Build a CORS layer from config
    pub use crate::config_helper::cors_layer_from_config;
}
/// Configuration helper utilities
pub mod config_helper;
/// Re-export of [`cors_layer_from_config`]
pub use config_helper::cors_layer_from_config;
