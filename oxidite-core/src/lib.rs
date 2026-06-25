/// Application boot coordinator and configuration.
pub mod app;
/// Error types and the `Result` alias.
pub mod error;
/// Request extractors (`Json`, `Path`, `Query`, `State`, `Form`, `Cookies`, `Body`, `WebSocketUpgrade`).
pub mod extract;
/// Extension trait for `Request` with body-reading helpers.
pub mod request;
/// Response constructors (`json`, `text`, `html`, helpers).
pub mod response;
/// HTTP router with path parameters, middleware, and CORS support.
pub mod router;
/// HTTP/1.1 server loop and body adapter.
pub mod server;
/// TLS/HTTPS support and `SecureServer`.
pub mod tls;
/// Type aliases for requests, responses, and boxed bodies.
pub mod types;
/// API versioning with `ApiVersion` and `VersionedRouter`.
pub mod versioning;
/// Backwards-compatible re-exports for cookies and form data.
pub mod cookie;

/// Core error and result types.
pub use error::{Error, Result};
/// Common request extractors.
pub use extract::{FromRequest, Json, Path, Query, State, Form, Cookies, Body, WebSocketUpgrade, PathParams};
/// Re-exports from the `http` crate.
pub use http::{StatusCode, Method, HeaderMap, HeaderValue};
/// Re-export of `tokio::sync::mpsc` for channel-based communication.
pub use tokio::sync::mpsc;
/// Re-export of `http_body_util::BodyExt` for body-reading utilities.
pub use http_body_util::BodyExt;

/// The `Application` type.
pub use app::Application;
/// Router and handler types.
pub use router::{Handler, Router, IntoHandler, handler_fn, CorsConfig};
/// The `Server` type.
pub use server::Server;
/// Core request and response types.
pub use types::{OxiditeRequest, OxiditeResponse};
/// Convenience alias: `OxiditeResponse` as `Response`.
pub use types::OxiditeResponse as Response;
/// Convenience alias: `OxiditeRequest` as `Request`.
pub use types::OxiditeRequest as Request;
/// API versioning types.
pub use versioning::{ApiVersion, VersionedRouter};

/// Re-export response helpers for convenient access.
///
/// These provide cleaner syntax: `json!({...})`, `text("...")`, `html("...")`
pub use response::helpers::{json, text, html};
