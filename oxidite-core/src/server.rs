use std::net::SocketAddr;
use tokio::net::TcpListener;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use hyper_util::service::TowerToHyperService;
use crate::error::{Error, Result};
use crate::types::{OxiditeRequest, OxiditeResponse};
use tower_service::Service;

use http_body_util::BodyExt;

use std::task::{Context, Poll};

#[cfg(feature = "http3")]
pub mod http3_server;

#[cfg(feature = "http3")]
pub use http3_server::Http3Server;

/// Adapter to convert hyper::Request<Incoming> to OxiditeRequest
#[derive(Clone)]
pub struct BodyAdapter<S>(S);

impl<S> BodyAdapter<S> {
    pub fn new(service: S) -> Self {
        Self(service)
    }
}

use futures_util::future::Map;
use futures_util::FutureExt;

impl<S> Service<hyper::Request<hyper::body::Incoming>> for BodyAdapter<S>
where
    S: Service<OxiditeRequest, Response = OxiditeResponse, Error = Error> + Clone,
{
    type Response = hyper::Response<crate::types::BoxBody>;
    type Error = Error;
    type Future = Map<S::Future, fn(std::result::Result<OxiditeResponse, Error>) -> std::result::Result<hyper::Response<crate::types::BoxBody>, Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<std::result::Result<(), Self::Error>> {
        self.0.poll_ready(cx)
    }

    fn call(&mut self, req: hyper::Request<hyper::body::Incoming>) -> Self::Future {
        let req = req.map(|b| b.map_err(|e| e.into()).boxed());
        fn map_response(res: std::result::Result<OxiditeResponse, Error>) -> std::result::Result<hyper::Response<crate::types::BoxBody>, Error> {
            res.map(|r| r.into())
        }
        self.0.call(req).map(map_response)
    }
}


pub struct Server<S> {
    service: S,
}

impl<S> Server<S>
where
    S: Service<OxiditeRequest, Response = OxiditeResponse, Error = Error> + Clone + Send + Sync + 'static,
    S::Future: Send + 'static,
{
    pub fn new(service: S) -> Self {
        Self {
            service,
        }
    }

    pub async fn listen(self, addr: SocketAddr) -> Result<()> {
        let listener = TcpListener::bind(addr).await?;
        println!("Listening on http://{}", addr);

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let service = self.service.clone();

            tokio::task::spawn(async move {
                let service = BodyAdapter::new(service);
                let hyper_service = TowerToHyperService::new(service);
                
                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, hyper_service)
                    .await
                {
                    // This `err` is a `hyper::Error`, not `crate::error::Error`.
                    // The user's requested logging for `crate::error::Error` types
                    // is now handled within the `hyper_compatible_service` wrapper.
                    // This `eprintln` now only catches connection-level `hyper::Error`s.
                    eprintln!("Error serving connection: {:?}", err);
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
        
        // Setup HTTP/1.1 server in background
        let http1_addr = addr;
        let http1_service = self.service.clone();
        
        tokio::spawn(async move {
            let listener = TcpListener::bind(http1_addr).await.unwrap();
            println!("HTTP/1.1 server listening on http://{}", http1_addr);
            
            loop {
                let (stream, _) = listener.accept().await.unwrap();
                let io = TokioIo::new(stream);
                let service = http1_service.clone();
                
                tokio::task::spawn(async move {
                    let service = BodyAdapter::new(service);
                    let hyper_service = TowerToHyperService::new(service);
                    
                    if let Err(err) = http1::Builder::new()
                        .serve_connection(io, hyper_service)
                        .await
                    {
                        eprintln!("HTTP/1.1 connection error: {:?}", err);
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
