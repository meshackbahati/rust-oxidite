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

        // Read file
        match std::fs::read_to_string(&full_path) {
            Ok(content) => {
                // Set content type based on extension
                let content_type = if full_path.extension().map_or(false, |ext| ext == "css") {
                    "text/css"
                } else if full_path.extension().map_or(false, |ext| ext == "js") {
                    "application/javascript"
                } else if full_path.extension().map_or(false, |ext| ext == "svg") {
                    "image/svg+xml"
                } else if full_path.extension().map_or(false, |ext| ext == "png") {
                    "image/png"
                } else if full_path.extension().map_or(false, |ext| ext == "jpg" || ext == "jpeg") {
                    "image/jpeg"
                } else if full_path.extension().map_or(false, |ext| ext == "html") {
                    "text/html"
                } else if full_path.extension().map_or(false, |ext| ext == "json") {
                    "application/json"
                } else {
                    "text/plain"
                };
                
                Ok(OxiditeResponse::html(content))
            },
            Err(_) => {
                // Return 404 Response instead of Error
                Ok(OxiditeResponse::html("404 Not Found"))
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
