use crate::error::{Error, Result};
use crate::types::{OxiditeRequest, OxiditeResponse};
use hyper::Method;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower_service::Service;
use regex::Regex;

pub trait Handler: Send + Sync + 'static {
    fn call(&self, req: OxiditeRequest) -> Pin<Box<dyn Future<Output = Result<OxiditeResponse>> + Send>>;
}

impl<F, Fut> Handler for F
where
    F: Fn(OxiditeRequest) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<OxiditeResponse>> + Send + 'static,
{
    fn call(&self, req: OxiditeRequest) -> Pin<Box<dyn Future<Output = Result<OxiditeResponse>> + Send>> {
        Box::pin(self(req))
    }
}

struct Route {
    pattern: Regex,
    param_names: Vec<String>,
    handler: Arc<dyn Handler>,
}

#[derive(Clone)]
pub struct Router {
    routes: HashMap<Method, Vec<Arc<Route>>>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    pub fn get<H>(&mut self, path: &str, handler: H)
    where
        H: Handler,
    {
        self.add_route(Method::GET, path, handler);
    }
    
    pub fn post<H>(&mut self, path: &str, handler: H)
    where
        H: Handler,
    {
        self.add_route(Method::POST, path, handler);
    }

    pub fn put<H>(&mut self, path: &str, handler: H)
    where
        H: Handler,
    {
        self.add_route(Method::PUT, path, handler);
    }

    pub fn delete<H>(&mut self, path: &str, handler: H)
    where
        H: Handler,
    {
        self.add_route(Method::DELETE, path, handler);
    }

    pub fn patch<H>(&mut self, path: &str, handler: H)
    where
        H: Handler,
    {
        self.add_route(Method::PATCH, path, handler);
    }

    fn add_route<H>(&mut self, method: Method, path: &str, handler: H)
    where
        H: Handler,
    {
        let (pattern, param_names) = compile_path(path);
        let route = Arc::new(Route {
            pattern,
            param_names,
            handler: Arc::new(handler),
        });
        
        self.routes
            .entry(method)
            .or_insert_with(Vec::new)
            .push(route);
    }

    pub async fn handle(&self, mut req: OxiditeRequest) -> Result<OxiditeResponse> {
        let method = req.method().clone();
        let path = req.uri().path().to_string();

        if let Some(routes) = self.routes.get(&method) {
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

                    return route.handler.call(req).await;
                }
            }
        }

        Err(Error::NotFound)
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
