use std::task::{Context, Poll};
use tower::{Service, Layer};
use oxidite_core::{OxiditeRequest, OxiditeResponse, Error};
use std::future::Future;
use std::pin::Pin;

#[derive(Clone)]
pub struct Logger<S> {
    inner: S,
}

impl<S> Logger<S> {
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
            if let Ok(ref response) = res {
                println!("Response: {}", response.status());
            }
            res
        })
    }
}

pub struct LoggerLayer;

impl<S> Layer<S> for LoggerLayer {
    type Service = Logger<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Logger::new(inner)
    }
}
