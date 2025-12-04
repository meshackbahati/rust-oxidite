//! HTTPS and HTTP/2 support for Oxidite

use std::net::SocketAddr;
use tokio::net::TcpListener;
use hyper::server::conn::{http1, http2};
use hyper_util::rt::TokioIo;
use hyper_util::service::TowerToHyperService;
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio_rustls::TlsAcceptor;
use std::sync::Arc;
use std::fs::File;
use std::io::BufReader;
use crate::error::{Error, Result};
use crate::types::{OxiditeRequest, OxiditeResponse};
use tower_service::Service;

/// TLS configuration for HTTPS
pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
}

impl TlsConfig {
    pub fn new(cert_path: impl Into<String>, key_path: impl Into<String>) -> Self {
        Self {
            cert_path: cert_path.into(),
            key_path: key_path.into(),
        }
    }
    
    /// Load certificates and private key
    pub fn load_config(&self) -> Result<ServerConfig> {
        let certs = load_certs(&self.cert_path)?;
        let key = load_private_key(&self.key_path)?;
        
        Ok(ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| Error::Server(e.to_string()))?)
    }
}

fn load_certs(path: &str) -> Result<Vec<CertificateDer<'static>>> {
    let file = File::open(path).map_err(|e| Error::Server(format!("Failed to open cert file: {}", e)))?;
    let mut reader = BufReader::new(file);
    rustls_pemfile::certs(&mut reader)
        .map(|res| res.map_err(|e| Error::Server(format!("Failed to parse cert: {}", e))))
        .collect::<Result<Vec<_>>>()
}

fn load_private_key(path: &str) -> Result<PrivateKeyDer<'static>> {
    let file = File::open(path).map_err(|e| Error::Server(format!("Failed to open key file: {}", e)))?;
    let mut reader = BufReader::new(file);

    // Try to read the first private key
    loop {
        match rustls_pemfile::read_one(&mut reader).map_err(|e| Error::Server(format!("Failed to parse key: {}", e)))? {
            Some(rustls_pemfile::Item::Pkcs1Key(key)) => return Ok(key.into()),
            Some(rustls_pemfile::Item::Pkcs8Key(key)) => return Ok(key.into()),
            Some(rustls_pemfile::Item::Sec1Key(key)) => return Ok(key.into()),
            None => break,
            _ => {} // Ignore other items like certificates
        }
    }

    Err(Error::Server("No supported private key found".to_string()))
}

/// HTTP protocol version
#[derive(Debug, Clone, Copy)]
pub enum HttpVersion {
    Http1,
    Http2,
    Auto, // Automatically negotiate
}

/// Server builder with HTTPS support
pub struct SecureServer<S> {
    service: S,
    tls_config: Option<TlsConfig>,
    http_version: HttpVersion,
}

impl<S> SecureServer<S>
where
    S: Service<OxiditeRequest, Response = OxiditeResponse, Error = Error> + Clone + Send + Sync + 'static,
    S::Future: Send + 'static,
{
    pub fn new(service: S) -> Self {
        Self {
            service,
            tls_config: None,
            http_version: HttpVersion::Auto,
        }
    }
    
    /// Enable HTTPS with TLS certificates
    pub fn with_tls(mut self, tls_config: TlsConfig) -> Self {
        self.tls_config = Some(tls_config);
        self
    }
    
    /// Set HTTP version
    pub fn with_http_version(mut self, version: HttpVersion) -> Self {
        self.http_version = version;
        self
    }
    
    /// Start the server
    pub async fn listen(self, addr: SocketAddr) -> Result<()> {
        if let Some(tls_config) = self.tls_config {
            Self::listen_https(addr, self.service, tls_config, self.http_version).await
        } else {
            Self::listen_http(addr, self.service).await
        }
    }
    
    /// Listen on HTTP
    async fn listen_http(addr: SocketAddr, service: S) -> Result<()> {
        let listener = TcpListener::bind(addr).await?;
        println!("Listening on http://{}", addr);

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let service = service.clone();

            tokio::task::spawn(async move {
                let hyper_service = TowerToHyperService::new(service);
                
                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, hyper_service)
                    .await
                {
                    eprintln!("Error serving connection: {:?}", err);
                }
            });
        }
    }
    
    /// Listen on HTTPS
    async fn listen_https(addr: SocketAddr, service: S, tls_config: TlsConfig, http_version: HttpVersion) -> Result<()> {
        let server_config = tls_config.load_config()?;
        let acceptor = TlsAcceptor::from(Arc::new(server_config));
        
        let listener = TcpListener::bind(addr).await?;
        println!("Listening on https://{}", addr);

        loop {
            let (stream, _) = listener.accept().await?;
            let acceptor = acceptor.clone();
            let service = service.clone();

            tokio::task::spawn(async move {
                match acceptor.accept(stream).await {
                    Ok(tls_stream) => {
                        let io = TokioIo::new(tls_stream);
                        let hyper_service = TowerToHyperService::new(service);
                        
                        let result = match http_version {
                            HttpVersion::Http1 => {
                                http1::Builder::new()
                                    .serve_connection(io, hyper_service)
                                    .await
                            }
                            HttpVersion::Http2 => {
                                http2::Builder::new(TokioExecutor)
                                    .serve_connection(io, hyper_service)
                                    .await
                            }
                            HttpVersion::Auto => {
                                // Use HTTP/1.1 by default, upgrade to HTTP/2 if requested
                                http1::Builder::new()
                                    .serve_connection(io, hyper_service)
                                    .await
                            }
                        };
                        
                        if let Err(err) = result {
                            eprintln!("Error serving TLS connection: {:?}", err);
                        }
                    }
                    Err(err) => {
                        eprintln!("TLS accept error: {:?}", err);
                    }
                }
            });
        }
    }
}

// Executor for HTTP/2
#[derive(Clone)]
struct TokioExecutor;

impl<F> hyper::rt::Executor<F> for TokioExecutor
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    fn execute(&self, fut: F) {
        tokio::task::spawn(fut);
    }
}
