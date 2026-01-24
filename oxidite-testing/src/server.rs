use oxidite_core::{Router, OxiditeRequest, OxiditeResponse, Result};
use tower::Service;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Test server for integration testing
pub struct TestServer<S> {
    service: S,
}

impl<S> TestServer<S>
where
    S: Service<OxiditeRequest, Response = OxiditeResponse> + Clone + Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    S::Future: Send,
{
    /// Create a new test server from a service
    pub fn new(service: S) -> Self {
        Self { service }
    }

    /// Send a request to the test server
    pub async fn call(&mut self, request: OxiditeRequest) -> Result<OxiditeResponse> {
        use tower::ServiceExt;
        self.service
            .ready()
            .await
            .map_err(|e| oxidite_core::Error::InternalServerError(format!("Service not ready: {:?}", e.into())))?
            .call(request)
            .await
            .map_err(|e| oxidite_core::Error::InternalServerError(format!("Request failed: {:?}", e.into())))
    }
}

/// Helper to test a router
pub fn test_router(router: Router) -> TestServer<Router> {
    TestServer::new(router)
}
