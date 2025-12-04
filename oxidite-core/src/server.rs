use std::net::SocketAddr;
use tokio::net::TcpListener;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use hyper_util::service::TowerToHyperService;
use crate::error::{Error, Result};
use crate::types::{OxiditeRequest, OxiditeResponse};
use tower_service::Service;
use std::future::Future;

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
}
