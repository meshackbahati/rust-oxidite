# Plugins

Plugins in Oxidite provide a way to extend the framework's functionality with modular, reusable components. This chapter covers creating, configuring, and using plugins in your Oxidite applications.

## Overview

Oxidite plugins allow you to:
- Extend framework functionality
- Share common features across applications
- Create modular, reusable components
- Hook into framework lifecycle events
- Customize request/response processing

## Plugin Architecture

The plugin system is built around traits and hooks:

```rust
use oxidite::prelude::*;
use std::sync::Arc;

/// Core plugin trait that all plugins must implement
#[async_trait::async_trait]
pub trait Plugin: Send + Sync {
    /// Plugin name for identification
    fn name(&self) -> &str;
    
    /// Plugin version
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    /// Initialize the plugin
    async fn initialize(&self, _router: &mut Router) -> Result<()> {
        Ok(())
    }
    
    /// Called before request processing
    async fn before_request(&self, _req: &mut Request) -> Result<()> {
        Ok(())
    }
    
    /// Called after request processing
    async fn after_request(&self, _req: &Request, _resp: &mut Response) -> Result<()> {
        Ok(())
    }
    
    /// Called on application shutdown
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

/// Plugin manager to handle multiple plugins
pub struct PluginManager {
    plugins: Vec<Arc<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }
    
    pub fn register_plugin(&mut self, plugin: Arc<dyn Plugin>) {
        self.plugins.push(plugin);
    }
    
    pub async fn initialize_plugins(&self, router: &mut Router) -> Result<()> {
        for plugin in &self.plugins {
            plugin.initialize(router).await?;
        }
        Ok(())
    }
    
    pub async fn before_request(&self, req: &mut Request) -> Result<()> {
        for plugin in &self.plugins {
            plugin.before_request(req).await?;
        }
        Ok(())
    }
    
    pub async fn after_request(&self, req: &Request, resp: &mut Response) -> Result<()> {
        for plugin in &self.plugins {
            plugin.after_request(req, resp).await?;
        }
        Ok(())
    }
    
    pub async fn shutdown_plugins(&self) -> Result<()> {
        for plugin in &self.plugins {
            plugin.shutdown().await?;
        }
        Ok(())
    }
}
```

## Creating a Basic Plugin

Create your first plugin:

```rust
use oxidite::prelude::*;
use std::sync::Arc;

/// A simple logging plugin
pub struct LoggingPlugin {
    log_level: String,
}

impl LoggingPlugin {
    pub fn new(log_level: &str) -> Self {
        Self {
            log_level: log_level.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl Plugin for LoggingPlugin {
    fn name(&self) -> &str {
        "logging"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    async fn before_request(&self, req: &mut Request) -> Result<()> {
        println!("[{}] {} {}", self.log_level, req.method(), req.uri());
        Ok(())
    }
    
    async fn after_request(&self, _req: &Request, resp: &mut Response) -> Result<()> {
        println!("[{}] Response: {}", self.log_level, resp.status());
        Ok(())
    }
}

// Usage example
#[tokio::main]
async fn main() -> Result<()> {
    let mut plugin_manager = PluginManager::new();
    
    // Register the logging plugin
    plugin_manager.register_plugin(Arc::new(LoggingPlugin::new("INFO")));
    
    let mut router = Router::new();
    
    // Initialize plugins
    plugin_manager.initialize_plugins(&mut router).await?;
    
    // Add routes
    router.get("/", |_req| async { Ok(Response::text("Hello, World!".to_string())) });
    
    // Create server with plugin middleware
    let server = Server::new(router)
        .with_plugin_manager(plugin_manager);
    
    server.listen("127.0.0.1:3000".parse()?).await
}

// Extend Server to support plugins
impl Server {
    pub fn with_plugin_manager(mut self, plugin_manager: PluginManager) -> Self {
        self.plugin_manager = Some(plugin_manager);
        self
    }
}
```

## Middleware Plugins

Create plugins that act as middleware:

```rust
use oxidite::prelude::*;
use std::sync::Arc;

/// CORS plugin for handling cross-origin requests
pub struct CorsPlugin {
    allowed_origins: Vec<String>,
    allowed_methods: Vec<String>,
    allowed_headers: Vec<String>,
}

impl CorsPlugin {
    pub fn new() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "OPTIONS".to_string(),
            ],
            allowed_headers: vec![
                "Content-Type".to_string(),
                "Authorization".to_string(),
                "X-Requested-With".to_string(),
            ],
        }
    }
    
    pub fn with_origins(mut self, origins: Vec<&str>) -> Self {
        self.allowed_origins = origins.iter().map(|s| s.to_string()).collect();
        self
    }
    
    pub fn with_methods(mut self, methods: Vec<&str>) -> Self {
        self.allowed_methods = methods.iter().map(|s| s.to_string()).collect();
        self
    }
    
    pub fn with_headers(mut self, headers: Vec<&str>) -> Self {
        self.allowed_headers = headers.iter().map(|s| s.to_string()).collect();
        self
    }
}

#[async_trait::async_trait]
impl Plugin for CorsPlugin {
    fn name(&self) -> &str {
        "cors"
    }
    
    async fn after_request(&self, _req: &Request, resp: &mut Response) -> Result<()> {
        // Handle preflight requests
        if _req.method() == http::Method::OPTIONS {
            *resp = Response::ok();
        }
        
        // Add CORS headers
        resp.headers_mut().insert(
            "Access-Control-Allow-Origin",
            self.allowed_origins.join(", ").parse().unwrap()
        );
        
        resp.headers_mut().insert(
            "Access-Control-Allow-Methods",
            self.allowed_methods.join(", ").parse().unwrap()
        );
        
        resp.headers_mut().insert(
            "Access-Control-Allow-Headers",
            self.allowed_headers.join(", ").parse().unwrap()
        );
        
        Ok(())
    }
}

// Rate limiting plugin
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

pub struct RateLimitPlugin {
    max_requests: u32,
    window_duration: Duration,
    requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
}

impl RateLimitPlugin {
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self {
            max_requests,
            window_duration: Duration::from_secs(window_seconds),
            requests: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl Plugin for RateLimitPlugin {
    fn name(&self) -> &str {
        "rate_limit"
    }
    
    async fn before_request(&self, req: &mut Request) -> Result<()> {
        // Extract client identifier (IP address)
        let client_id = req.headers()
            .get("x-forwarded-for")
            .and_then(|hv| hv.to_str().ok())
            .unwrap_or("unknown")
            .to_string();
        
        let now = Instant::now();
        let window_start = now - self.window_duration;
        
        {
            let mut requests = self.requests.write().await;
            
            // Clean old requests
            if let Some(times) = requests.get_mut(&client_id) {
                times.retain(|time| *time > window_start);
            }
            
            // Check rate limit
            let current_count = requests
                .entry(client_id.clone())
                .or_insert_with(Vec::new)
                .len();
            
            if current_count >= self.max_requests as usize {
                return Err(Error::TooManyRequests);
            }
            
            // Record request
            requests.get_mut(&client_id).unwrap().push(now);
        }
        
        Ok(())
    }
}
```

## Database Plugins

Create plugins that integrate with databases:

```rust
use oxidite::prelude::*;
use std::sync::Arc;

/// Database connection plugin
pub struct DatabasePlugin {
    connection_string: String,
    pool_size: usize,
}

impl DatabasePlugin {
    pub fn new(connection_string: &str, pool_size: usize) -> Self {
        Self {
            connection_string: connection_string.to_string(),
            pool_size,
        }
    }
}

#[async_trait::async_trait]
impl Plugin for DatabasePlugin {
    fn name(&self) -> &str {
        "database"
    }
    
    async fn initialize(&self, _router: &mut Router) -> Result<()> {
        // Initialize database connection pool
        // This would typically connect to the actual database
        println!("Initializing database connection to: {}", self.connection_string);
        
        // Store connection in router state
        // _router.with_state(Arc::new(DatabaseConnection::new(&self.connection_string)?));
        
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<()> {
        // Close database connections
        println!("Closing database connections");
        Ok(())
    }
}

// Example database connection wrapper
pub struct DatabaseConnection {
    // Connection pool or client
}

impl DatabaseConnection {
    pub fn new(_connection_string: &str) -> Result<Self> {
        // In a real implementation, this would create the actual connection
        Ok(Self {})
    }
}

// Migration plugin
pub struct MigrationPlugin {
    migrations_path: String,
}

impl MigrationPlugin {
    pub fn new(migrations_path: &str) -> Self {
        Self {
            migrations_path: migrations_path.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl Plugin for MigrationPlugin {
    fn name(&self) -> &str {
        "migrations"
    }
    
    async fn initialize(&self, _router: &mut Router) -> Result<()> {
        println!("Running migrations from: {}", self.migrations_path);
        // Run pending migrations
        Ok(())
    }
}
```

## Authentication Plugins

Create authentication plugins:

```rust
use oxidite::prelude::*;
use std::sync::Arc;

/// JWT authentication plugin
pub struct JwtAuthPlugin {
    secret: String,
    expiration: std::time::Duration,
}

impl JwtAuthPlugin {
    pub fn new(secret: &str, expiration_hours: u64) -> Self {
        Self {
            secret: secret.to_string(),
            expiration: std::time::Duration::from_secs(expiration_hours * 3600),
        }
    }
}

#[async_trait::async_trait]
impl Plugin for JwtAuthPlugin {
    fn name(&self) -> &str {
        "jwt_auth"
    }
    
    async fn before_request(&self, req: &mut Request) -> Result<()> {
        // Check for JWT token in Authorization header
        let auth_header = req.headers()
            .get("authorization")
            .and_then(|hv| hv.to_str().ok());
        
        if let Some(auth) = auth_header {
            if auth.starts_with("Bearer ") {
                let token = auth.trim_start_matches("Bearer ").trim();
                
                if !self.verify_token(token).await {
                    return Err(Error::Unauthorized("Invalid token".to_string()));
                }
            } else {
                return Err(Error::Unauthorized("Invalid authorization format".to_string()));
            }
        } else {
            // For public endpoints, this might be acceptable
            // Return Ok(()) to continue processing
        }
        
        Ok(())
    }
}

impl JwtAuthPlugin {
    async fn verify_token(&self, _token: &str) -> bool {
        // In a real implementation, verify the JWT token
        // This is a placeholder
        _token == "valid_token"
    }
}

/// API Key authentication plugin
pub struct ApiKeyPlugin {
    valid_keys: Vec<String>,
}

impl ApiKeyPlugin {
    pub fn new(valid_keys: Vec<&str>) -> Self {
        Self {
            valid_keys: valid_keys.iter().map(|k| k.to_string()).collect(),
        }
    }
}

#[async_trait::async_trait]
impl Plugin for ApiKeyPlugin {
    fn name(&self) -> &str {
        "api_key_auth"
    }
    
    async fn before_request(&self, req: &mut Request) -> Result<()> {
        // Check for API key in header or query parameter
        let api_key = req.headers()
            .get("x-api-key")
            .and_then(|hv| hv.to_str().ok())
            .or_else(|| {
                req.uri().query().and_then(|q| {
                    q.split('&')
                     .find(|param| param.starts_with("api_key="))
                     .map(|param| param.split('=').nth(1).unwrap_or(""))
                })
            });
        
        if let Some(key) = api_key {
            if self.valid_keys.contains(&key.to_string()) {
                // Add user info to request extensions
                req.extensions_mut().insert(ApiKeyUser {
                    key: key.to_string(),
                    permissions: vec!["read".to_string(), "write".to_string()],
                });
                return Ok(());
            }
        }
        
        Err(Error::Unauthorized("Invalid or missing API key".to_string()))
    }
}

#[derive(Clone)]
struct ApiKeyUser {
    key: String,
    permissions: Vec<String>,
}
```

## Template Engine Plugins

Create plugins that integrate with template engines:

```rust
use oxidite::prelude::*;
use std::sync::Arc;

/// Template engine plugin
pub struct TemplatePlugin {
    templates_dir: String,
    cache_enabled: bool,
}

impl TemplatePlugin {
    pub fn new(templates_dir: &str) -> Self {
        Self {
            templates_dir: templates_dir.to_string(),
            cache_enabled: true,
        }
    }
    
    pub fn with_cache(mut self, enabled: bool) -> Self {
        self.cache_enabled = enabled;
        self
    }
}

#[async_trait::async_trait]
impl Plugin for TemplatePlugin {
    fn name(&self) -> &str {
        "template_engine"
    }
    
    async fn initialize(&self, router: &mut Router) -> Result<()> {
        // Initialize template engine
        let mut template_engine = oxidite_template::TemplateEngine::new();
        
        // Load templates from directory
        // This would scan the templates directory and load all templates
        println!("Loading templates from: {}", self.templates_dir);
        
        // Store template engine in router state
        router.with_state(Arc::new(template_engine));
        
        Ok(())
    }
    
    async fn after_request(&self, _req: &Request, resp: &mut Response) -> Result<()> {
        // Template rendering happens in route handlers
        // This plugin primarily manages the template engine
        Ok(())
    }
}
```

## Plugin Configuration

Configure plugins with options:

```rust
use oxidite::prelude::*;
use serde::Deserialize;

/// Configuration for plugins
#[derive(Deserialize, Clone)]
pub struct PluginConfig {
    pub enabled: bool,
    pub settings: std::collections::HashMap<String, serde_json::Value>,
}

impl PluginConfig {
    pub fn get<T>(&self, key: &str) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.settings.get(key)
            .and_then(|value| serde_json::from_value(value.clone()).ok())
    }
    
    pub fn get_or<T>(&self, key: &str, default: T) -> T
    where
        T: serde::de::DeserializeOwned,
    {
        self.get(key).unwrap_or(default)
    }
}

/// Configurable plugin base
pub struct ConfigurablePlugin {
    name: String,
    config: PluginConfig,
}

impl ConfigurablePlugin {
    pub fn new(name: &str, config: PluginConfig) -> Self {
        Self {
            name: name.to_string(),
            config,
        }
    }
    
    pub fn get_config(&self) -> &PluginConfig {
        &self.config
    }
}

#[async_trait::async_trait]
impl Plugin for ConfigurablePlugin {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn initialize(&self, _router: &mut Router) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }
        
        println!("Initializing configurable plugin: {}", self.name);
        Ok(())
    }
}

// Example configuration file
/*
plugins:
  cors:
    enabled: true
    settings:
      allowed_origins: ["http://localhost:3000", "https://myapp.com"]
      allowed_methods: ["GET", "POST", "PUT", "DELETE"]
  
  rate_limit:
    enabled: true
    settings:
      max_requests: 100
      window_seconds: 60
  
  jwt_auth:
    enabled: true
    settings:
      secret: "my_secret_key"
      expiration_hours: 24
*/
```

## Plugin Registry

Create a registry for managing plugins:

```rust
use oxidite::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

/// Plugin registry to manage plugin lifecycle
pub struct PluginRegistry {
    plugins: HashMap<String, Arc<dyn Plugin>>,
    initialized: bool,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            initialized: false,
        }
    }
    
    pub fn register(&mut self, plugin: Arc<dyn Plugin>) -> Result<()> {
        let name = plugin.name().to_string();
        
        if self.plugins.contains_key(&name) {
            return Err(Error::Server(format!("Plugin '{}' already registered", name)));
        }
        
        self.plugins.insert(name, plugin);
        Ok(())
    }
    
    pub fn get(&self, name: &str) -> Option<&Arc<dyn Plugin>> {
        self.plugins.get(name)
    }
    
    pub async fn initialize_all(&mut self, router: &mut Router) -> Result<()> {
        if self.initialized {
            return Ok(());
        }
        
        for plugin in self.plugins.values() {
            plugin.initialize(router).await?;
        }
        
        self.initialized = true;
        Ok(())
    }
    
    pub async fn shutdown_all(&self) -> Result<()> {
        for plugin in self.plugins.values() {
            plugin.shutdown().await?;
        }
        Ok(())
    }
    
    pub fn list_plugins(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }
}

// Plugin factory for creating plugins from configuration
pub struct PluginFactory;

impl PluginFactory {
    pub fn create_from_config(config: &PluginConfig) -> Result<Vec<Arc<dyn Plugin>>> {
        let mut plugins = Vec::new();
        
        // Example: create CORS plugin if configured
        if config.enabled {
            // In a real implementation, this would check the plugin type
            // and create the appropriate plugin instance
        }
        
        Ok(plugins)
    }
    
    pub fn create_cors_plugin(settings: &serde_json::Value) -> Result<Arc<dyn Plugin>> {
        let cors_plugin = CorsPlugin::new()
            .with_origins(
                settings.get("allowed_origins")
                    .and_then(|origins| origins.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
                    .unwrap_or_else(|| vec!["*"])
            );
        
        Ok(Arc::new(cors_plugin))
    }
    
    pub fn create_rate_limit_plugin(settings: &serde_json::Value) -> Result<Arc<dyn Plugin>> {
        let max_requests = settings.get("max_requests")
            .and_then(|v| v.as_u64())
            .unwrap_or(100) as u32;
            
        let window_seconds = settings.get("window_seconds")
            .and_then(|v| v.as_u64())
            .unwrap_or(60);
        
        Ok(Arc::new(RateLimitPlugin::new(max_requests, window_seconds)))
    }
}
```

## Plugin Dependencies

Handle plugin dependencies and ordering:

```rust
use oxidite::prelude::*;
use std::sync::Arc;

/// Plugin with dependencies
pub struct DependencyAwarePlugin {
    name: String,
    dependencies: Vec<String>,
    plugin: Arc<dyn Plugin>,
}

impl DependencyAwarePlugin {
    pub fn new(name: &str, plugin: Arc<dyn Plugin>, dependencies: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            dependencies,
            plugin,
        }
    }
    
    pub fn get_dependencies(&self) -> &[String] {
        &self.dependencies
    }
    
    pub fn get_plugin(&self) -> &Arc<dyn Plugin> {
        &self.plugin
    }
}

/// Topological sorter for plugin dependencies
pub struct PluginDependencySorter;

impl PluginDependencySorter {
    pub fn sort_plugins(plugins: Vec<DependencyAwarePlugin>) -> Result<Vec<DependencyAwarePlugin>> {
        let mut sorted = Vec::new();
        let mut remaining: Vec<_> = plugins.into_iter().enumerate().collect();
        let mut processed = std::collections::HashSet::new();
        
        while !remaining.is_empty() {
            let mut progress = false;
            
            let mut i = 0;
            while i < remaining.len() {
                let (_, plugin) = &remaining[i];
                
                // Check if all dependencies are satisfied
                let all_deps_satisfied = plugin.get_dependencies()
                    .iter()
                    .all(|dep| processed.contains(dep));
                
                if all_deps_satisfied {
                    let (_, plugin) = remaining.remove(i);
                    sorted.push(plugin);
                    processed.insert(plugin.name.to_string());
                    progress = true;
                } else {
                    i += 1;
                }
            }
            
            if !progress && !remaining.is_empty() {
                return Err(Error::Server("Circular dependency detected in plugins".to_string()));
            }
        }
        
        Ok(sorted)
    }
}

// Plugin with explicit dependency example
pub struct DatabaseDependentPlugin {
    db_plugin_name: String,
}

#[async_trait::async_trait]
impl Plugin for DatabaseDependentPlugin {
    fn name(&self) -> &str {
        "db_dependent"
    }
    
    async fn initialize(&self, router: &mut Router) -> Result<()> {
        // This plugin expects a database connection to be available
        // It would access the database connection from router state
        println!("Initializing plugin that depends on database");
        Ok(())
    }
}

// Create a dependency-aware version
pub fn create_db_dependent_plugin() -> DependencyAwarePlugin {
    DependencyAwarePlugin::new(
        "db_dependent",
        Arc::new(DatabaseDependentPlugin {
            db_plugin_name: "database".to_string(),
        }),
        vec!["database".to_string()]
    )
}
```

## Plugin Marketplace Concept

Concept for a plugin marketplace:

```rust
use oxidite::prelude::*;
use std::sync::Arc;

/// Plugin manifest for distribution
#[derive(serde::Deserialize, serde::Serialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub dependencies: Vec<PluginDependency>,
    pub hooks: Vec<String>, // Events the plugin hooks into
    pub config_schema: Option<serde_json::Value>, // JSON Schema for configuration
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct PluginDependency {
    pub name: String,
    pub version_requirement: String,
}

/// Plugin loader for external plugins
pub struct PluginLoader {
    plugin_dirs: Vec<String>,
}

impl PluginLoader {
    pub fn new(plugin_dirs: Vec<String>) -> Self {
        Self { plugin_dirs }
    }
    
    pub async fn load_external_plugin(&self, name: &str) -> Result<Arc<dyn Plugin>> {
        // In a real implementation, this would:
        // 1. Locate the plugin file in plugin directories
        // 2. Load the dynamic library (if compiled as dylib)
        // 3. Validate the plugin manifest
        // 4. Instantiate the plugin
        
        // For now, return a dummy plugin
        Ok(Arc::new(DummyPlugin::new(name)))
    }
    
    pub fn validate_manifest(&self, manifest: &PluginManifest) -> Result<()> {
        // Validate plugin manifest
        if manifest.name.is_empty() {
            return Err(Error::Server("Plugin name is required".to_string()));
        }
        
        if manifest.version.is_empty() {
            return Err(Error::Server("Plugin version is required".to_string()));
        }
        
        Ok(())
    }
}

// Dummy plugin for demonstration
struct DummyPlugin {
    name: String,
}

impl DummyPlugin {
    fn new(name: &str) -> Self {
        Self { name: name.to_string() }
    }
}

#[async_trait::async_trait]
impl Plugin for DummyPlugin {
    fn name(&self) -> &str {
        &self.name
    }
}
```

## Testing Plugins

Test your plugins properly:

```rust
use oxidite::prelude::*;
use oxidite_testing::TestServer;

#[cfg(test)]
mod plugin_tests {
    use super::*;
    
    // Test plugin for testing purposes
    #[derive(Default)]
    pub struct TestPlugin {
        pub before_request_called: std::sync::Arc<tokio::sync::Mutex<bool>>,
        pub after_request_called: std::sync::Arc<tokio::sync::Mutex<bool>>,
        pub initialize_called: std::sync::Arc<tokio::sync::Mutex<bool>>,
    }

    #[async_trait::async_trait]
    impl Plugin for TestPlugin {
        fn name(&self) -> &str {
            "test_plugin"
        }
        
        async fn before_request(&self, _req: &mut Request) -> Result<()> {
            let mut called = self.before_request_called.lock().await;
            *called = true;
            Ok(())
        }
        
        async fn after_request(&self, _req: &Request, _resp: &mut Response) -> Result<()> {
            let mut called = self.after_request_called.lock().await;
            *called = true;
            Ok(())
        }
        
        async fn initialize(&self, _router: &mut Router) -> Result<()> {
            let mut called = self.initialize_called.lock().await;
            *called = true;
            Ok(())
        }
    }
    
    #[tokio::test]
    async fn test_plugin_execution() {
        let test_plugin = TestPlugin::default();
        let plugin_arc = Arc::new(test_plugin);
        
        let before_called = plugin_arc.before_request_called.clone();
        let after_called = plugin_arc.after_request_called.clone();
        let init_called = plugin_arc.initialize_called.clone();
        
        let server = TestServer::new(move |router| {
            router.get("/test", |_req| async { 
                Ok(Response::text("test response".to_string())) 
            });
        }).await;
        
        // Manually test plugin methods
        let mut req = Request::builder()
            .uri("/test")
            .body(Default::default())
            .unwrap();
        
        // Test before_request
        plugin_arc.before_request(&mut req).await.unwrap();
        assert!(*before_called.lock().await);
        
        // Test initialize
        let mut router = Router::new();
        plugin_arc.initialize(&mut router).await.unwrap();
        assert!(*init_called.lock().await);
        
        // Test after_request
        let mut resp = Response::ok();
        plugin_arc.after_request(&req, &mut resp).await.unwrap();
        assert!(*after_called.lock().await);
    }
    
    #[tokio::test]
    async fn test_cors_plugin() {
        let cors_plugin = Arc::new(CorsPlugin::new());
        
        let server = TestServer::new(move |router| {
            router.get("/api/test")
                .handler(|_req| async { Ok(Response::text("API response".to_string())) });
        }).await;
        
        // Test preflight request
        let response = server
            .request(http::Method::OPTIONS, "/api/test")
            .header("Origin", "http://localhost:3000")
            .header("Access-Control-Request-Method", "POST")
            .send()
            .await;
        
        assert_eq!(response.status(), 200);
    }
    
    #[tokio::test]
    async fn test_rate_limit_plugin() {
        let rate_limit_plugin = Arc::new(RateLimitPlugin::new(2, 1)); // 2 requests per 1 second
        
        // Test rate limiting by calling the plugin directly
        let mut req = Request::builder()
            .uri("/test")
            .header("X-Forwarded-For", "127.0.0.1")
            .body(Default::default())
            .unwrap();
        
        // First request should succeed
        assert!(rate_limit_plugin.before_request(&mut req).await.is_ok());
        
        // Second request should succeed
        assert!(rate_limit_plugin.before_request(&mut req).await.is_ok());
        
        // Third request should be rate limited
        match rate_limit_plugin.before_request(&mut req).await {
            Err(Error::TooManyRequests) => (), // Expected
            _ => panic!("Expected TooManyRequests error"),
        }
    }
}
```

## Plugin Best Practices

Follow these best practices when creating plugins:

```rust
use oxidite::prelude::*;

/// Well-designed plugin example
pub struct WellDesignedPlugin {
    config: PluginConfig,
    // Use appropriate data structures for state
    state: Arc<tokio::sync::RwLock<PluginState>>,
}

#[derive(Default)]
struct PluginState {
    initialized: bool,
    stats: PluginStats,
}

#[derive(Default)]
struct PluginStats {
    requests_processed: u64,
    errors_encountered: u64,
}

impl WellDesignedPlugin {
    pub fn new(config: PluginConfig) -> Self {
        Self {
            config,
            state: Arc::new(tokio::sync::RwLock::new(PluginState::default())),
        }
    }
    
    /// Plugin should have clear, descriptive name
    fn name(&self) -> &str {
        "well_designed"
    }
    
    /// Document what the plugin does
    /// This plugin demonstrates best practices for plugin development
    async fn before_request(&self, req: &mut Request) -> Result<()> {
        // Use proper error handling
        if !self.config.enabled {
            return Ok(());
        }
        
        // Update statistics safely
        {
            let mut state = self.state.write().await;
            state.stats.requests_processed += 1;
        }
        
        // Implement proper logging
        println!("WellDesignedPlugin processing request: {} {}", 
                 req.method(), req.uri());
        
        Ok(())
    }
    
    /// Clean shutdown is important
    async fn shutdown(&self) -> Result<()> {
        let state = self.state.read().await;
        println!("Plugin stats - processed: {}, errors: {}", 
                 state.stats.requests_processed, state.stats.errors_encountered);
        Ok(())
    }
}

// Plugin documentation checklist:
// ✓ Clear purpose and functionality
// ✓ Proper error handling
// ✓ Configuration options
// ✓ Performance considerations
// ✓ Security best practices
// ✓ Proper shutdown/cleanup
// ✓ Testing strategy
// ✓ Documentation
// ✓ Dependency management
// ✓ Compatibility considerations
```

## Summary

Oxidite plugins provide a powerful way to:

- **Extend functionality**: Add new features to the framework
- **Modular design**: Keep applications organized and maintainable
- **Share components**: Reuse code across multiple applications
- **Hook into lifecycle**: Intercept and modify request/response flow
- **Configure behavior**: Customize plugin behavior through settings
- **Manage dependencies**: Handle plugin interdependencies
- **Ensure testability**: Make plugins easy to test in isolation

The plugin system enables building rich, extensible Oxidite applications while maintaining clean separation of concerns and promoting code reuse.