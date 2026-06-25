use std::task::{Context, Poll};
use tower::{Service, Layer};
use oxidite_core::{OxiditeRequest, OxiditeResponse, Error};
use std::future::Future;
use std::pin::Pin;

/// Tower service that logs incoming requests and outgoing responses
#[derive(Clone)]
pub struct Logger<S> {
    inner: S,
}

impl<S> Logger<S> {
    /// Create a new `Logger` wrapping the inner service
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S> Service<OxiditeRequest> for Logger<S>
where
    S: Service<OxiditeRequest, Response = OxiditeResponse, Error = Error> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: OxiditeRequest) -> Self::Future {
        println!("Request: {} {}", req.method(), req.uri());
        let fut = self.inner.call(req);
        Box::pin(async move {
            let res = fut.await;
            match &res {
                Ok(response) => {
                    println!("Response: {}", response.status());
                }
                Err(err) => {
                    println!("Response error: {} {}", err.status_code(), err);
                }
            }
            res
        })
    }
}

/// Tower layer that wraps a service with [`Logger`]
pub struct LoggerLayer;

impl<S> Layer<S> for LoggerLayer {
    type Service = Logger<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Logger::new(inner)
    }
}
