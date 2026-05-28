pub mod error;
pub mod extract;
pub mod request;
pub mod response;
pub mod router;
pub mod server;
pub mod tls;
pub mod types;
pub mod versioning;
pub mod cookie;

pub use error::{Error, Result};
pub use extract::{FromRequest, Json, Path, Query, State, Form, Cookies, Body, WebSocketUpgrade, PathParams};
pub use http::{StatusCode, Method, HeaderMap, HeaderValue};
pub use tokio::sync::mpsc;
pub use http_body_util::BodyExt;

pub use router::{Handler, Router, IntoHandler, handler_fn, CorsConfig};
pub use server::Server;
pub use types::{OxiditeRequest, OxiditeResponse};
pub use types::OxiditeResponse as Response;
pub use types::OxiditeRequest as Request;
pub use versioning::{ApiVersion, VersionedRouter};

/// Re-export response helpers for convenient access.
///
/// These provide cleaner syntax: `json!({...})`, `text("...")`, `html("...")`
pub use response::helpers::{json, text, html};
