//! API Versioning support

use std::collections::HashMap;
use crate::{Router, OxiditeRequest, OxiditeResponse};

/// API version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ApiVersion {
    V1,
    V2,
    V3,
    Custom(u8),
}

impl ApiVersion {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "v1" | "1" => Some(ApiVersion::V1),
            "v2" | "2" => Some(ApiVersion::V2),
            "v3" | "3" => Some(ApiVersion::V3),
            _ => s.trim_start_matches('v').parse::<u8>().ok().map(ApiVersion::Custom),
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            ApiVersion::V1 => "v1",
            ApiVersion::V2 => "v2",
            ApiVersion::V3 => "v3",
            ApiVersion::Custom(_) => "custom",
        }
    }
}

/// Versioned router
pub struct VersionedRouter {
    routers: HashMap<ApiVersion, Router>,
    default_version: ApiVersion,
}

impl VersionedRouter {
    pub fn new(default_version: ApiVersion) -> Self {
        Self {
            routers: HashMap::new(),
            default_version,
        }
    }
    
    /// Add a router for a specific version
    pub fn version(&mut self, version: ApiVersion, router: Router) {
        self.routers.insert(version, router);
    }
    
    /// Extract version from request
    /// Supports:
    /// - URL path: /api/v1/users
    /// - Header: Accept: application/vnd.api+json;version=1
    /// - Query param: /api/users?version=1
    pub fn extract_version(&self, req: &OxiditeRequest) -> ApiVersion {
        // Try URL path first
        if let Some(path) = req.uri().path().split('/').find(|s| s.starts_with('v')) {
            if let Some(version) = ApiVersion::from_str(path) {
                return version;
            }
        }
        
        // Try Accept header
        if let Some(accept) = req.headers().get("accept") {
            if let Ok(accept_str) = accept.to_str() {
                if let Some(version_part) = accept_str.split(";version=").nth(1) {
                    if let Some(version) = ApiVersion::from_str(version_part.split(',').next().unwrap_or("")) {
                        return version;
                    }
                }
            }
        }
        
        // Try query parameter
        if let Some(query) = req.uri().query() {
            for pair in query.split('&') {
                if let Some((key, value)) = pair.split_once('=') {
                    if key == "version" {
                        if let Some(version) = ApiVersion::from_str(value) {
                            return version;
                        }
                    }
                }
            }
        }
        
        // Return default
        self.default_version
    }
    
    /// Get router for version
    pub fn get_router(&self, version: ApiVersion) -> Option<&Router> {
        self.routers.get(&version)
    }
}

/// Version deprecation middleware
pub struct DeprecationMiddleware {
    deprecated_versions: Vec<ApiVersion>,
    sunset_date: Option<String>,
}

impl DeprecationMiddleware {
    pub fn new(deprecated_versions: Vec<ApiVersion>) -> Self {
        Self {
            deprecated_versions,
            sunset_date: None,
        }
    }
    
    pub fn with_sunset_date(mut self, date: String) -> Self {
        self.sunset_date = Some(date);
        self
    }
    
    /// Add deprecation headers to response
    pub fn add_headers(&self, version: ApiVersion, response: &mut OxiditeResponse) {
        if self.deprecated_versions.contains(&version) {
            response.headers_mut().insert(
                "Deprecation",
                "true".parse().unwrap()
            );
            
            if let Some(date) = &self.sunset_date {
                response.headers_mut().insert(
                    "Sunset",
                    date.parse().unwrap()
                );
            }
            
            response.headers_mut().insert(
                "Link",
                format!("</api/docs>; rel=\"deprecation\"").parse().unwrap()
            );
        }
    }
}
