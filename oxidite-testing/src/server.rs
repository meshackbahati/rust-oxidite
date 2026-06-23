use oxidite_core::{Router, OxiditeRequest, OxiditeResponse, Result};
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use tower::Service;

/// Test server for integration testing.
///
/// This server is **single-threaded by design** — it holds a single `&mut` reference
/// to the underlying service and processes one request at a time.  For concurrent
/// tests, create a separate `TestServer` per test function (the server is cheap to
/// construct).
///
/// # Example
///
/// ```rust,ignore
/// use oxidite_testing::{test_router, TestRequest};
///
/// #[tokio::test]
/// async fn test_hello() {
///     let mut server = test_router(my_router());
///     let req = TestRequest::get("/hello").build_oxidite();
///     let resp = server.call(req).await.unwrap();
///     assert_eq!(resp.status(), 200);
/// }
/// ```
pub struct TestServer<S> {
    service: S,
}

impl<S> Clone for TestServer<S>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
        }
    }
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

    /// Send a request to the test server.
    ///
    /// Because the underlying Tower `Service` requires `&mut self` for `ready()`,
    /// this method takes `&mut self`.  If you need concurrent requests, either
    /// clone the test server or construct a fresh one per concurrent task.
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

    /// Send a plain HTTP test request after converting its body to Oxidite's boxed body.
    pub async fn call_http(&mut self, request: http::Request<Full<Bytes>>) -> Result<OxiditeResponse> {
        let (parts, body) = request.into_parts();
        let request = http::Request::from_parts(parts, body.map_err(|e| match e {}).boxed());
        self.call(request).await
    }
}

/// Helper to test a router
pub fn test_router(router: Router) -> TestServer<Router> {
    TestServer::new(router)
}
