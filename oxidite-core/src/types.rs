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

    /// Convenience method to get the status code (avoids needing a full Into conversion in test code)
    pub fn status(&self) -> http::StatusCode {
        self.0.status()
    }

    /// Convenience method to get a reference to the response headers
    pub fn headers(&self) -> &http::HeaderMap {
        self.0.headers()
    }

    /// Convenience method to get a mutable reference to the response headers
    pub fn headers_mut(&mut self) -> &mut http::HeaderMap {
        self.0.headers_mut()
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

/// Generic conversion from OxiditeResponse to any hyper::Response<B> where B: Default + From<BoxBody>
impl<B: Default + From<BoxBody>> From<OxiditeResponse> for hyper::Response<B> {
    fn from(wrapper: OxiditeResponse) -> Self {
        let (parts, body) = wrapper.0.into_parts();
        Self::from_parts(parts, B::from(body))
    }
}

/// Helper function to create an `Ok(Result<T>)` in closure contexts where
/// the compiler cannot infer the error type parameter `E` on `std::result::Result::Ok`.
///
/// This shadows `std::result::Result::Ok` when `crate::types::*` or `oxidite::prelude::*`
/// is in scope, so you can write `Ok(value)` inside `.map()` closures without a
/// turbofish annotation.
///
/// # Example
///
/// ```rust,ignore
/// use oxidite::prelude::*;
///
/// let parsed: Result<Vec<String>> = items.into_iter().map(|s| {
///     Ok(s.to_string())
/// }).collect();
/// ```
#[inline]
#[allow(non_snake_case)]
pub fn Ok<T>(value: T) -> crate::error::Result<T> {
    std::result::Result::Ok(value)
}
