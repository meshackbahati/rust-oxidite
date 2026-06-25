use crate::error::{Error, Result};
use crate::types::{OxiditeRequest, OxiditeResponse};
use crate::extract::FromRequest;
use hyper::Method;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower_service::Service;

use regex::Regex;

/// Trait for type-erased handlers stored in the router
pub trait Endpoint: Send + Sync + 'static {
    fn call(&self, req: OxiditeRequest) -> Pin<Box<dyn Future<Output = Result<OxiditeResponse>> + Send>>;
}

impl Endpoint for Arc<dyn Endpoint> {
    fn call(&self, req: OxiditeRequest) -> Pin<Box<dyn Future<Output = Result<OxiditeResponse>> + Send>> {
        (**self).call(req)
    }
}

/// A `tower::Service` wrapper around a type-erased [`Endpoint`].
///
/// Bridges the `Endpoint` trait with tower's `Service` trait so that
/// handlers can be used with tower middleware and hyper's server stack.
pub struct EndpointService(pub Arc<dyn Endpoint>);

impl Clone for EndpointService {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Service<OxiditeRequest> for EndpointService {
    type Response = OxiditeResponse;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<OxiditeResponse>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<std::result::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: OxiditeRequest) -> Self::Future {
        self.0.call(req)
    }
}

impl Endpoint for EndpointService {
    fn call(&self, req: OxiditeRequest) -> Pin<Box<dyn Future<Output = Result<OxiditeResponse>> + Send>> {
        self.0.call(req)
    }
}

/// CORS configuration for the Router
#[derive(Clone, Debug)]
pub struct CorsConfig {
    /// Allowed origins (e.g., "http://localhost:3000")
    /// Use "*" to allow all origins
    pub allowed_origins: Vec<String>,
    /// Allowed HTTP methods (e.g., "GET", "POST", "PUT", "DELETE")
    /// Empty means allow all methods
    pub allowed_methods: Vec<String>,
    /// Allowed HTTP headers (e.g., "Content-Type", "Authorization")
    /// Empty means allow all headers
    pub allowed_headers: Vec<String>,
    /// Whether to allow credentials (cookies, authorization headers)
    pub allow_credentials: bool,
    /// Max age for CORS preflight cache (in seconds)
    pub max_age: u32,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string(), "OPTIONS".to_string(), "PATCH".to_string()],
            allowed_headers: vec!["*".to_string()],
            allow_credentials: false,
            max_age: 3600,
        }
    }
}

impl CorsConfig {
    /// Create a new CORS config that allows everything (useful for development)
    pub fn permissive() -> Self {
        Self::default()
    }

    /// Create a new CORS config with no allowed origins (restrictive default)
    pub fn restrictive() -> Self {
        Self {
            allowed_origins: Vec::new(),
            allowed_methods: vec!["GET".to_string(), "POST".to_string()],
            allowed_headers: vec!["Content-Type".to_string()],
            allow_credentials: false,
            max_age: 3600,
        }
    }

    /// Builder: add an allowed origin.
    pub fn allow_origin(mut self, origin: &str) -> Self {
        self.allowed_origins.push(origin.to_string());
        self
    }

    /// Builder: add an allowed HTTP method.
    pub fn allow_method(mut self, method: &str) -> Self {
        self.allowed_methods.push(method.to_string());
        self
    }

    /// Builder: add an allowed HTTP header.
    pub fn allow_header(mut self, header: &str) -> Self {
        self.allowed_headers.push(header.to_string());
        self
    }

    /// Builder: enable credentials (cookies, authorization headers).
    pub fn credentials(mut self) -> Self {
        self.allow_credentials = true;
        self
    }

    /// Builder: set the max age for CORS preflight cache (in seconds).
    pub fn max_age(mut self, seconds: u32) -> Self {
        self.max_age = seconds;
        self
    }
}

// Note: tower-http middleware like Cors and Compression change the response body type,
// so they cannot be used as Endpoint-level middleware. They should be applied at the
// Server level instead, after body type conversion.
//
// For CORS at the router level, use custom middleware that only modifies headers
// without changing the body type.


/// Trait for async functions that can be used as handlers
pub trait Handler<Args>: Clone + Send + Sync + 'static {
    fn call(&self, req: OxiditeRequest) -> Pin<Box<dyn Future<Output = Result<OxiditeResponse>> + Send>>;
}

// Wrapper to convert Handler<Args> into Endpoint
struct HandlerService<H, Args> {
    handler: H,
    _marker: std::marker::PhantomData<Args>,
}

impl<H, Args> Endpoint for HandlerService<H, Args>
where
    H: Handler<Args>,
    Args: Send + Sync + 'static,
{
    fn call(&self, req: OxiditeRequest) -> Pin<Box<dyn Future<Output = Result<OxiditeResponse>> + Send>> {
        self.handler.call(req)
    }
}

// Implement Handler for Fn(OxiditeRequest) -> Fut
impl<F, Fut> Handler<OxiditeRequest> for F
where
    F: Fn(OxiditeRequest) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<OxiditeResponse>> + Send + 'static,
{
    fn call(&self, req: OxiditeRequest) -> Pin<Box<dyn Future<Output = Result<OxiditeResponse>> + Send>> {
        let fut = self(req);
        Box::pin(async move { fut.await })
    }
}

// Implement Handler for Fn() -> Fut
impl<F, Fut> Handler<()> for F
where
    F: Fn() -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<OxiditeResponse>> + Send + 'static,
{
    fn call(&self, _req: OxiditeRequest) -> Pin<Box<dyn Future<Output = Result<OxiditeResponse>> + Send>> {
        let fut = self();
        Box::pin(async move { fut.await })
    }
}

// Implement Handler for Fn(T1) -> Fut
impl<F, Fut, T1> Handler<(T1,)> for F
where
    F: Fn(T1) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<OxiditeResponse>> + Send + 'static,
    T1: FromRequest + Send + 'static,
{
    fn call(&self, mut req: OxiditeRequest) -> Pin<Box<dyn Future<Output = Result<OxiditeResponse>> + Send>> {
        let handler = self.clone();
        Box::pin(async move {
            let t1 = T1::from_request(&mut req).await?;
            handler(t1).await
        })
    }
}

// Implement Handler for Fn(T1, T2, ..., T9) -> Fut
impl<F, Fut, T1, T2, T3, T4, T5, T6, T7, T8, T9> Handler<(T1, T2, T3, T4, T5, T6, T7, T8, T9)> for F
where
    F: Fn(T1, T2, T3, T4, T5, T6, T7, T8, T9) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<OxiditeResponse>> + Send + 'static,
    T1: FromRequest + Send + 'static,
    T2: FromRequest + Send + 'static,
    T3: FromRequest + Send + 'static,
    T4: FromRequest + Send + 'static,
    T5: FromRequest + Send + 'static,
    T6: FromRequest + Send + 'static,
    T7: FromRequest + Send + 'static,
    T8: FromRequest + Send + 'static,
    T9: FromRequest + Send + 'static,
{
    fn call(&self, mut req: OxiditeRequest) -> Pin<Box<dyn Future<Output = Result<OxiditeResponse>> + Send>> {
        let handler = self.clone();
        Box::pin(async move {
            let t1 = T1::from_request(&mut req).await?;
            let t2 = T2::from_request(&mut req).await?;
            let t3 = T3::from_request(&mut req).await?;
            let t4 = T4::from_request(&mut req).await?;
            let t5 = T5::from_request(&mut req).await?;
            let t6 = T6::from_request(&mut req).await?;
            let t7 = T7::from_request(&mut req).await?;
            let t8 = T8::from_request(&mut req).await?;
            let t9 = T9::from_request(&mut req).await?;
            handler(t1, t2, t3, t4, t5, t6, t7, t8, t9).await
        })
    }
}

// Implement Handler for Fn(T1, T2, ..., T10) -> Fut
impl<F, Fut, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10> Handler<(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10)> for F
where
    F: Fn(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<OxiditeResponse>> + Send + 'static,
    T1: FromRequest + Send + 'static,
    T2: FromRequest + Send + 'static,
    T3: FromRequest + Send + 'static,
    T4: FromRequest + Send + 'static,
    T5: FromRequest + Send + 'static,
    T6: FromRequest + Send + 'static,
    T7: FromRequest + Send + 'static,
    T8: FromRequest + Send + 'static,
    T9: FromRequest + Send + 'static,
    T10: FromRequest + Send + 'static,
{
    fn call(&self, mut req: OxiditeRequest) -> Pin<Box<dyn Future<Output = Result<OxiditeResponse>> + Send>> {
        let handler = self.clone();
        Box::pin(async move {
            let t1 = T1::from_request(&mut req).await?;
            let t2 = T2::from_request(&mut req).await?;
            let t3 = T3::from_request(&mut req).await?;
            let t4 = T4::from_request(&mut req).await?;
            let t5 = T5::from_request(&mut req).await?;
            let t6 = T6::from_request(&mut req).await?;
            let t7 = T7::from_request(&mut req).await?;
            let t8 = T8::from_request(&mut req).await?;
            let t9 = T9::from_request(&mut req).await?;
            let t10 = T10::from_request(&mut req).await?;
            handler(t1, t2, t3, t4, t5, t6, t7, t8, t9, t10).await
        })
    }
}

// Implement Handler for Fn(T1, T2, ..., T11) -> Fut
impl<F, Fut, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11> Handler<(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11)> for F
where
    F: Fn(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<OxiditeResponse>> + Send + 'static,
    T1: FromRequest + Send + 'static,
    T2: FromRequest + Send + 'static,
    T3: FromRequest + Send + 'static,
    T4: FromRequest + Send + 'static,
    T5: FromRequest + Send + 'static,
    T6: FromRequest + Send + 'static,
    T7: FromRequest + Send + 'static,
    T8: FromRequest + Send + 'static,
    T9: FromRequest + Send + 'static,
    T10: FromRequest + Send + 'static,
    T11: FromRequest + Send + 'static,
{
    fn call(&self, mut req: OxiditeRequest) -> Pin<Box<dyn Future<Output = Result<OxiditeResponse>> + Send>> {
        let handler = self.clone();
        Box::pin(async move {
            let t1 = T1::from_request(&mut req).await?;
            let t2 = T2::from_request(&mut req).await?;
            let t3 = T3::from_request(&mut req).await?;
            let t4 = T4::from_request(&mut req).await?;
            let t5 = T5::from_request(&mut req).await?;
            let t6 = T6::from_request(&mut req).await?;
            let t7 = T7::from_request(&mut req).await?;
            let t8 = T8::from_request(&mut req).await?;
            let t9 = T9::from_request(&mut req).await?;
            let t10 = T10::from_request(&mut req).await?;
            let t11 = T11::from_request(&mut req).await?;
            handler(t1, t2, t3, t4, t5, t6, t7, t8, t9, t10, t11).await
        })
    }
}

// Implement Handler for Fn(T1, T2, ..., T12) -> Fut
impl<F, Fut, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12> Handler<(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12)> for F
where
    F: Fn(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<OxiditeResponse>> + Send + 'static,
    T1: FromRequest + Send + 'static,
    T2: FromRequest + Send + 'static,
    T3: FromRequest + Send + 'static,
    T4: FromRequest + Send + 'static,
    T5: FromRequest + Send + 'static,
    T6: FromRequest + Send + 'static,
    T7: FromRequest + Send + 'static,
    T8: FromRequest + Send + 'static,
    T9: FromRequest + Send + 'static,
    T10: FromRequest + Send + 'static,
    T11: FromRequest + Send + 'static,
    T12: FromRequest + Send + 'static,
{
    fn call(&self, mut req: OxiditeRequest) -> Pin<Box<dyn Future<Output = Result<OxiditeResponse>> + Send>> {
        let handler = self.clone();
        Box::pin(async move {
            let t1 = T1::from_request(&mut req).await?;
            let t2 = T2::from_request(&mut req).await?;
            let t3 = T3::from_request(&mut req).await?;
            let t4 = T4::from_request(&mut req).await?;
            let t5 = T5::from_request(&mut req).await?;
            let t6 = T6::from_request(&mut req).await?;
            let t7 = T7::from_request(&mut req).await?;
            let t8 = T8::from_request(&mut req).await?;
            let t9 = T9::from_request(&mut req).await?;
            let t10 = T10::from_request(&mut req).await?;
            let t11 = T11::from_request(&mut req).await?;
            let t12 = T12::from_request(&mut req).await?;
            handler(t1, t2, t3, t4, t5, t6, t7, t8, t9, t10, t11, t12).await
        })
    }
}

// Implement Handler for Fn(T1, T2) -> Fut
impl<F, Fut, T1, T2> Handler<(T1, T2)> for F
where
    F: Fn(T1, T2) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<OxiditeResponse>> + Send + 'static,
    T1: FromRequest + Send + 'static,
    T2: FromRequest + Send + 'static,
{
    fn call(&self, mut req: OxiditeRequest) -> Pin<Box<dyn Future<Output = Result<OxiditeResponse>> + Send>> {
        let handler = self.clone();
        Box::pin(async move {
            let t1 = T1::from_request(&mut req).await?;
            let t2 = T2::from_request(&mut req).await?;
            handler(t1, t2).await
        })
    }
}

// Implement Handler for Fn(T1, T2, T3) -> Fut
impl<F, Fut, T1, T2, T3> Handler<(T1, T2, T3)> for F
where
    F: Fn(T1, T2, T3) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<OxiditeResponse>> + Send + 'static,
    T1: FromRequest + Send + 'static,
    T2: FromRequest + Send + 'static,
    T3: FromRequest + Send + 'static,
{
    fn call(&self, mut req: OxiditeRequest) -> Pin<Box<dyn Future<Output = Result<OxiditeResponse>> + Send>> {
        let handler = self.clone();
        Box::pin(async move {
            let t1 = T1::from_request(&mut req).await?;
            let t2 = T2::from_request(&mut req).await?;
            let t3 = T3::from_request(&mut req).await?;
            handler(t1, t2, t3).await
        })
    }
}

// Implement Handler for Fn(T1, T2, T3, T4) -> Fut
impl<F, Fut, T1, T2, T3, T4> Handler<(T1, T2, T3, T4)> for F
where
    F: Fn(T1, T2, T3, T4) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<OxiditeResponse>> + Send + 'static,
    T1: FromRequest + Send + 'static,
    T2: FromRequest + Send + 'static,
    T3: FromRequest + Send + 'static,
    T4: FromRequest + Send + 'static,
{
    fn call(&self, mut req: OxiditeRequest) -> Pin<Box<dyn Future<Output = Result<OxiditeResponse>> + Send>> {
        let handler = self.clone();
        Box::pin(async move {
            let t1 = T1::from_request(&mut req).await?;
            let t2 = T2::from_request(&mut req).await?;
            let t3 = T3::from_request(&mut req).await?;
            let t4 = T4::from_request(&mut req).await?;
            handler(t1, t2, t3, t4).await
        })
    }
}

// Implement Handler for Fn(T1, T2, T3, T4, T5) -> Fut
impl<F, Fut, T1, T2, T3, T4, T5> Handler<(T1, T2, T3, T4, T5)> for F
where
    F: Fn(T1, T2, T3, T4, T5) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<OxiditeResponse>> + Send + 'static,
    T1: FromRequest + Send + 'static,
    T2: FromRequest + Send + 'static,
    T3: FromRequest + Send + 'static,
    T4: FromRequest + Send + 'static,
    T5: FromRequest + Send + 'static,
{
    fn call(&self, mut req: OxiditeRequest) -> Pin<Box<dyn Future<Output = Result<OxiditeResponse>> + Send>> {
        let handler = self.clone();
        Box::pin(async move {
            let t1 = T1::from_request(&mut req).await?;
            let t2 = T2::from_request(&mut req).await?;
            let t3 = T3::from_request(&mut req).await?;
            let t4 = T4::from_request(&mut req).await?;
            let t5 = T5::from_request(&mut req).await?;
            handler(t1, t2, t3, t4, t5).await
        })
    }
}

// Implement Handler for Fn(T1, T2, T3, T4, T5, T6) -> Fut
impl<F, Fut, T1, T2, T3, T4, T5, T6> Handler<(T1, T2, T3, T4, T5, T6)> for F
where
    F: Fn(T1, T2, T3, T4, T5, T6) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<OxiditeResponse>> + Send + 'static,
    T1: FromRequest + Send + 'static,
    T2: FromRequest + Send + 'static,
    T3: FromRequest + Send + 'static,
    T4: FromRequest + Send + 'static,
    T5: FromRequest + Send + 'static,
    T6: FromRequest + Send + 'static,
{
    fn call(&self, mut req: OxiditeRequest) -> Pin<Box<dyn Future<Output = Result<OxiditeResponse>> + Send>> {
        let handler = self.clone();
        Box::pin(async move {
            let t1 = T1::from_request(&mut req).await?;
            let t2 = T2::from_request(&mut req).await?;
            let t3 = T3::from_request(&mut req).await?;
            let t4 = T4::from_request(&mut req).await?;
            let t5 = T5::from_request(&mut req).await?;
            let t6 = T6::from_request(&mut req).await?;
            handler(t1, t2, t3, t4, t5, t6).await
        })
    }
}

// Implement Handler for Fn(T1, T2, T3, T4, T5, T6, T7) -> Fut
impl<F, Fut, T1, T2, T3, T4, T5, T6, T7> Handler<(T1, T2, T3, T4, T5, T6, T7)> for F
where
    F: Fn(T1, T2, T3, T4, T5, T6, T7) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<OxiditeResponse>> + Send + 'static,
    T1: FromRequest + Send + 'static,
    T2: FromRequest + Send + 'static,
    T3: FromRequest + Send + 'static,
    T4: FromRequest + Send + 'static,
    T5: FromRequest + Send + 'static,
    T6: FromRequest + Send + 'static,
    T7: FromRequest + Send + 'static,
{
    fn call(&self, mut req: OxiditeRequest) -> Pin<Box<dyn Future<Output = Result<OxiditeResponse>> + Send>> {
        let handler = self.clone();
        Box::pin(async move {
            let t1 = T1::from_request(&mut req).await?;
            let t2 = T2::from_request(&mut req).await?;
            let t3 = T3::from_request(&mut req).await?;
            let t4 = T4::from_request(&mut req).await?;
            let t5 = T5::from_request(&mut req).await?;
            let t6 = T6::from_request(&mut req).await?;
            let t7 = T7::from_request(&mut req).await?;
            handler(t1, t2, t3, t4, t5, t6, t7).await
        })
    }
}

// Implement Handler for Fn(T1, T2, T3, T4, T5, T6, T7, T8) -> Fut
impl<F, Fut, T1, T2, T3, T4, T5, T6, T7, T8> Handler<(T1, T2, T3, T4, T5, T6, T7, T8)> for F
where
    F: Fn(T1, T2, T3, T4, T5, T6, T7, T8) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<OxiditeResponse>> + Send + 'static,
    T1: FromRequest + Send + 'static,
    T2: FromRequest + Send + 'static,
    T3: FromRequest + Send + 'static,
    T4: FromRequest + Send + 'static,
    T5: FromRequest + Send + 'static,
    T6: FromRequest + Send + 'static,
    T7: FromRequest + Send + 'static,
    T8: FromRequest + Send + 'static,
{
    fn call(&self, mut req: OxiditeRequest) -> Pin<Box<dyn Future<Output = Result<OxiditeResponse>> + Send>> {
        let handler = self.clone();
        Box::pin(async move {
            let t1 = T1::from_request(&mut req).await?;
            let t2 = T2::from_request(&mut req).await?;
            let t3 = T3::from_request(&mut req).await?;
            let t4 = T4::from_request(&mut req).await?;
            let t5 = T5::from_request(&mut req).await?;
            let t6 = T6::from_request(&mut req).await?;
            let t7 = T7::from_request(&mut req).await?;
            let t8 = T8::from_request(&mut req).await?;
            handler(t1, t2, t3, t4, t5, t6, t7, t8).await
        })
    }
}

struct Route {
    pattern: Regex,
    param_names: Vec<String>,
    handler: Arc<dyn Endpoint>,
}

/// The central request router.
///
/// Maintains a map of HTTP methods to registered route patterns and their
/// associated handler endpoints. Supports path parameters (`/users/:id`),
/// wildcard segments, middleware layers, and CORS configuration.
///
/// Implements `tower::Service` so it can be used directly with hyper's
/// server or wrapped with tower middleware.
#[derive(Clone)]
pub struct Router {
    routes: HashMap<Method, Vec<Arc<Route>>>,
    extensions: Arc<std::sync::RwLock<http::Extensions>>,
    middleware: Vec<Arc<dyn Fn(Arc<dyn Endpoint>) -> Arc<dyn Endpoint> + Send + Sync>>,
    cors_config: Option<CorsConfig>,
}

impl Router {
    /// Create an empty router with no routes, middleware, or CORS configuration.
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            extensions: Arc::new(std::sync::RwLock::new(http::Extensions::new())),
            middleware: Vec::new(),
            cors_config: None,
        }
    }

    /// Add a shared state to the router that will be available in all handlers
    pub fn with_state<T: Clone + Send + Sync + 'static>(&mut self, state: T) {
        if let Ok(mut exts) = self.extensions.write() {
            exts.insert(state);
        }
    }

    /// Register a handler for `GET` requests at the given path.
    pub fn get<H, Args>(&mut self, path: &str, handler: H)
    where
        H: Handler<Args>,
        Args: Send + Sync + 'static,
    {
        self.add_route(Method::GET, path, handler);
    }
    
    /// Register a handler for `POST` requests at the given path.
    pub fn post<H, Args>(&mut self, path: &str, handler: H)
    where
        H: Handler<Args>,
        Args: Send + Sync + 'static,
    {
        self.add_route(Method::POST, path, handler);
    }

    /// Register a handler for `PUT` requests at the given path.
    pub fn put<H, Args>(&mut self, path: &str, handler: H)
    where
        H: Handler<Args>,
        Args: Send + Sync + 'static,
    {
        self.add_route(Method::PUT, path, handler);
    }

    /// Register a handler for `DELETE` requests at the given path.
    pub fn delete<H, Args>(&mut self, path: &str, handler: H)
    where
        H: Handler<Args>,
        Args: Send + Sync + 'static,
    {
        self.add_route(Method::DELETE, path, handler);
    }

    /// Register a handler for `PATCH` requests at the given path.
    pub fn patch<H, Args>(&mut self, path: &str, handler: H)
    where
        H: Handler<Args>,
        Args: Send + Sync + 'static,
    {
        self.add_route(Method::PATCH, path, handler);
    }

    /// Add a middleware layer to all routes in the router.
    ///
    /// The layer must implement `tower::Layer<EndpointService>` and return a new `Endpoint`.
    ///
    /// # Limitations
    ///
    /// Body-type-changing middleware (like `CorsLayer` and `CompressionLayer` from tower-http)
    /// **cannot** be used with this method. These middleware change the HTTP response body type,
    /// which is incompatible with the `Endpoint` trait that expects `OxiditeResponse`.
    ///
    /// For such middleware, use `ServiceBuilder` instead:
    /// ```rust,ignore
    /// let service = ServiceBuilder::new()
    ///     .layer(CorsLayer::permissive())
    ///     .layer(CompressionLayer::new())
    ///     .service(router);
    /// ```
    pub fn layer<L>(mut self, layer: L) -> Self
    where
        L: tower::Layer<EndpointService> + Send + Sync + 'static,
        L::Service: Endpoint,
    {
        let layer = Arc::new(layer);
        self.middleware.push(Arc::new(move |endpoint| {
            Arc::new(layer.layer(EndpointService(endpoint)))
        }));
        self
    }

    /// Alias for `.layer()`.
    pub fn with_layer<L>(self, layer: L) -> Self
    where
        L: tower::Layer<EndpointService> + Send + Sync + 'static,
        L::Service: Endpoint,
    {
        self.layer(layer)
    }

    /// Configure CORS for this router.
    ///
    /// This adds CORS headers to all responses, including preflight OPTIONS requests.
    /// This is a framework-level CORS implementation that doesn't require tower-http.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use oxidite::prelude::*;
    ///
    /// let router = Router::new()
    ///     .with_cors(CorsConfig {
    ///         allowed_origins: vec!["http://localhost:3000".to_string()],
    ///         allowed_methods: vec!["GET".to_string(), "POST".to_string()],
    ///         allowed_headers: vec!["Content-Type".to_string(), "Authorization".to_string()],
    ///         allow_credentials: true,
    ///         max_age: 3600,
    ///     });
    /// ```
    ///
    /// For development, you can use the permissive config:
    ///
    /// ```rust,ignore
    /// let router = Router::new()
    ///     .with_cors(CorsConfig::permissive());
    /// ```
    pub fn with_cors(mut self, config: CorsConfig) -> Self {
        self.cors_config = Some(config);
        self
    }

    /// Internal: compile the path pattern, wrap the handler, apply router-level
    /// middleware, and store the resulting route.
    fn add_route<H, Args>(&mut self, method: Method, path: &str, handler: H)
    where
        H: Handler<Args>,
        Args: Send + Sync + 'static,
    {
        let (pattern, param_names) = compile_path(path);
        let mut endpoint: Arc<dyn Endpoint> = Arc::new(HandlerService {
            handler,
            _marker: std::marker::PhantomData,
        });

        // Apply router-level middleware in reverse order (so the first added is the outermost)
        for mw in self.middleware.iter().rev() {
            endpoint = mw(endpoint);
        }
        
        let route = Arc::new(Route {
            pattern,
            param_names,
            handler: endpoint,
        });
        
        self.routes
            .entry(method)
            .or_insert_with(Vec::new)
            .push(route);
    }

    /// Add CORS headers to a response builder based on the configured CORS policy
    fn add_cors_headers(&self, mut builder: http::response::Builder) -> http::response::Builder {
        if let Some(cors) = &self.cors_config {
            // Add Access-Control-Allow-Origin
            if !cors.allowed_origins.is_empty() {
                if cors.allowed_origins.contains(&"*".to_string()) {
                    builder = builder.header("Access-Control-Allow-Origin", "*");
                } else {
                    // For specific origins, we'd need to check the Origin header
                    // For now, we'll add the first origin (could be improved with request checking)
                    if let Some(origin) = cors.allowed_origins.first() {
                        builder = builder.header("Access-Control-Allow-Origin", origin);
                    }
                }
            }

            // Add Access-Control-Allow-Methods
            if !cors.allowed_methods.is_empty() {
                let methods = cors.allowed_methods.join(", ");
                builder = builder.header("Access-Control-Allow-Methods", methods);
            }

            // Add Access-Control-Allow-Headers
            if !cors.allowed_headers.is_empty() {
                let headers = cors.allowed_headers.join(", ");
                builder = builder.header("Access-Control-Allow-Headers", headers);
            }

            // Add Access-Control-Allow-Credentials
            if cors.allow_credentials {
                builder = builder.header("Access-Control-Allow-Credentials", "true");
            }

            // Add Access-Control-Max-Age
            builder = builder.header("Access-Control-Max-Age", cors.max_age.to_string());
        }
        builder
    }

    /// Match the incoming request against registered routes and dispatch
    /// to the corresponding handler.
    ///
    /// Handles route matching with path parameter extraction, CORS preflight
    /// (OPTIONS), HEAD-to-GET fallback, and produces `405 Method Not Allowed`
    /// or `404 Not Found` errors as appropriate.
    pub async fn handle(&self, mut req: OxiditeRequest) -> Result<OxiditeResponse> {
        req.extensions_mut().insert(self.extensions.clone());
        let method = req.method().clone();
        let path = req.uri().path().to_string();

        // Helper to try matching routes for a specific method
        let try_match = |target_method: &Method, req: &mut OxiditeRequest| -> Option<Arc<Route>> {
            if let Some(routes) = self.routes.get(target_method) {
                for route in routes {
                    if let Some(captures) = route.pattern.captures(&path) {
                        // Extract path parameters
                        let mut params = serde_json::Map::new();
                        for (i, name) in route.param_names.iter().enumerate() {
                            if let Some(value) = captures.get(i + 1) {
                                params.insert(
                                    name.clone(),
                                    serde_json::Value::String(value.as_str().to_string()),
                                );
                            }
                        }

                        // Store params in request extensions
                        if !params.is_empty() {
                            req.extensions_mut().insert(crate::extract::PathParams(
                                serde_json::Value::Object(params),
                            ));
                        }
                        
                        return Some(route.clone());
                    }
                }
            }
            None
        };

        // 1. Try exact method match
        if let Some(route) = try_match(&method, &mut req) {
            // Add router extensions to request so State extractor can find global state
            req.extensions_mut().insert(self.extensions.clone());
            let response = route.handler.call(req).await?;
            
            // Add CORS headers to successful responses
            if self.cors_config.is_some() {
                let hyper_response: hyper::Response<crate::types::BoxBody> = response.into();
                let (parts, body) = hyper_response.into_parts();
                let mut builder = http::Response::builder()
                    .status(parts.status);
                
                // Copy existing headers
                for (key, value) in parts.headers {
                    if let Some(key) = key {
                        builder = builder.header(key, value);
                    }
                }
                
                // Add CORS headers
                builder = self.add_cors_headers(builder);
                
                return Ok(OxiditeResponse::new(builder.body(body).unwrap()));
            }
            
            return Ok(response);
        }

        // 2. If OPTIONS, return empty success response for CORS if no explicit handler
        if method == Method::OPTIONS {
            if let Some(_route) = try_match(&Method::OPTIONS, &mut req) {
                // Explicit handler exists, will be handled by step 1
            } else {
                // Return 204 No Content for CORS preflight
                let mut builder = http::Response::builder()
                    .status(http::StatusCode::NO_CONTENT);
                
                // Add CORS headers to preflight response
                builder = self.add_cors_headers(builder);
                
                return Ok(OxiditeResponse::new(builder
                    .body(crate::types::BoxBody::default())
                    .unwrap()));
            }
        }

        // 3. If HEAD, try GET
        if method == Method::HEAD {
            if let Some(route) = try_match(&Method::GET, &mut req) {
                // Add router extensions to request so State extractor can find global state
                req.extensions_mut().insert(self.extensions.clone());
                // For HEAD requests, we execute the GET handler but the server/hyper 
                // will strip the body automatically since it's a HEAD response.
                return route.handler.call(req).await;
            }
        }

        // 3. Path exists for other methods => method not allowed
        let allowed_methods: Vec<String> = self
            .routes
            .iter()
            .filter(|(route_method, _)| **route_method != method)
            .filter_map(|(route_method, routes)| {
                if routes.iter().any(|route| route.pattern.is_match(&path)) {
                    Some(route_method.as_str().to_string())
                } else {
                    None
                }
            })
            .collect();
        if !allowed_methods.is_empty() {
            return Err(Error::MethodNotAllowed(format!(
                "{} {} (allowed: {})",
                method,
                path,
                allowed_methods.join(", ")
            )));
        }

        Err(Error::NotFound("Route not found".to_string()))
    }
}

impl Service<OxiditeRequest> for Router {
    type Response = OxiditeResponse;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<std::result::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: OxiditeRequest) -> Self::Future {
        let router = self.clone();
        Box::pin(async move {
            router.handle(req).await
        })
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

/// Compile a route path pattern into a regex
/// Converts `/users/:id` to `^/users/([^/]+)$` and returns param names
fn compile_path(path: &str) -> (Regex, Vec<String>) {
    let mut pattern = String::from("^");
    let mut param_names = Vec::new();
    let mut chars = path.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            ':' => {
                // Extract parameter name
                let mut param_name = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch.is_alphanumeric() || next_ch == '_' {
                        param_name.push(next_ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                param_names.push(param_name);
                pattern.push_str("([^/]+)");
            }
            '*' => {
                // Wildcard
                pattern.push_str("(.*)");
            }
            '.' | '+' | '?' | '^' | '$' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | '\\' => {
                // Escape regex special characters
                pattern.push('\\');
                pattern.push(ch);
            }
            _ => {
                pattern.push(ch);
            }
        }
    }

    pattern.push('$');
    let regex = Regex::new(&pattern).expect("Invalid route pattern");
    (regex, param_names)
}

/// Trait that provides compile-time verification that a function is a valid handler.
///
/// This is used by the [`handler_fn`] function to give clear, readable error messages
/// when a function doesn't satisfy the handler requirements, rather than the cryptic
/// trait-bound errors that would otherwise surface from the router.
///
/// # Example
///
/// ```rust,ignore
/// use oxidite::prelude::*;
///
/// // This compiles because the function matches Handler<(State<Arc<AppState>>,)>
/// let h = handler_fn(my_handler);
/// router.get("/users", h);
///
/// // This would fail at compile time with a clear error if the function
/// // has extractors that don't implement FromRequest.
/// ```
pub trait IntoHandler<Args>: Handler<Args> + Sized {
    fn into_handler(self) -> Self {
        self
    }
}

impl<H, Args> IntoHandler<Args> for H where H: Handler<Args> {}

/// Compile-time handler verification helper.
///
/// Wraps a handler function and ensures at compile time that all its extractors
/// implement `FromRequest`. Provides clearer error messages than raw trait bounds.
///
/// # Example
///
/// ```rust,ignore
/// use oxidite::prelude::*;
///
/// async fn index(State(s): State<Arc<AppState>>) -> Result<OxiditeResponse> {
///     Ok(response::json(serde_json::json!({"ok": true})))
/// }
///
/// // Verified at compile time:
/// router.get("/", handler_fn(index));
/// ```
pub fn handler_fn<H, Args>(handler: H) -> H
where
    H: IntoHandler<Args>,
    Args: Send + Sync + 'static,
{
    handler
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BoxBody;

    #[test]
    fn test_compile_path() {
        let (regex, params) = compile_path("/users/:id");
        assert_eq!(params, vec!["id"]);
        assert!(regex.is_match("/users/123"));
        assert!(!regex.is_match("/users/123/posts"));

        let (regex, params) = compile_path("/users/:user_id/posts/:post_id");
        assert_eq!(params, vec!["user_id", "post_id"]);
        assert!(regex.is_match("/users/1/posts/2"));
    }

    #[test]
    fn test_exact_match() {
        let (regex, params) = compile_path("/users");
        assert_eq!(params.len(), 0);
        assert!(regex.is_match("/users"));
        assert!(!regex.is_match("/users/123"));
    }

    #[tokio::test]
    async fn test_handler_with_12_extractors() {
        use crate::extract::State;

        #[derive(Clone)]
        struct AppState;

        async fn h12(
            _s1: State<AppState>,
            _s2: State<AppState>,
            _s3: State<AppState>,
            _s4: State<AppState>,
            _s5: State<AppState>,
            _s6: State<AppState>,
            _s7: State<AppState>,
            _s8: State<AppState>,
            _s9: State<AppState>,
            _s10: State<AppState>,
            _s11: State<AppState>,
            _s12: State<AppState>,
        ) -> Result<OxiditeResponse> {
            Ok(OxiditeResponse::text("ok"))
        }

        let mut router = Router::new();
        router.with_state(AppState);
        router.get("/", h12);

        let req = http::Request::builder()
            .method(Method::GET)
            .uri("/")
            .body(BoxBody::default())
            .expect("request");

        let result = router.handle(req).await.expect("handle");
        assert_eq!(result.status(), http::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_method_not_allowed_when_path_exists() {
        let mut router = Router::new();
        router.get("/users", || async { Ok(crate::OxiditeResponse::text("ok")) });
        let req = http::Request::builder()
            .method(Method::POST)
            .uri("/users")
            .body(BoxBody::default())
            .expect("request");

        let result = router.handle(req).await;
        assert!(matches!(result, Err(Error::MethodNotAllowed(_))));
    }

    #[tokio::test]
    async fn test_not_found_when_path_missing() {
        let router = Router::new();
        let req = http::Request::builder()
            .method(Method::GET)
            .uri("/missing")
            .body(BoxBody::default())
            .expect("request");

        let result = router.handle(req).await;
        assert!(matches!(result, Err(Error::NotFound(_))));
    }

    #[tokio::test]
    async fn test_handler_with_8_extractors() {
        use crate::extract::State;

        #[derive(Clone)]
        struct AppState;

        async fn h8(
            _s1: State<AppState>,
            _s2: State<AppState>,
            _s3: State<AppState>,
            _s4: State<AppState>,
            _s5: State<AppState>,
            _s6: State<AppState>,
            _s7: State<AppState>,
            _s8: State<AppState>,
        ) -> Result<OxiditeResponse> {
            Ok(OxiditeResponse::text("ok"))
        }

        let mut router = Router::new();
        router.with_state(AppState);
        router.get("/", h8);

        let req = http::Request::builder()
            .method(Method::GET)
            .uri("/")
            .body(BoxBody::default())
            .expect("request");

        let result = router.handle(req).await.expect("handle");
        assert_eq!(result.status(), http::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_router_layer() {
        use tower::Layer;

        struct MyMiddleware<S>(S);
        impl<S: Endpoint> Endpoint for MyMiddleware<S> {
            fn call(&self, mut req: OxiditeRequest) -> Pin<Box<dyn Future<Output = Result<OxiditeResponse>> + Send>> {
                req.extensions_mut().insert("middleware_called".to_string());
                self.0.call(req)
            }
        }

        struct MyLayer;
        impl<S> Layer<S> for MyLayer {
            type Service = MyMiddleware<S>;
            fn layer(&self, inner: S) -> Self::Service {
                MyMiddleware(inner)
            }
        }

        // We need to check if extensions were modified.
        // But the handler in this test doesn't check it.
        // Let's modify the handler to check.

        let mut router = Router::new()
            .layer(MyLayer);
        router.get("/", |req: OxiditeRequest| async move {
            if req.extensions().get::<String>().map(|s| s == "middleware_called").unwrap_or(false) {
                Ok(OxiditeResponse::text("middleware_ok"))
            } else {
                Ok(OxiditeResponse::text("middleware_fail"))
            }
        });

        let req = http::Request::builder()
            .method(Method::GET)
            .uri("/")
            .body(BoxBody::default())
            .expect("request");

        let res = router.handle(req).await.expect("handle");
        let hyper_res: hyper::Response<crate::types::BoxBody> = res.into();
        let body = hyper_res.into_body();
        use http_body_util::BodyExt;
        let bytes = body.collect().await.unwrap().to_bytes();
        assert_eq!(bytes, "middleware_ok");
    }

    #[tokio::test]
    async fn test_cors_config_default() {
        let config = CorsConfig::default();
        assert_eq!(config.allowed_origins, vec!["*"]);
        assert!(!config.allowed_methods.is_empty());
        assert_eq!(config.allowed_headers, vec!["*"]);
        assert!(!config.allow_credentials);
        assert_eq!(config.max_age, 3600);
    }

    #[tokio::test]
    async fn test_cors_config_permissive() {
        let config = CorsConfig::permissive();
        assert_eq!(config.allowed_origins, vec!["*"]);
        assert_eq!(config.allowed_headers, vec!["*"]);
    }

    #[tokio::test]
    async fn test_cors_config_restrictive() {
        let config = CorsConfig::restrictive();
        assert!(config.allowed_origins.is_empty());
        assert_eq!(config.allowed_methods, vec!["GET", "POST"]);
        assert_eq!(config.allowed_headers, vec!["Content-Type"]);
    }

    #[tokio::test]
    async fn test_cors_preflight_response() {
        let mut router = Router::new()
            .with_cors(CorsConfig {
                allowed_origins: vec!["http://localhost:3000".to_string()],
                allowed_methods: vec!["GET".to_string(), "POST".to_string()],
                allowed_headers: vec!["Content-Type".to_string()],
                allow_credentials: true,
                max_age: 7200,
            });
        router.get("/test", || async { Ok(OxiditeResponse::text("ok")) });

        let req = http::Request::builder()
            .method(Method::OPTIONS)
            .uri("/test")
            .body(BoxBody::default())
            .expect("request");

        let res = router.handle(req).await.expect("handle");
        let hyper_res: hyper::Response<crate::types::BoxBody> = res.into();
        
        // Should be 204 No Content for preflight
        assert_eq!(hyper_res.status(), http::StatusCode::NO_CONTENT);
        
        // Check CORS headers
        let headers = hyper_res.headers();
        assert_eq!(
            headers.get("Access-Control-Allow-Origin").unwrap(),
            "http://localhost:3000"
        );
        assert_eq!(
            headers.get("Access-Control-Allow-Methods").unwrap(),
            "GET, POST"
        );
        assert_eq!(
            headers.get("Access-Control-Allow-Headers").unwrap(),
            "Content-Type"
        );
        assert_eq!(
            headers.get("Access-Control-Allow-Credentials").unwrap(),
            "true"
        );
        assert_eq!(
            headers.get("Access-Control-Max-Age").unwrap(),
            "7200"
        );
    }

    #[tokio::test]
    async fn test_cors_on_successful_response() {
        let mut router = Router::new()
            .with_cors(CorsConfig::permissive());
        router.get("/test", || async { Ok(OxiditeResponse::text("ok")) });

        let req = http::Request::builder()
            .method(Method::GET)
            .uri("/test")
            .body(BoxBody::default())
            .expect("request");

        let res = router.handle(req).await.expect("handle");
        let hyper_res: hyper::Response<crate::types::BoxBody> = res.into();
        
        // Check CORS headers are present
        let headers = hyper_res.headers();
        assert_eq!(
            headers.get("Access-Control-Allow-Origin").unwrap(),
            "*"
        );
    }

    #[tokio::test]
    async fn test_cors_wildcard_origin() {
        let mut router = Router::new()
            .with_cors(CorsConfig::permissive());
        router.get("/test", || async { Ok(OxiditeResponse::text("ok")) });

        let req = http::Request::builder()
            .method(Method::GET)
            .uri("/test")
            .body(BoxBody::default())
            .expect("request");

        let res = router.handle(req).await.expect("handle");
        let hyper_res: hyper::Response<crate::types::BoxBody> = res.into();
        
        let headers = hyper_res.headers();
        assert_eq!(
            headers.get("Access-Control-Allow-Origin").unwrap(),
            "*"
        );
    }

    #[tokio::test]
    async fn test_no_cors_when_not_configured() {
        let mut router = Router::new();
        router.get("/test", || async { Ok(OxiditeResponse::text("ok")) });

        let req = http::Request::builder()
            .method(Method::GET)
            .uri("/test")
            .body(BoxBody::default())
            .expect("request");

        let res = router.handle(req).await.expect("handle");
        let hyper_res: hyper::Response<crate::types::BoxBody> = res.into();
        
        // Should NOT have CORS headers
        let headers = hyper_res.headers();
        assert!(headers.get("Access-Control-Allow-Origin").is_none());
        assert!(headers.get("Access-Control-Allow-Methods").is_none());
    }
}
