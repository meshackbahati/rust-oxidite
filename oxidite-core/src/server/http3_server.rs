#![cfg(feature = "http3")]

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Notify;
use rustls::ServerConfig;
use quinn::{Endpoint, ServerConfig as QuinnServerConfig};
use h3::server::RequestStream;
use h3_quinn;
use bytes::Bytes;
use http::{Request, Response};
use http_body_util::BodyExt;
use crate::error::Result;
use crate::types::{OxiditeRequest, OxiditeResponse};
use tower_service::Service;

pub struct Http3Server<S> {
    service: S,
}

impl<S> Http3Server<S>
where
    S: Service<OxiditeRequest, Response = OxiditeResponse, Error = crate::error::Error> 
        + Clone 
        + Send 
        + Sync 
        + 'static,
    S::Future: Send + 'static,
{
    pub fn new(service: S) -> Self {
        Self { service }
    }

    pub async fn listen(self, addr: SocketAddr, tls_config: ServerConfig) -> Result<()> {
        let crypto = quinn::crypto::rustls::QuicServerConfig::try_from(Arc::new(tls_config))
            .map_err(|e| crate::error::Error::InternalServerError(e.to_string()))?;
        let quinn_config = QuinnServerConfig::with_crypto(Arc::new(crypto));
        
        let endpoint = Endpoint::server(quinn_config, addr)
            .map_err(|e| crate::error::Error::InternalServerError(e.to_string()))?;

        println!("HTTP/3 server listening on https://{}", addr);

        let notify_shutdown = Arc::new(Notify::new());
        let (_shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::unbounded_channel::<()>();

        loop {
            tokio::select! {
                conn = endpoint.accept() => {
                    if let Some(conn) = conn {
                        let quic_conn = match conn.await {
                            Ok(conn) => conn,
                            Err(e) => {
                                eprintln!("Connection error: {}", e);
                                continue;
                            }
                        };

                        let service = self.service.clone();
                        let _notify = notify_shutdown.clone();
                        
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_connection(quic_conn, service).await {
                                eprintln!("Connection handler error: {}", e);
                            }
                        });
                    }
                }
                _ = shutdown_rx.recv() => {
                    println!("Shutting down HTTP/3 server...");
                    endpoint.close(0u32.into(), b"shutdown");
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_connection(
        quic_conn: quinn::Connection,
        service: S,
    ) -> Result<()> {
        let mut h3_conn = h3::server::Connection::new(h3_quinn::Connection::new(quic_conn))
            .await
            .map_err(|e| crate::error::Error::InternalServerError(e.to_string()))?;

        loop {
            match h3_conn.accept().await {
                Ok(Some(resolver)) => {
                    let (req, stream) = match resolver.resolve_request().await {
                        Ok(res) => res,
                        Err(e) => {
                            eprintln!("Error resolving request: {}", e);
                            continue;
                        }
                    };
                    Self::handle_request(req, stream, service.clone()).await?;
                }
                Ok(None) => break, // Connection closed
                Err(e) => {
                    eprintln!("Error accepting request: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_request(
        h3_request: Request<()>,
        mut stream: RequestStream<h3_quinn::BidiStream<Bytes>, Bytes>,
        mut service: S,
    ) -> Result<()> {
        // Convert H3 request to OxiditeRequest
        let (parts, _) = h3_request.into_parts();

        // For simplicity, we'll create a basic request body
        let body = http_body_util::Full::new(Bytes::new()).map_err(|e| match e {}).boxed();
        
        let oxidite_req = Request::from_parts(parts, body);

        // Process with the service
        let response = service.call(oxidite_req).await
            .map_err(|e| crate::error::Error::InternalServerError(e.to_string()))?;

        // Convert OxiditeResponse to H3 response
        let status = response.status();
        let response_headers = response.headers().clone();
        
        // Use into_inner() to get the underlying hyper Response, then consume body
        let response_body = response.into_inner().into_body().collect().await
            .map_err(|e| crate::error::Error::InternalServerError(e.to_string()))?
            .to_bytes();

        let mut h3_response = Response::builder()
            .status(status.as_u16());

        *h3_response.headers_mut().unwrap() = response_headers;

        stream.send_response(h3_response.body(()).unwrap()).await
            .map_err(|e| crate::error::Error::InternalServerError(e.to_string()))?;

        stream.send_data(response_body).await
            .map_err(|e| crate::error::Error::InternalServerError(e.to_string()))?;

        stream.finish().await
            .map_err(|e| crate::error::Error::InternalServerError(e.to_string()))?;

        Ok(())
    }
}

// Helper function to create TLS configuration
pub fn create_tls_config(cert_pem: &str, key_pem: &str) -> Result<ServerConfig> {
    use rustls_pemfile::{certs, pkcs8_private_keys};
    use std::io::Cursor;

    let cert_chain = certs(&mut Cursor::new(cert_pem))
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| crate::error::Error::InternalServerError(e.to_string()))?;

    let mut keys = pkcs8_private_keys(&mut Cursor::new(key_pem))
        .collect::<std::result::Result<Vec<_>, _>>()?;

    if keys.is_empty() {
        return Err(crate::error::Error::InternalServerError("No private keys found".to_string()));
    }

    let mut config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, rustls::pki_types::PrivateKeyDer::Pkcs8(keys.remove(0)))
        .map_err(|e| crate::error::Error::InternalServerError(e.to_string()))?;

    config.alpn_protocols = vec![b"h3".to_vec()];
    
    Ok(config)
}