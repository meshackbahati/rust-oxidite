use oxidite_core::{OxiditeRequest, OxiditeResponse, Error, Result};

use std::path::Path;
use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;

/// Configuration for static file serving
#[derive(Clone)]
pub struct StaticFiles {
    root: String,
    url_prefix: Option<String>,
}

impl StaticFiles {
    /// Create a new StaticFiles handler
    /// 
    /// # Arguments
    /// * `root` - The directory on the filesystem to serve files from (e.g., "public")
    /// * `url_prefix` - Optional URL prefix to strip from the request path (e.g., "/public")
    pub fn new(root: impl Into<String>, url_prefix: Option<String>) -> Self {
        Self {
            root: root.into(),
            url_prefix,
        }
    }

    /// Serve a static file based on the request
    pub async fn serve(&self, req: OxiditeRequest) -> Result<OxiditeResponse> {
        use hyper::{Response, header};
        use http_body_util::{Full, BodyExt};
        use bytes::Bytes;
        use http::StatusCode;

        let path = req.uri().path();
        
        // Remove prefix if configured
        let file_path = if let Some(prefix) = &self.url_prefix {
            if path.starts_with(prefix) {
                path.strip_prefix(prefix).unwrap_or(path)
            } else {
                path
            }
        } else {
            path
        };

        // Clean up leading slashes to make it relative
        let file_path = file_path.trim_start_matches('/');
        
        // Security: prevent directory traversal
        if file_path.contains("..") {
            return Err(Error::BadRequest("Invalid path".to_string()));
        }
        
        let full_path = Path::new(&self.root).join(file_path);
        
        // Check if path is a directory, if so try index.html
        let full_path = if full_path.is_dir() {
            full_path.join("index.html")
        } else {
            full_path
        };

        // Read file asynchronously as bytes
        match tokio::fs::read(&full_path).await {
            Ok(content) => {
                // Determine content type based on extension
                let content_type = full_path.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| match ext.to_lowercase().as_str() {
                        "html" | "htm" => "text/html",
                        "css" => "text/css",
                        "js" | "mjs" => "application/javascript",
                        "json" => "application/json",
                        "png" => "image/png",
                        "jpg" | "jpeg" => "image/jpeg",
                        "gif" => "image/gif",
                        "svg" => "image/svg+xml",
                        "ico" => "image/x-icon",
                        "webp" => "image/webp",
                        "woff" => "font/woff",
                        "woff2" => "font/woff2",
                        "ttf" => "font/ttf",
                        "otf" => "font/otf",
                        "eot" => "application/vnd.ms-fontobject",
                        "wasm" => "application/wasm",
                        "mp4" => "video/mp4",
                        "webm" => "video/webm",
                        "txt" => "text/plain",
                        "xml" => "text/xml",
                        _ => "application/octet-stream",
                    })
                    .unwrap_or("application/octet-stream");
                
                let res = Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, content_type)
                    .header(header::CONTENT_LENGTH, content.len())
                    .header(header::SERVER, "Oxidite/2.0.1")
                    .body(Full::new(Bytes::from(content)).map_err(|e| match e {}).boxed())
                    .map_err(|e| Error::InternalServerError(format!("Failed to build response: {}", e)))?;
                
                Ok(OxiditeResponse::new(res))
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // Return 404 Not Found
                let res = Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header(header::CONTENT_TYPE, "text/plain")
                    .header(header::SERVER, "Oxidite/2.0.1")
                    .body(Full::new(Bytes::from("404 Not Found")).map_err(|e| match e {}).boxed())
                    .map_err(|e| Error::InternalServerError(format!("Failed to build response: {}", e)))?;
                
                Ok(OxiditeResponse::new(res))
            },
            Err(e) => {
                // Return 500 Internal Server Error for other errors
                Err(Error::InternalServerError(format!("Failed to read file: {}", e)))
            }
        }
    }
}

/// Create a static file handler for a specific directory.
/// 
/// # Example
/// ```rust
/// router.get("/assets/*", static_handler("public"));
/// ```
pub fn static_handler(root: impl Into<String>) -> impl Fn(OxiditeRequest) -> Pin<Box<dyn Future<Output = Result<OxiditeResponse>> + Send>> + Send + Sync + 'static {
    let root = root.into();
    let static_files = Arc::new(StaticFiles::new(root, None));
    
    move |req| {
        let static_files = static_files.clone();
        Box::pin(async move {
            static_files.serve(req).await
        })
    }
}

/// Helper function to serve static files from the "public" directory.
/// 
/// This handler serves files relative to the root of the "public" directory.
/// For example, a request to `/style.css` will serve `public/style.css`.
pub async fn serve_static(req: OxiditeRequest) -> Result<OxiditeResponse> {
    let static_files = StaticFiles::new("public", None);
    static_files.serve(req).await
}
