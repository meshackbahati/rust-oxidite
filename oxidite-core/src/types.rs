use http_body_util::combinators::BoxBody as HttpBoxBody;
use bytes::Bytes;
use hyper::{Request, Response};

pub type BoxBody = HttpBoxBody<Bytes, hyper::Error>;
pub type OxiditeBody = HttpBoxBody<Bytes, hyper::Error>;
pub type OxiditeRequest = Request<OxiditeBody>;

pub struct OxiditeResponse(pub Response<BoxBody>);

impl OxiditeResponse {
    pub fn new(response: Response<BoxBody>) -> Self {
        Self(response)
    }

    pub fn into_inner(self) -> Response<BoxBody> {
        self.0
    }
}

impl std::ops::Deref for OxiditeResponse {
    type Target = Response<BoxBody>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for OxiditeResponse {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Response<BoxBody>> for OxiditeResponse {
    fn from(inner: Response<BoxBody>) -> Self {
        Self(inner)
    }
}

impl From<OxiditeResponse> for Response<BoxBody> {
    fn from(wrapper: OxiditeResponse) -> Self {
        wrapper.0
    }
}
