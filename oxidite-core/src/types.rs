use http_body_util::Full;
use bytes::Bytes;
use hyper::{Request, Response, body::Incoming};

pub type BoxBody = Full<Bytes>;
pub type OxiditeRequest = Request<Incoming>;
pub type OxiditeResponse = Response<BoxBody>;
