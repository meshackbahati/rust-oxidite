# API Versioning

API versioning allows you to manage changes to your API over time while maintaining backward compatibility. This chapter covers various approaches to API versioning in Oxidite.

## Overview

API versioning strategies include:
- URL-based versioning (e.g., `/api/v1/users`)
- Header-based versioning (e.g., `Accept: application/vnd.api.v1+json`)
- Query parameter versioning (e.g., `?version=1`)
- Media type versioning
- Semantic versioning practices

## URL-Based Versioning

The most common approach is to include the version in the URL path:

```rust
use oxidite::prelude::*;

// V1 API routes
async fn v1_get_users(_req: Request) -> Result<Response> {
    Ok(Response::json(serde_json::json!([
        {"id": 1, "name": "John", "email": "john@example.com"}
    ])))
}

async fn v1_get_user(Path(user_id): Path<u32>) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "id": user_id,
        "name": format!("User {}", user_id),
        "email": format!("user{}@example.com", user_id)
    })))
}

// V2 API routes - with breaking changes
async fn v2_get_users(_req: Request) -> Result<Response> {
    Ok(Response::json(serde_json::json!([
        {
            "id": 1,
            "name": "John",
            "email": "john@example.com",
            "profile": {
                "bio": "Software developer",
                "avatar_url": "https://example.com/avatar.jpg"
            }
        }
    ])))
}

async fn v2_get_user(Path(user_id): Path<u32>) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "id": user_id,
        "name": format!("User {}", user_id),
        "email": format!("user{}@example.com", user_id),
        "profile": {
            "bio": "User bio",
            "avatar_url": format!("https://example.com/avatar/{}.jpg", user_id),
            "preferences": {
                "theme": "light",
                "notifications": true
            }
        }
    })))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    
    // V1 API
    router.get("/api/v1/users", v1_get_users);
    router.get("/api/v1/users/:id", v1_get_user);
    
    // V2 API
    router.get("/api/v2/users", v2_get_users);
    router.get("/api/v2/users/:id", v2_get_user);
    
    Server::new(router)
        .listen("127.0.0.1:3000".parse()?)
        .await
}
```

## Header-Based Versioning

Use HTTP headers to specify the API version:

```rust
use oxidite::prelude::*;

// Middleware to extract version from headers
async fn version_middleware(req: Request, next: Next) -> Result<Response> {
    // Extract version from Accept header
    let version = req.headers()
        .get("accept")
        .and_then(|hv| hv.to_str().ok())
        .and_then(|accept| {
            // Look for version in vendor media type
            // e.g., application/vnd.myapi.v1+json
            if accept.contains("application/vnd.myapi.v") {
                accept.find("v").and_then(|pos| {
                    accept[pos + 1..].chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse::<u32>().ok()
                })
            } else {
                None
            }
        })
        .or_else(|| {
            // Fallback to custom header
            req.headers()
                .get("x-api-version")
                .and_then(|hv| hv.to_str().ok())
                .and_then(|version_str| version_str.parse::<u32>().ok())
        })
        .unwrap_or(1); // Default to v1
    
    // Add version to request extensions
    let mut req = req;
    req.extensions_mut().insert(ApiVersion(version));
    
    next.run(req).await
}

#[derive(Clone)]
struct ApiVersion(u32);

// Route handlers that check the version
async fn get_users_by_version(req: Request) -> Result<Response> {
    if let Some(ApiVersion(version)) = req.extensions().get::<ApiVersion>() {
        match version {
            1 => v1_get_users(req).await,
            2 => v2_get_users(req).await,
            _ => Err(Error::NotImplemented),
        }
    } else {
        v1_get_users(req).await // Default to v1
    }
}

async fn v1_get_users(_req: Request) -> Result<Response> {
    Ok(Response::json(serde_json::json!([
        {"id": 1, "name": "John", "email": "john@example.com"}
    ])))
}

async fn v2_get_users(_req: Request) -> Result<Response> {
    Ok(Response::json(serde_json::json!([
        {
            "id": 1,
            "name": "John",
            "email": "john@example.com",
            "profile": {
                "bio": "Software developer",
                "avatar_url": "https://example.com/avatar.jpg"
            }
        }
    ])))
}
```

## Query Parameter Versioning

Use query parameters to specify the API version:

```rust
use oxidite::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct VersionedQuery {
    version: Option<u32>,
}

async fn versioned_handler(Query(params): Query<VersionedQuery>) -> Result<Response> {
    let version = params.version.unwrap_or(1);
    
    match version {
        1 => v1_response(),
        2 => v2_response(),
        _ => Err(Error::NotImplemented),
    }
}

fn v1_response() -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "data": [
            {"id": 1, "name": "John"}
        ],
        "version": "v1"
    })))
}

fn v2_response() -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "data": [
            {
                "id": 1,
                "name": "John",
                "metadata": {
                    "created_at": "2023-01-01T00:00:00Z",
                    "updated_at": "2023-01-02T00:00:00Z"
                }
            }
        ],
        "version": "v2",
        "pagination": {
            "page": 1,
            "per_page": 10,
            "total": 100
        }
    })))
}

// Alternative: Middleware approach for query versioning
async fn query_version_middleware(req: Request, next: Next) -> Result<Response> {
    // Extract version from query parameters
    let version = req.uri().query()
        .and_then(|q| {
            q.split('&')
             .find(|param| param.starts_with("version="))
             .map(|param| param.split('=').nth(1)?.parse::<u32>().ok())
        })
        .flatten()
        .unwrap_or(1);
    
    // Add version to request extensions
    let mut req = req;
    req.extensions_mut().insert(ApiVersion(version));
    
    next.run(req).await
}
```

## Versioned Models and Serializers

Handle different versions of data models:

```rust
use oxidite::prelude::*;
use serde::{Deserialize, Serialize};

// V1 User Model
#[derive(Serialize, Deserialize)]
pub struct UserV1 {
    pub id: u32,
    pub name: String,
    pub email: String,
}

// V2 User Model - with additional fields
#[derive(Serialize, Deserialize)]
pub struct UserV2 {
    pub id: u32,
    pub name: String,
    pub email: String,
    pub profile: UserProfile,
}

#[derive(Serialize, Deserialize)]
pub struct UserProfile {
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub preferences: UserPreferences,
}

#[derive(Serialize, Deserialize)]
pub struct UserPreferences {
    pub theme: String,
    pub notifications: bool,
}

// Version-aware handler
async fn get_user_versioned(
    Path(user_id): Path<u32>,
    req: Request
) -> Result<Response> {
    // Fetch user from database (simplified)
    let user = fetch_user_from_db(user_id).await?;
    
    if let Some(ApiVersion(version)) = req.extensions().get::<ApiVersion>() {
        match version {
            1 => {
                let v1_user = UserV1 {
                    id: user.id,
                    name: user.name,
                    email: user.email,
                };
                Ok(Response::json(v1_user))
            }
            2 => {
                let v2_user = UserV2 {
                    id: user.id,
                    name: user.name,
                    email: user.email,
                    profile: UserProfile {
                        bio: Some("Default bio".to_string()),
                        avatar_url: Some(format!("https://example.com/avatar/{}.jpg", user.id)),
                        preferences: UserPreferences {
                            theme: "light".to_string(),
                            notifications: true,
                        },
                    },
                };
                Ok(Response::json(v2_user))
            }
            _ => Err(Error::NotImplemented),
        }
    } else {
        // Default to v1
        let v1_user = UserV1 {
            id: user.id,
            name: user.name,
            email: user.email,
        };
        Ok(Response::json(v1_user))
    }
}

// Simulated database fetch
async fn fetch_user_from_db(id: u32) -> Result<UserV2> {
    Ok(UserV2 {
        id,
        name: format!("User {}", id),
        email: format!("user{}@example.com", id),
        profile: UserProfile {
            bio: Some("Sample bio".to_string()),
            avatar_url: Some(format!("https://example.com/avatar/{}.jpg", id)),
            preferences: UserPreferences {
                theme: "light".to_string(),
                notifications: true,
            },
        },
    })
}

// Convert between versions
impl UserV2 {
    pub fn to_v1(self) -> UserV1 {
        UserV1 {
            id: self.id,
            name: self.name,
            email: self.email,
        }
    }
}

impl UserV1 {
    pub fn to_v2(self) -> UserV2 {
        UserV2 {
            id: self.id,
            name: self.name,
            email: self.email,
            profile: UserProfile {
                bio: None,
                avatar_url: None,
                preferences: UserPreferences {
                    theme: "light".to_string(),
                    notifications: false,
                },
            },
        }
    }
}
```

## Version Negotiation

Implement automatic version negotiation:

```rust
use oxidite::prelude::*;

#[derive(Clone)]
struct ApiVersionManager {
    supported_versions: std::collections::HashSet<u32>,
    default_version: u32,
}

impl ApiVersionManager {
    fn new() -> Self {
        let mut supported = std::collections::HashSet::new();
        supported.insert(1);
        supported.insert(2);
        supported.insert(3);
        
        Self {
            supported_versions: supported,
            default_version: 1,
        }
    }
    
    fn negotiate_version(&self, req: &Request) -> u32 {
        // Try header version first
        if let Some(version) = self.extract_header_version(req) {
            if self.supported_versions.contains(&version) {
                return version;
            }
        }
        
        // Try query parameter
        if let Some(version) = self.extract_query_version(req) {
            if self.supported_versions.contains(&version) {
                return version;
            }
        }
        
        // Fall back to default
        self.default_version
    }
    
    fn extract_header_version(&self, req: &Request) -> Option<u32> {
        req.headers()
            .get("accept")
            .and_then(|hv| hv.to_str().ok())
            .and_then(|accept| {
                if accept.contains("application/vnd.myapi.v") {
                    accept.find("v").and_then(|pos| {
                        accept[pos + 1..].chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse::<u32>().ok()
                    })
                } else {
                    None
                }
            })
            .or_else(|| {
                req.headers()
                    .get("x-api-version")
                    .and_then(|hv| hv.to_str().ok())
                    .and_then(|version_str| version_str.parse::<u32>().ok())
            })
    }
    
    fn extract_query_version(&self, req: &Request) -> Option<u32> {
        req.uri().query()
            .and_then(|q| {
                q.split('&')
                 .find(|param| param.starts_with("version="))
                 .map(|param| param.split('=').nth(1)?.parse::<u32>().ok())
            })
            .flatten()
    }
}

// Version negotiation middleware
async fn version_negotiation_middleware(
    req: Request,
    next: Next,
    State(version_manager): State<Arc<ApiVersionManager>>
) -> Result<Response> {
    let negotiated_version = version_manager.negotiate_version(&req);
    
    let mut req = req;
    req.extensions_mut().insert(ApiVersion(negotiated_version));
    
    // Add version to response headers
    let mut response = next.run(req).await?;
    response.headers_mut().insert(
        "X-API-Version",
        format!("{}", negotiated_version).parse().unwrap()
    );
    
    Ok(response)
}

// Get version info endpoint
async fn api_version_info(State(version_manager): State<Arc<ApiVersionManager>>) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "current_version": version_manager.default_version,
        "supported_versions": version_manager.supported_versions.iter().collect::<Vec<_>>(),
        "latest_stable": 2,
        "deprecation_warning": null
    })))
}
```

## Deprecation and Sunset Policies

Manage deprecated versions:

```rust
use oxidite::prelude::*;

#[derive(Clone)]
struct VersionDeprecationPolicy {
    deprecated_versions: std::collections::HashMap<u32, DeprecationInfo>,
}

#[derive(Clone)]
struct DeprecationInfo {
    deprecation_date: chrono::DateTime<chrono::Utc>,
    sunset_date: chrono::DateTime<chrono::Utc>,
    migration_guide_url: String,
    alternative_endpoints: Vec<String>,
}

impl VersionDeprecationPolicy {
    fn new() -> Self {
        let mut deprecated = std::collections::HashMap::new();
        
        // Example: deprecate v1 on 2024-01-01, sunset on 2024-07-01
        deprecated.insert(1, DeprecationInfo {
            deprecation_date: chrono::DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap().into(),
            sunset_date: chrono::DateTime::parse_from_rfc3339("2024-07-01T00:00:00Z").unwrap().into(),
            migration_guide_url: "https://docs.example.com/v1-to-v2-migration".to_string(),
            alternative_endpoints: vec!["/api/v2/users".to_string()],
        });
        
        Self {
            deprecated_versions: deprecated,
        }
    }
    
    fn check_deprecation(&self, version: u32) -> Option<&DeprecationInfo> {
        self.deprecated_versions.get(&version)
    }
    
    fn is_sunset(&self, version: u32) -> bool {
        if let Some(info) = self.check_deprecation(version) {
            chrono::Utc::now() > info.sunset_date
        } else {
            false
        }
    }
}

// Deprecation middleware
async fn deprecation_middleware(
    req: Request,
    next: Next,
    State(policy): State<Arc<VersionDeprecationPolicy>>
) -> Result<Response> {
    if let Some(ApiVersion(version)) = req.extensions().get::<ApiVersion>() {
        if policy.is_sunset(*version) {
            return Err(Error::Gone("This API version has been sunset. Please upgrade to a newer version.".to_string()));
        }
        
        if let Some(deprecation_info) = policy.check_deprecation(*version) {
            let mut response = next.run(req).await?;
            
            // Add deprecation headers
            response.headers_mut().insert(
                "X-API-Deprecated",
                "true".parse().unwrap()
            );
            
            response.headers_mut().insert(
                "X-API-Deprecation-Date",
                deprecation_info.deprecation_date.to_rfc3339().parse().unwrap()
            );
            
            response.headers_mut().insert(
                "X-API-Sunset-Date",
                deprecation_info.sunset_date.to_rfc3339().parse().unwrap()
            );
            
            response.headers_mut().insert(
                "X-API-Migration-Guide",
                deprecation_info.migration_guide_url.parse().unwrap()
            );
            
            return Ok(response);
        }
    }
    
    next.run(req).await
}
```

## Content Negotiation

Handle different content types based on version:

```rust
use oxidite::prelude::*;

// Content negotiation middleware
async fn content_negotiation_middleware(req: Request, next: Next) -> Result<Response> {
    // Determine response format based on Accept header and version
    let accept_header = req.headers()
        .get("accept")
        .and_then(|hv| hv.to_str().ok())
        .unwrap_or("*/*");
    
    let mut response = next.run(req).await?;
    
    // Set content type based on requested format
    if accept_header.contains("application/json") {
        response.headers_mut().insert(
            "Content-Type",
            "application/json".parse().unwrap()
        );
    } else if accept_header.contains("text/html") {
        response.headers_mut().insert(
            "Content-Type",
            "text/html".parse().unwrap()
        );
    } else {
        response.headers_mut().insert(
            "Content-Type",
            "application/json".parse().unwrap()
        );
    }
    
    Ok(response)
}

// Version-specific content types
async fn versioned_content_handler(req: Request) -> Result<Response> {
    let accept_header = req.headers()
        .get("accept")
        .and_then(|hv| hv.to_str().ok())
        .unwrap_or("*/*");
    
    if let Some(ApiVersion(version)) = req.extensions().get::<ApiVersion>() {
        match (version, accept_header) {
            (1, accept) if accept.contains("application/vnd.myapi.v1+json") => {
                // V1 JSON response
                Ok(Response::json(serde_json::json!({
                    "users": [
                        {"id": 1, "name": "John"}
                    ]
                })))
            }
            (2, accept) if accept.contains("application/vnd.myapi.v2+json") => {
                // V2 JSON response with more fields
                Ok(Response::json(serde_json::json!({
                    "data": {
                        "users": [
                            {
                                "id": 1,
                                "name": "John",
                                "meta": {
                                    "total": 1
                                }
                            }
                        ]
                    },
                    "links": {
                        "self": "/api/v2/users",
                        "next": "/api/v2/users?page=2"
                    }
                })))
            }
            _ => {
                // Fallback response
                Ok(Response::json(serde_json::json!({
                    "error": "Unsupported version or content type"
                })))
            }
        }
    } else {
        // Default response
        Ok(Response::json(serde_json::json!({
            "users": [
                {"id": 1, "name": "John"}
            ]
        })))
    }
}
```

## Version-Specific Middleware

Apply different middleware based on API version:

```rust
use oxidite::prelude::*;

// Version-specific rate limiting
async fn v1_rate_limit_middleware(req: Request, next: Next) -> Result<Response> {
    // V1 has stricter limits
    let max_requests = 100; // per hour for v1
    check_rate_limit(&req, max_requests, "v1")?;
    next.run(req).await
}

async fn v2_rate_limit_middleware(req: Request, next: Next) -> Result<Response> {
    // V2 has higher limits
    let max_requests = 1000; // per hour for v2
    check_rate_limit(&req, max_requests, "v2")?;
    next.run(req).await
}

fn check_rate_limit(_req: &Request, _limit: usize, _version: &str) -> Result<()> {
    // Implementation would check rate limits
    Ok(())
}

// Version-aware router
async fn versioned_router(req: Request) -> Result<Response> {
    if let Some(ApiVersion(version)) = req.extensions().get::<ApiVersion>() {
        match version {
            1 => {
                // Apply v1-specific middleware and handlers
                v1_rate_limit_middleware(req, Next::new(|req| async {
                    // V1 handler
                    Ok(Response::json(serde_json::json!({"version": "v1"})))
                })).await
            }
            2 => {
                // Apply v2-specific middleware and handlers
                v2_rate_limit_middleware(req, Next::new(|req| async {
                    // V2 handler
                    Ok(Response::json(serde_json::json!({"version": "v2"})))
                })).await
            }
            _ => Err(Error::NotImplemented),
        }
    } else {
        // Default to v1
        v1_rate_limit_middleware(req, Next::new(|req| async {
            Ok(Response::json(serde_json::json!({"version": "v1", "default": true})))
        })).await
    }
}

// Next type for middleware chaining
struct Next<F> {
    handler: F,
}

impl<F> Next<F> {
    fn new(handler: F) -> Self {
        Self { handler }
    }
    
    async fn run(self, req: Request) -> Result<Response> {
        (self.handler)(req).await
    }
}
```

## Testing Versioned APIs

Write tests for versioned APIs:

```rust
use oxidite::prelude::*;
use oxidite_testing::{TestServer, RequestBuilder};

#[cfg(test)]
mod version_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_v1_api() {
        let server = TestServer::new(|router| {
            router.get("/api/v1/users", v1_get_users);
        }).await;
        
        let response = server
            .get("/api/v1/users")
            .header("Accept", "application/json")
            .send()
            .await;
        
        assert_eq!(response.status(), 200);
        
        let json: serde_json::Value = response.json().await;
        assert!(json.as_array().unwrap().first().unwrap()["email"].is_string());
    }
    
    #[tokio::test]
    async fn test_v2_api() {
        let server = TestServer::new(|router| {
            router.get("/api/v2/users", v2_get_users);
        }).await;
        
        let response = server
            .get("/api/v2/users")
            .header("Accept", "application/json")
            .send()
            .await;
        
        assert_eq!(response.status(), 200);
        
        let json: serde_json::Value = response.json().await;
        assert!(json.as_array().unwrap().first().unwrap()["profile"].isObject());
    }
    
    #[tokio::test]
    async fn test_header_versioning() {
        let server = TestServer::new(|router| {
            router.get("/users")
                .middleware(version_middleware)
                .handler(get_users_by_version);
        }).await;
        
        let response = server
            .get("/users")
            .header("Accept", "application/vnd.myapi.v2+json")
            .send()
            .await;
        
        assert_eq!(response.status(), 200);
        
        let json: serde_json::Value = response.json().await;
        assert!(json.as_array().unwrap().first().unwrap()["profile"].isObject());
    }
    
    #[tokio::test]
    async fn test_deprecated_version() {
        let policy = Arc::new(VersionDeprecationPolicy::new());
        let server = TestServer::new(move |router| {
            let policy_clone = policy.clone();
            router.get("/users")
                .with_state(policy_clone)
                .middleware(deprecation_middleware)
                .handler(|_| async { Ok(Response::json(serde_json::json!({"test": true}))) });
        }).await;
        
        let response = server
            .get("/users")
            .header("X-API-Version", "1")
            .send()
            .await;
        
        assert_eq!(response.status(), 200);
        assert!(response.headers().get("X-API-Deprecated").is_some());
    }
}
```

## Migration Strategies

Plan for API migrations:

```rust
use oxidite::prelude::*;

// Migration guide endpoint
async fn migration_guide(_req: Request) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "v1_to_v2": {
            "breaking_changes": [
                "User objects now include profile information",
                "Response format changed to include metadata",
                "New pagination structure"
            ],
            "migration_steps": [
                "Update client libraries",
                "Modify data processing logic",
                "Update error handling",
                "Test with v2 endpoints"
            ],
            "timeline": {
                "deprecation_date": "2024-01-01",
                "sunset_date": "2024-07-01",
                "recommended_action": "Migrate to v2 before deprecation date"
            }
        }
    })))
}

// Feature flags for gradual rollout
#[derive(Clone)]
struct FeatureFlags {
    enabled_features: std::collections::HashSet<String>,
}

impl FeatureFlags {
    fn new() -> Self {
        let mut features = std::collections::HashSet::new();
        features.insert("new_user_format".to_string());
        features.insert("enhanced_pagination".to_string());
        
        Self {
            enabled_features: features,
        }
    }
    
    fn is_enabled(&self, feature: &str) -> bool {
        self.enabled_features.contains(feature)
    }
}

// Version with feature flags
async fn feature_flagged_handler(
    req: Request,
    State(flags): State<Arc<FeatureFlags>>
) -> Result<Response> {
    if let Some(ApiVersion(version)) = req.extensions().get::<ApiVersion>() {
        match version {
            2 => {
                let mut response_data = serde_json::json!({
                    "users": [
                        {
                            "id": 1,
                            "name": "John"
                        }
                    ]
                });
                
                // Conditionally add features based on flags
                if flags.is_enabled("enhanced_pagination") {
                    response_data["pagination"] = serde_json::json!({
                        "page": 1,
                        "per_page": 10,
                        "total": 100,
                        "pages": 10
                    });
                }
                
                if flags.is_enabled("new_user_format") {
                    if let Some(users) = response_data["users"].as_array_mut() {
                        for user in users {
                            user["profile"] = serde_json::json!({
                                "bio": "Software developer",
                                "avatar_url": "https://example.com/avatar.jpg"
                            });
                        }
                    }
                }
                
                Ok(Response::json(response_data))
            }
            _ => v1_response(), // Default to v1 behavior
        }
    } else {
        v1_response()
    }
}
```

## Summary

API versioning in Oxidite supports multiple strategies:

- **URL-based**: `/api/v1/resource` (most common)
- **Header-based**: `Accept: application/vnd.api.v1+json`
- **Query parameter**: `?version=1`
- **Media type versioning**: Custom content types

Best practices include:
- Clear deprecation policies with advance notice
- Automated version negotiation
- Proper error handling for unsupported versions
- Comprehensive testing across versions
- Gradual migration strategies
- Feature flags for controlled rollouts

Choose the versioning strategy that best fits your API consumers' needs and your team's capabilities.