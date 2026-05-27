use std::net::SocketAddr;
use tokio::net::TcpListener;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use hyper_util::service::TowerToHyperService;
use crate::error::{Error, Result};
use crate::types::{OxiditeRequest, OxiditeResponse};
use crate::router::CorsConfig;
use tower_service::Service;
use std::error::Error as StdError;
use http::HeaderValue;

use http_body_util::BodyExt;

use std::task::{Context, Poll};

#[cfg(feature = "http3")]
pub mod http3_server;

#[cfg(feature = "http3")]
pub use http3_server::Http3Server;

/// Adapter to convert hyper::Request<Incoming> to OxiditeRequest
#[derive(Clone)]
pub struct BodyAdapter<S> {
    inner: S,
    cors_config: Option<CorsConfig>,
}

impl<S> BodyAdapter<S> {
    pub fn new(service: S) -> Self {
        Self {
            inner: service,
            cors_config: None,
        }
    }

    pub fn with_cors(mut self, cors_config: Option<CorsConfig>) -> Self {
        self.cors_config = cors_config;
        self
    }

    /// Add CORS headers to a hyper response
    fn add_cors_to_response(&self, res: &mut hyper::Response<crate::types::BoxBody>) {
        if let Some(cors) = &self.cors_config {
            let headers = res.headers_mut();
            
            if let Some(origin) = cors.allowed_origins.first() {
                if let Ok(val) = HeaderValue::from_str(origin) {
                    headers.insert(http::header::ACCESS_CONTROL_ALLOW_ORIGIN, val);
                }
            }
            
            // Add Access-Control-Allow-Methods (join all methods with ", ")
            if !cors.allowed_methods.is_empty() {
                let methods = cors.allowed_methods.join(", ");
                if let Ok(val) = HeaderValue::from_str(&methods) {
                    headers.insert(http::header::ACCESS_CONTROL_ALLOW_METHODS, val);
                }
            }
            
            // Add Access-Control-Allow-Headers (join all headers with ", ")
            if !cors.allowed_headers.is_empty() {
                let headers_list = cors.allowed_headers.join(", ");
                if let Ok(val) = HeaderValue::from_str(&headers_list) {
                    headers.insert(http::header::ACCESS_CONTROL_ALLOW_HEADERS, val);
                }
            }
            
            if cors.allow_credentials {
                headers.insert(http::header::ACCESS_CONTROL_ALLOW_CREDENTIALS, HeaderValue::from_static("true"));
            }
            
            if let Ok(val) = HeaderValue::from_str(&cors.max_age.to_string()) {
                headers.insert(http::header::ACCESS_CONTROL_MAX_AGE, val);
            }
        }
    }
}



use std::pin::Pin;

impl<S> Service<hyper::Request<hyper::body::Incoming>> for BodyAdapter<S>
where
    S: Service<OxiditeRequest, Response = OxiditeResponse, Error = Error> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = hyper::Response<crate::types::BoxBody>;
    type Error = Error;
    type Future = Pin<Box<dyn std::future::Future<Output = std::result::Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<std::result::Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: hyper::Request<hyper::body::Incoming>) -> Self::Future {
        let accepts_html = req.headers().get(hyper::header::ACCEPT)
            .map(|h| h.to_str().unwrap_or("").contains("text/html"))
            .unwrap_or(false);
            
        let req = req.map(|b| b.map_err(|e| e.into()).boxed());
        let fut = self.inner.call(req);
        let cors = self.cors_config.clone();
        
        Box::pin(async move {
            match fut.await {
                Ok(response) => {
                    let mut hyper_response: hyper::Response<crate::types::BoxBody> = response.into();
                    // Add CORS headers to successful responses
                    let adapter = BodyAdapter { inner: (), cors_config: cors };
                    adapter.add_cors_to_response(&mut hyper_response);
                    Ok(hyper_response)
                },
                Err(error) => {
                    let env = std::env::var("OXIDITE_ENV").unwrap_or_else(|_| "development".to_string());
                    
                    if env == "development" && accepts_html && error.is_server_error() {
                        use bytes::Bytes;
                        use http_body_util::Full;
                        use hyper::header::{CONTENT_TYPE, SERVER};
                        
                        let html = crate::error::render_ignition_error(&error);
                        
                        let mut res = hyper::Response::builder()
                            .status(error.status_code())
                            .header(CONTENT_TYPE, "text/html; charset=utf-8")
                            .header(SERVER, crate::response::SERVER_HEADER_VALUE)
                            .body(Full::new(Bytes::from(html)).map_err(|e| match e {}).boxed())
                            .unwrap();
                        
                        // Add CORS headers to error responses too
                        let adapter = BodyAdapter { inner: (), cors_config: cors };
                        adapter.add_cors_to_response(&mut res);
                        Ok(res)
                    } else {
                        let mut error_response: hyper::Response<crate::types::BoxBody> = OxiditeResponse::from(error).into();
                        // Add CORS headers to error responses
                        let adapter = BodyAdapter { inner: (), cors_config: cors };
                        adapter.add_cors_to_response(&mut error_response);
                        Ok(error_response)
                    }
                }
            }
        })
    }
}



pub struct Server<S> {
    service: S,
    addr: Option<SocketAddr>,
    cors_config: Option<CorsConfig>,
}

impl<S> Server<S>
where
    S: Service<OxiditeRequest, Response = OxiditeResponse, Error = Error> + Clone + Send + Sync + 'static,
    S::Future: Send + 'static,
{
    pub fn new(service: S) -> Self {
        Self {
            service,
            addr: None,
            cors_config: None,
        }
    }

    pub fn bind(mut self, addr: SocketAddr) -> Self {
        self.addr = Some(addr);
        self
    }

    /// Configure CORS for the server (applies to all responses including errors)
    pub fn with_cors(mut self, cors_config: CorsConfig) -> Self {
        self.cors_config = Some(cors_config);
        self
    }

    pub async fn run(self) -> Result<()> {
        let addr = self.addr.unwrap_or_else(|| "127.0.0.1:3000".parse().unwrap());
        self.listen(addr).await
    }

    pub async fn listen(self, addr: SocketAddr) -> Result<()> {
        let listener = TcpListener::bind(addr).await?;
        println!("Listening on http://{}", addr);

        let cors_config = self.cors_config.clone();

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let service = self.service.clone();
            let cors = cors_config.clone();

            tokio::task::spawn(async move {
                let service = BodyAdapter::new(service).with_cors(cors);
                let hyper_service = TowerToHyperService::new(service);
                
                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, hyper_service)
                    .await
                {
                    // Only log actual server errors, not client errors like 404
                    // Check if this is a user service error that we can inspect
                    if let Some(service_err) = err.source().and_then(|e| e.downcast_ref::<Error>()) {
                        // Only log if it's a server error (5xx), not a client error (4xx)
                        if service_err.is_server_error() {
                            eprintln!("Server error: {}", service_err);
                        }
                        // Client errors (404, etc.) are silently handled - they're expected
                    } else {
                        // For non-service errors (connection issues, etc.), log them
                        // but only if they're not common expected errors
                        let err_msg = err.to_string();
                        // Don't log if it's just a client disconnecting or similar
                        if !err_msg.contains("NotFound") && !err_msg.contains("connection closed") {
                            eprintln!("Connection error: {}", err);
                        }
                    }
                }
            });
        }
    }

    /// Listen with both HTTP/1.1 and HTTP/3 support
    #[cfg(feature = "http3")]
    pub async fn listen_h3(self, addr: SocketAddr, cert_pem: &str, key_pem: &str) -> Result<()> {
        use rustls::ServerConfig;  
        use rustls_pemfile::{certs, pkcs8_private_keys};
        use std::io::Cursor;
        
        let cors_config = self.cors_config.clone();
        
        // Setup HTTP/1.1 server in background
        let http1_addr = addr;
        let http1_service = self.service.clone();
        let http1_cors = cors_config.clone();
        
        tokio::spawn(async move {
            let listener = TcpListener::bind(http1_addr).await.unwrap();
            println!("HTTP/1.1 server listening on http://{}", http1_addr);
            
            loop {
                let (stream, _) = listener.accept().await.unwrap();
                let io = TokioIo::new(stream);
                let service = http1_service.clone();
                let cors = http1_cors.clone();
                
                tokio::task::spawn(async move {
                    let service = BodyAdapter::new(service).with_cors(cors);
                    let hyper_service = TowerToHyperService::new(service);
                    
                    if let Err(err) = http1::Builder::new()
                        .serve_connection(io, hyper_service)
                        .await
                    {
                        // Only log server errors, not client errors
                        if let Some(service_err) = err.source().and_then(|e| e.downcast_ref::<Error>()) {
                            if service_err.is_server_error() {
                                eprintln!("HTTP/1.1 server error: {}", service_err);
                            }
                        } else {
                            let err_msg = err.to_string();
                            if !err_msg.contains("NotFound") && !err_msg.contains("connection closed") {
                                eprintln!("HTTP/1.1 connection error: {}", err);
                            }
                        }
                    }
                });
            }
        });
        
        // Setup HTTP/3 server
        let cert_chain = certs(&mut Cursor::new(cert_pem))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| crate::error::Error::InternalServerError(e.to_string()))?;
        
        let mut keys = pkcs8_private_keys(&mut Cursor::new(key_pem))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        
        if keys.is_empty() {
            return Err(crate::error::Error::InternalServerError("No private keys found".to_string()));
        }
        
        let tls_config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_chain, rustls::pki_types::PrivateKeyDer::Pkcs8(keys.remove(0)))
            .map_err(|e| crate::error::Error::InternalServerError(e.to_string()))?;
        
        let http3_server = Http3Server::new(self.service);
        http3_server.listen(addr, tls_config).await?;
        
        Ok(())
    }
}
