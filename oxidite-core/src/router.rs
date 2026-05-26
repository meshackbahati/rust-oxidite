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

impl<S, E> Endpoint for tower_http::cors::Cors<S>
where
    S: Service<OxiditeRequest, Response = OxiditeResponse, Error = E> + Clone + Send + Sync + 'static,
    S::Future: Send + 'static,
    E: std::fmt::Display + Send + 'static,
{
    fn call(&self, req: OxiditeRequest) -> Pin<Box<dyn Future<Output = Result<OxiditeResponse>> + Send>> {

        let this = self.clone();
        Box::pin(async move {
            let res = this.call(req).await.map_err(|e| Error::InternalServerError(e.to_string()))?;
            Ok(res)
        })
    }
}

impl<S, E> Endpoint for tower_http::compression::Compression<S>
where
    S: Service<OxiditeRequest, Response = OxiditeResponse, Error = E> + Clone + Send + Sync + 'static,
    S::Future: Send + 'static,
    E: std::fmt::Display + Send + 'static,
{
    fn call(&self, req: OxiditeRequest) -> Pin<Box<dyn Future<Output = Result<OxiditeResponse>> + Send>> {

        let this = self.clone();
        Box::pin(async move {
            let res = this.call(req).await.map_err(|e| Error::InternalServerError(e.to_string()))?;
            Ok(res)
        })
    }
}


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

#[derive(Clone)]
pub struct Router {
    routes: HashMap<Method, Vec<Arc<Route>>>,
    extensions: Arc<std::sync::RwLock<http::Extensions>>,
    middleware: Vec<Arc<dyn Fn(Arc<dyn Endpoint>) -> Arc<dyn Endpoint> + Send + Sync>>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            extensions: Arc::new(std::sync::RwLock::new(http::Extensions::new())),
            middleware: Vec::new(),
        }
    }

    /// Add a shared state to the router that will be available in all handlers
    pub fn with_state<T: Clone + Send + Sync + 'static>(&mut self, state: T) {
        if let Ok(mut exts) = self.extensions.write() {
            exts.insert(state);
        }
    }

    pub fn get<H, Args>(&mut self, path: &str, handler: H)
    where
        H: Handler<Args>,
        Args: Send + Sync + 'static,
    {
        self.add_route(Method::GET, path, handler);
    }
    
    pub fn post<H, Args>(&mut self, path: &str, handler: H)
    where
        H: Handler<Args>,
        Args: Send + Sync + 'static,
    {
        self.add_route(Method::POST, path, handler);
    }

    pub fn put<H, Args>(&mut self, path: &str, handler: H)
    where
        H: Handler<Args>,
        Args: Send + Sync + 'static,
    {
        self.add_route(Method::PUT, path, handler);
    }

    pub fn delete<H, Args>(&mut self, path: &str, handler: H)
    where
        H: Handler<Args>,
        Args: Send + Sync + 'static,
    {
        self.add_route(Method::DELETE, path, handler);
    }

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

    pub async fn handle(&self, mut req: OxiditeRequest) -> Result<OxiditeResponse> {
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
            return route.handler.call(req).await;
        }

        // 2. If HEAD, try GET
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
}
