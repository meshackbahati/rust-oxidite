use oxidite_core::{Request, Response, Error};
use std::path::Path;

/// Configuration for static file serving
#[derive(Clone)]
pub struct StaticFiles {
    root: String,
    url_prefix: String,
}

impl StaticFiles {
    /// Create a new StaticFiles handler
    /// 
    /// # Arguments
    /// * `root` - The directory on the filesystem to serve files from (e.g., "public")
    /// * `url_prefix` - The URL prefix to strip from the request path (e.g., "/public")
    pub fn new(root: impl Into<String>, url_prefix: impl Into<String>) -> Self {
        Self {
            root: root.into(),
            url_prefix: url_prefix.into(),
        }
    }

    /// Serve a static file based on the request
    pub async fn serve(&self, req: Request) -> Result<Response, Error> {
        let path = req.uri().path();
        
        // Remove prefix
        let file_path = if path.starts_with(&self.url_prefix) {
            path.strip_prefix(&self.url_prefix).unwrap_or(path)
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
                
                let mut response = Response::new(content.into());
                response.headers_mut().insert(
                    "content-type",
                    content_type.parse().unwrap()
                );
                Ok(response)
            },
            Err(_) => Err(Error::NotFound)
        }
    }
}

/// Helper function to serve static files from a "public" directory
/// mapped to "/public" URL prefix.
/// 
/// This is a convenience wrapper around `StaticFiles`.
pub async fn serve_static(req: Request) -> Result<Response, Error> {
    let static_files = StaticFiles::new("public", "/public");
    static_files.serve(req).await
}
