pub mod error;
pub mod extract;
pub mod request;
pub mod response;
pub mod router;
pub mod server;
pub mod types;
pub mod versioning;
pub mod tls;

pub use error::{Error, Result};
pub use extract::{Path, Query, Json, FromRequest, State};
pub use request::RequestExt;
pub use response::{json, html, text};
pub use router::Router;
pub use server::Server;
pub use types::{OxiditeRequest, OxiditeResponse, BoxBody};
pub type Response = OxiditeResponse;
pub type Request = OxiditeRequest;
