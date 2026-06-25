use std::task::{Context, Poll};
use tower::{Service, Layer};
use oxidite_core::{OxiditeRequest, OxiditeResponse, Error};
use oxidite_utils::GLOBAL_METRICS;
use std::future::Future;
use std::pin::Pin;
use std::time::Instant;

/// Tower service wrapping HTTP requests for telemetry metrics logging
#[derive(Clone)]
pub struct Metrics<S> {
    inner: S,
}

impl<S> Metrics<S> {
    /// Create a new `Metrics` service wrapping the inner service
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S> Service<OxiditeRequest> for Metrics<S>
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
        let start = Instant::now();
        GLOBAL_METRICS.increment_concurrent();
        
        let path = req.uri().path().to_string();
        let fut = self.inner.call(req);
        
        Box::pin(async move {
            let res = fut.await;
            GLOBAL_METRICS.decrement_concurrent();
            let duration_ms = start.elapsed().as_millis() as u64;
            
            match &res {
                Ok(response) => {
                    let is_success = response.status().as_u16() < 400;
                    GLOBAL_METRICS.record_request(&path, duration_ms, is_success);
                }
                Err(_) => {
                    GLOBAL_METRICS.record_request(&path, duration_ms, false);
                }
            }
            res
        })
    }
}

/// Tower Layer for injecting the Metrics middleware
pub struct MetricsLayer;

impl<S> Layer<S> for MetricsLayer {
    type Service = Metrics<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Metrics::new(inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn metrics_registry_tracks_requests() {
        GLOBAL_METRICS.record_request("/home_test", 25, true);
        GLOBAL_METRICS.record_request("/home_test", 15, true);
        GLOBAL_METRICS.record_request("/home_test", 50, false);
        
        let snapshot = GLOBAL_METRICS.get_snapshot();
        let (reqs, successes, errors, duration) = snapshot.get("/home_test").unwrap();
        assert_eq!(*reqs, 3);
        assert_eq!(*successes, 2);
        assert_eq!(*errors, 1);
        assert_eq!(*duration, 90);
    }
}
