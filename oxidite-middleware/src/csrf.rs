use oxidite_core::{OxiditeRequest, OxiditeResponse, Error as CoreError};
use tower::{Service, Layer};
use std::task::{Context, Poll};
use std::future::Future;
use std::pin::Pin;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};
use rand::Rng;

const CSRF_TOKEN_HEADER: &str = "x-csrf-token";
const CSRF_COOKIE_NAME: &str = "csrf_token";

/// CSRF protection middleware
#[derive(Clone)]
pub struct CsrfMiddleware<S> {
    inner: S,
    config: CsrfConfig,
}

#[derive(Clone, Debug)]
pub struct CsrfConfig {
    pub token_length: usize,
    pub exempt_paths: Vec<String>,
}

impl Default for CsrfConfig {
    fn default() -> Self {
        Self {
            token_length: 32,
            exempt_paths: vec![],
        }
    }
}

impl<S> CsrfMiddleware<S> {
    pub fn new(inner: S, config: CsrfConfig) -> Self {
        Self { inner, config }
    }

    fn is_exempt(&self, path: &str) -> bool {
        self.config.exempt_paths.iter().any(|exempt| path.starts_with(exempt))
    }

    fn generate_token() -> String {
        let random_bytes: Vec<u8> = (0..32).map(|_| rand::rng().random()).collect();
        general_purpose::STANDARD.encode(random_bytes)
    }

    fn verify_token(token: &str, cookie_token: &str) -> bool {
        token == cookie_token
    }
}

impl<S> Service<OxiditeRequest> for CsrfMiddleware<S>
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
        let path = req.uri().path().to_string();
        let method = req.method().clone();
        
        // Check if path is exempt
        let is_exempt =  self.is_exempt(&path);
        
        // Extract CSRF token from header
        let header_token = req
            .headers()
            .get(CSRF_TOKEN_HEADER)
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        // Extract CSRF token from cookie (simplified - in production use proper cookie parsing)
        let cookie_token = req
            .headers()
            .get("cookie")
            .and_then(|h| h.to_str().ok())
            .and_then(|cookies| {
                cookies.split(';')
                    .find(|c| c.trim().starts_with(CSRF_COOKIE_NAME))
                    .and_then(|c| c.split('=').nth(1))
                    .map(|s| s.trim().to_string())
            });

        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Validate CSRF for state-changing methods
            if !is_exempt && (method == "POST" || method == "PUT" || method == "DELETE" || method == "PATCH") {
                match (header_token, cookie_token.clone()) {
                    (Some(h_token), Some(c_token)) => {
                        if !CsrfMiddleware::<S>::verify_token(&h_token, &c_token) {
                            return Err(CoreError::BadRequest("Invalid CSRF token".to_string()));
                        }
                    }
                    _ => {
                        return Err(CoreError::BadRequest("Missing CSRF token".to_string()));
                    }
                }
            }

            let mut response = inner.call(req).await?;

            // Set CSRF token cookie if not present
            if cookie_token.is_none() {
                let new_token = CsrfMiddleware::<S>::generate_token();
                let cookie_value = format!("{}={}; HttpOnly; SameSite=Strict; Path=/", CSRF_COOKIE_NAME, new_token);
                if let Ok(value) = cookie_value.parse() {
                    response.headers_mut().insert("set-cookie", value);
                }
            }

            Ok(response)
        })
    }
}

/// Layer for CSRF middleware
#[derive(Clone)]
pub struct CsrfLayer {
    config: CsrfConfig,
}

impl CsrfLayer {
    pub fn new(config: CsrfConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self {
            config: CsrfConfig::default(),
        }
    }
}

impl<S> Layer<S> for CsrfLayer {
    type Service = CsrfMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CsrfMiddleware::new(inner, self.config.clone())
    }
}
