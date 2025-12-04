use oxidite_core::{OxiditeRequest, OxiditeResponse, Error as CoreError};
use tower::{Service, Layer};
use std::task::{Context, Poll};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use cookie::{Cookie, CookieJar, SameSite};
use crate::session::{Session, SessionStore};

const SESSION_COOKIE_NAME: &str = "oxidite_session";

/// Session middleware
#[derive(Clone)]
pub struct SessionMiddleware<S> {
    inner: S,
    store: Arc<dyn SessionStore>,
    cookie_secure: bool,
    cookie_http_only: bool,
    session_ttl_secs: u64,
}

impl<S> SessionMiddleware<S> {
    pub fn new(
        inner: S,
        store: Arc<dyn SessionStore>,
        cookie_secure: bool,
        cookie_http_only: bool,
        session_ttl_secs: u64,
    ) -> Self {
        Self {
            inner,
            store,
            cookie_secure,
            cookie_http_only,
            session_ttl_secs,
        }
    }
}

impl<S> Service<OxiditeRequest> for SessionMiddleware<S>
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
        // Extract session cookie
        let session_id = req
            .headers()
            .get("cookie")
            .and_then(|h| h.to_str().ok())
            .and_then(|cookies| {
                for cookie_str in cookies.split(';') {
                    if let Ok(cookie) = Cookie::parse(cookie_str.trim()) {
                        if cookie.name() == SESSION_COOKIE_NAME {
                            return Some(cookie.value().to_string());
                        }
                    }
                }
                None
            });

        let store = self.store.clone();
        let cookie_secure = self.cookie_secure;
        let cookie_http_only = self.cookie_http_only;
        let session_ttl_secs = self.session_ttl_secs;
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Try to load existing session
            let session = if let Some(sid) = session_id {
                store.get(&sid).await.ok().flatten()
            } else {
                None
            };

            // TODO: Attach session to request context
            // For now, we just validate that the session exists
            // In a full implementation, we'd use request extensions

            let mut response = inner.call(req).await?;

            // If session was renewed or created, set cookie
            if let Some(sess) = session {
                if !sess.is_expired() {
                    let cookie = Cookie::build((SESSION_COOKIE_NAME, sess.id.clone()))
                        .secure(cookie_secure)
                        .http_only(cookie_http_only)
                        .same_site(SameSite::Lax)
                        .max_age(cookie::time::Duration::seconds(session_ttl_secs as i64))
                        .path("/")
                        .build();

                    if let Ok(cookie_val) = cookie.to_string().parse() {
                        response.headers_mut().insert("set-cookie", cookie_val);
                    }
                }
            }

            Ok(response)
        })
    }
}

/// Layer for session middleware
pub struct SessionLayer {
    store: Arc<dyn SessionStore>,
    cookie_secure: bool,
    cookie_http_only: bool,
    session_ttl_secs: u64,
}

impl SessionLayer {
    pub fn new(
        store: Arc<dyn SessionStore>,
        cookie_secure: bool,
        cookie_http_only: bool,
        session_ttl_secs: u64,
    ) -> Self {
        Self {
            store,
            cookie_secure,
            cookie_http_only,
            session_ttl_secs,
        }
    }

    pub fn with_defaults(store: Arc<dyn SessionStore>) -> Self {
        Self::new(store, true, true, 3600)
    }
}

impl<S> Layer<S> for SessionLayer {
    type Service = SessionMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SessionMiddleware::new(
            inner,
            self.store.clone(),
            self.cookie_secure,
            self.cookie_http_only,
            self.session_ttl_secs,
        )
    }
}
