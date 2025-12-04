use oxidite_core::{OxiditeRequest, OxiditeResponse,  Error as CoreError};
use tower::{Service, Layer};
use std::task::{Context, Poll};
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

const REQUEST_ID_HEADER: &str = "x-request-id";

/// Request ID middleware
#[derive(Clone)]
pub struct RequestIdMiddleware<S> {
    inner: S,
}

impl<S> RequestIdMiddleware<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S> Service<OxiditeRequest> for RequestIdMiddleware<S>
where
    S: Service<OxiditeRequest, Response = OxiditeResponse, Error = CoreError> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: OxiditeRequest) -> Self::Future {
        // Extract or generate request ID
        let request_id = req
            .headers()
            .get(REQUEST_ID_HEADER)
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let mut inner = self.inner.clone();

        Box::pin(async move {
            // TODO: Attach request_id to request extensions
            let mut response = inner.call(req).await?;

            // Add request ID to response headers
            if let Ok(header_value) = request_id.parse() {
                response.headers_mut().insert(REQUEST_ID_HEADER, header_value);
            }

            Ok(response)
        })
    }
}

/// Layer for Request ID middleware
#[derive(Clone, Default)]
pub struct RequestIdLayer;

impl RequestIdLayer {
    pub fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for RequestIdLayer {
    type Service = RequestIdMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequestIdMiddleware::new(inner)
    }
}
