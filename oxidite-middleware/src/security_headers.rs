use oxidite_core::{OxiditeRequest, OxiditeResponse, Error as CoreError};
use tower::{Service, Layer};
use std::task::{Context, Poll};
use std::future::Future;
use std::pin::Pin;

/// Security headers middleware
#[derive(Clone)]
pub struct SecurityHeadersMiddleware<S> {
    inner: S,
    config: SecurityHeadersConfig,
}

#[derive(Clone, Debug)]
pub struct SecurityHeadersConfig {
    pub csp: Option<String>,
    pub hsts_max_age: Option<u64>,
    pub frame_options: FrameOptions,
    pub content_type_options: bool,
    pub xss_protection: bool,
    pub referrer_policy: Option<String>,
}

#[derive(Clone, Debug)]
pub enum FrameOptions {
    Deny,
    SameOrigin,
    Allow,
}

impl Default for SecurityHeadersConfig {
    fn default() -> Self {
        Self {
            csp: Some("default-src 'self'".to_string()),
            hsts_max_age: Some(31536000), // 1 year
            frame_options: FrameOptions::SameOrigin,
            content_type_options: true,
            xss_protection: true,
            referrer_policy: Some("strict-origin-when-cross-origin".to_string()),
        }
    }
}

impl<S> SecurityHeadersMiddleware<S> {
    pub fn new(inner: S, config: SecurityHeadersConfig) -> Self {
        Self { inner, config }
    }
}

impl<S> Service<OxiditeRequest> for SecurityHeadersMiddleware<S>
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
        let config = self.config.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let mut response = inner.call(req).await?;
            let headers = response.headers_mut();

            // Content-Security-Policy
            if let Some(csp) = &config.csp {
                if let Ok(value) = csp.parse() {
                    headers.insert("content-security-policy", value);
                }
            }

            // Strict-Transport-Security
            if let Some(max_age) = config.hsts_max_age {
                let hsts = format!("max-age={}; includeSubDomains", max_age);
                if let Ok(value) = hsts.parse() {
                    headers.insert("strict-transport-security", value);
                }
            }

            // X-Frame-Options
            let frame_option = match config.frame_options {
                FrameOptions::Deny => "DENY",
                FrameOptions::SameOrigin => "SAMEORIGIN",
                FrameOptions::Allow => return Ok(response),
            };
            if let Ok(value) = frame_option.parse() {
                headers.insert("x-frame-options", value);
            }

            // X-Content-Type-Options
            if config.content_type_options {
                if let Ok(value) = "nosniff".parse() {
                    headers.insert("x-content-type-options", value);
                }
            }

            // X-XSS-Protection
            if config.xss_protection {
                if let Ok(value) = "1; mode=block".parse() {
                    headers.insert("x-xss-protection", value);
                }
            }

            // Referrer-Policy
            if let Some(policy) = &config.referrer_policy {
                if let Ok(value) = policy.parse() {
                    headers.insert("referrer-policy", value);
                }
            }

            Ok(response)
        })
    }
}

/// Layer for security headers middleware
#[derive(Clone)]
pub struct SecurityHeadersLayer {
    config: SecurityHeadersConfig,
}

impl SecurityHeadersLayer {
    pub fn new(config: SecurityHeadersConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self {
            config: SecurityHeadersConfig::default(),
        }
    }
}

impl<S> Layer<S> for SecurityHeadersLayer {
    type Service = SecurityHeadersMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SecurityHeadersMiddleware::new(inner, self.config.clone())
    }
}
