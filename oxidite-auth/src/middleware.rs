use oxidite_core::{OxiditeRequest, OxiditeResponse, Error as CoreError};
use tower::{Service, Layer};
use std::task::{Context, Poll};
use std::future::Future;
use std::pin::Pin;
use crate::verify_token;

/// Auth middleware that validates JWT tokens
#[derive(Clone)]
pub struct AuthMiddleware<S> {
    inner: S,
    secret: String,
}

impl<S> AuthMiddleware<S> {
    pub fn new(inner: S, secret: String) -> Self {
        Self { inner, secret }
    }
}

impl<S> Service<OxiditeRequest> for AuthMiddleware<S>
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
        // Extract Authorization header before moving req
        let token = req
            .headers()
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .map(|s| s.to_string());

        let secret = self.secret.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Verify token
            if let Some(token_str) = token {
                match verify_token(&token_str, &secret) {
                    Ok(_claims) => {
                        // Token is valid, proceed with request
                        inner.call(req).await
                    }
                    Err(_) => {
                        // Invalid token
                        Err(CoreError::BadRequest("Invalid token".to_string()))
                    }
                }
            } else {
                // No token provided
                Err(CoreError::BadRequest("Missing authorization header".to_string()))
            }
        })
    }
}

/// Layer for Auth middleware
pub struct AuthLayer {
    secret: String,
}

impl AuthLayer {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }
}

impl<S> Layer<S> for AuthLayer {
    type Service = AuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthMiddleware::new(inner, self.secret.clone())
    }
}
