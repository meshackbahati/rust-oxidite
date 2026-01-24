# Production Setup

Deploying Oxidite applications to production requires careful consideration of performance, security, monitoring, and reliability. This chapter covers everything you need to know to run Oxidite applications in production.

## Overview

Production setup includes:
- Environment configuration
- Security hardening
- Performance optimization
- Monitoring and logging
- Deployment strategies
- Scaling considerations
- Backup and disaster recovery

## Environment Configuration

Configure your application for production environments:

```toml
# config/production.toml
[server]
host = "0.0.0.0"
port = 80
workers = 4
timeout = 30
keep_alive = 75
tcp_nodelay = true

[database]
url = "${DATABASE_URL}"
pool_size = 20
timeout = 30
max_lifetime = 1800
idle_timeout = 600

[logging]
level = "info"
format = "json"
output = "stdout"
sentry_dsn = "${SENTRY_DSN}"

[cache]
backend = "redis"
url = "${REDIS_URL}"
ttl = 3600

[security]
cors_enabled = true
allowed_origins = ["https://yourdomain.com", "https://www.yourdomain.com"]
csrf_enabled = true
hsts_enabled = true
content_security_policy = "default-src 'self'; script-src 'self' 'unsafe-inline'"
rate_limiting = true
max_requests_per_minute = 100

[ssl]
enabled = true
cert_path = "/etc/ssl/certs/cert.pem"
key_path = "/etc/ssl/private/key.pem"
```

### Environment Variables

Use environment variables for sensitive configuration:

```bash
# Production environment variables
export DATABASE_URL="postgresql://user:pass@prod-db:5432/app_prod"
export REDIS_URL="redis://prod-redis:6379"
export JWT_SECRET="long-random-string-here"
export ENCRYPTION_KEY="32-byte-encryption-key-here"
export SENTRY_DSN="https://key@sentry.io/project"
export SMTP_HOST="smtp.gmail.com"
export SMTP_USER="noreply@yourdomain.com"
export SMTP_PASS="smtp-password"
export AWS_ACCESS_KEY_ID="your-access-key"
export AWS_SECRET_ACCESS_KEY="your-secret-key"
```

### Configuration Loading

Load configuration dynamically:

```rust
use oxidite::prelude::*;
use config::{Config, ConfigError, Environment, File};

#[derive(serde::Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub security: SecurityConfig,
    pub cache: CacheConfig,
}

#[derive(serde::Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub timeout: u64,
}

#[derive(serde::Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: u32,
    pub timeout: u64,
}

#[derive(serde::Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub sentry_dsn: Option<String>,
}

#[derive(serde::Deserialize, Clone)]
pub struct SecurityConfig {
    pub cors_enabled: bool,
    pub allowed_origins: Vec<String>,
    pub csrf_enabled: bool,
    pub rate_limiting: bool,
    pub max_requests_per_minute: u32,
}

#[derive(serde::Deserialize, Clone)]
pub struct CacheConfig {
    pub backend: String,
    pub url: String,
    pub ttl: u64,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut cfg = Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name(&format!("config/{}", 
                std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string())
            )).required(false))
            .add_source(Environment::with_prefix("APP"));

        // Override with specific environment if set
        if let Ok(env) = std::env::var("APP_ENV") {
            cfg = cfg.add_source(File::with_name(&format!("config/{}", env)).required(false));
        }

        cfg.build()?.try_deserialize()
    }
}

// Initialize application with configuration
#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::from_env()
        .map_err(|e| Error::InternalServerError(format!("Configuration error: {}", e)))?;
    
    // Initialize logging
    init_logging(&config.logging).await?;
    
    // Initialize database
    init_database(&config.database).await?;
    
    // Initialize cache
    init_cache(&config.cache).await?;
    
    // Create and run server
    let router = create_routes(&config).await?;
    let server = Server::new(router);
    
    server.listen(format!("{}:{}", config.server.host, config.server.port).parse()?).await
}

async fn init_logging(config: &LoggingConfig) -> Result<()> {
    // Initialize logging based on configuration
    match config.level.as_str() {
        "debug" => std::env::set_var("RUST_LOG", "debug"),
        "info" => std::env::set_var("RUST_LOG", "info"),
        "warn" => std::env::set_var("RUST_LOG", "warn"),
        "error" => std::env::set_var("RUST_LOG", "error"),
        _ => std::env::set_var("RUST_LOG", "info"),
    }
    
    // Initialize tracing subscriber
    use tracing_subscriber::{EnvFilter, fmt};
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or(EnvFilter::new(&config.level));
    
    let subscriber = fmt()
        .with_env_filter(filter)
        .json();
    
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| Error::InternalServerError(format!("Logging setup error: {}", e)))?;
    
    Ok(())
}

async fn init_database(config: &DatabaseConfig) -> Result<()> {
    // Initialize database connection pool
    println!("Connecting to database: {}", config.url);
    Ok(())
}

async fn init_cache(config: &CacheConfig) -> Result<()> {
    // Initialize cache backend
    println!("Connecting to cache: {} ({})", config.url, config.backend);
    Ok(())
}

async fn create_routes(_config: &AppConfig) -> Result<Router> {
    let mut router = Router::new();
    
    // Add routes
    router.get("/", |_req| async { Ok(Response::text("Hello from production!".to_string())) });
    
    Ok(router)
}
```

## Security Hardening

Implement security best practices:

```rust
use oxidite::prelude::*;

// Security middleware
async fn security_middleware(req: Request, next: Next) -> Result<Response> {
    // Add security headers
    let mut response = next.run(req).await?;
    
    // HTTP Strict Transport Security
    response.headers_mut().insert(
        "Strict-Transport-Security",
        "max-age=31536000; includeSubDomains; preload".parse().unwrap()
    );
    
    // X-Frame-Options
    response.headers_mut().insert(
        "X-Frame-Options",
        "SAMEORIGIN".parse().unwrap()
    );
    
    // X-Content-Type-Options
    response.headers_mut().insert(
        "X-Content-Type-Options",
        "nosniff".parse().unwrap()
    );
    
    // X-XSS-Protection
    response.headers_mut().insert(
        "X-XSS-Protection",
        "1; mode=block".parse().unwrap()
    );
    
    // Content Security Policy
    response.headers_mut().insert(
        "Content-Security-Policy",
        "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' https:; connect-src 'self' https://*.sentry.io".parse().unwrap()
    );
    
    // Referrer Policy
    response.headers_mut().insert(
        "Referrer-Policy",
        "strict-origin-when-cross-origin".parse().unwrap()
    );
    
    Ok(response)
}

// Input validation middleware
async fn input_validation_middleware(req: Request, next: Next) -> Result<Response> {
    // Validate content length
    if let Some(content_length) = req.headers().get("content-length") {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<usize>() {
                const MAX_BODY_SIZE: usize = 10 * 1024 * 1024; // 10MB
                if length > MAX_BODY_SIZE {
                    return Err(Error::PayloadTooLarge);
                }
            }
        }
    }
    
    // Sanitize input (simplified)
    let mut req = req;
    validate_request_body(&mut req).await?;
    
    next.run(req).await
}

async fn validate_request_body(req: &mut Request) -> Result<()> {
    // In a real implementation, this would validate and sanitize the request body
    // Check for SQL injection patterns, XSS attempts, etc.
    Ok(())
}

// Rate limiting middleware
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct RateLimiter {
    limits: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
    max_requests: u32,
    window_duration: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self {
            limits: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            window_duration: Duration::from_secs(window_seconds),
        }
    }
    
    pub async fn is_allowed(&self, identifier: &str) -> bool {
        let now = Instant::now();
        let window_start = now - self.window_duration;
        
        let mut limits = self.limits.write().await;
        
        // Clean old requests
        if let Some(times) = limits.get_mut(identifier) {
            times.retain(|time| *time > window_start);
        }
        
        // Check limit
        let count = limits
            .entry(identifier.to_string())
            .or_insert_with(Vec::new)
            .len();
        
        if count < self.max_requests as usize {
            limits.get_mut(identifier).unwrap().push(now);
            true
        } else {
            false
        }
    }
}

async fn rate_limiting_middleware(
    req: Request,
    next: Next,
    State(rate_limiter): State<Arc<RateLimiter>>
) -> Result<Response> {
    let client_ip = req.headers()
        .get("x-forwarded-for")
        .and_then(|hv| hv.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    
    if !rate_limiter.is_allowed(&client_ip).await {
        return Err(Error::TooManyRequests);
    }
    
    next.run(req).await
}
```

## Performance Optimization

Optimize your application for production performance:

```rust
use oxidite::prelude::*;
use std::sync::Arc;

// Connection pooling configuration
pub struct ConnectionPoolConfig {
    pub min_connections: u32,
    pub max_connections: u32,
    pub acquire_timeout: std::time::Duration,
    pub idle_timeout: std::time::Duration,
    pub max_lifetime: std::time::Duration,
}

impl ConnectionPoolConfig {
    pub fn production() -> Self {
        Self {
            min_connections: 5,
            max_connections: 20,
            acquire_timeout: std::time::Duration::from_secs(30),
            idle_timeout: std::time::Duration::from_secs(600),
            max_lifetime: std::time::Duration::from_secs(1800),
        }
    }
}

// Caching middleware
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct CacheLayer {
    store: Arc<RwLock<HashMap<String, CachedResponse>>>,
    ttl: std::time::Duration,
}

#[derive(Clone)]
struct CachedResponse {
    response: Response,
    timestamp: std::time::Instant,
}

impl CacheLayer {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            ttl: std::time::Duration::from_secs(ttl_seconds),
        }
    }
    
    pub async fn get(&self, key: &str) -> Option<Response> {
        let cache = self.store.read().await;
        if let Some(cached) = cache.get(key) {
            if cached.timestamp.elapsed() < self.ttl {
                Some(cached.response.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
    
    pub async fn set(&self, key: String, response: Response) {
        let mut cache = self.store.write().await;
        cache.insert(key, CachedResponse {
            response,
            timestamp: std::time::Instant::now(),
        });
    }
}

// Caching middleware for GET requests
async fn caching_middleware(
    req: Request,
    next: Next,
    State(cache): State<Arc<CacheLayer>>
) -> Result<Response> {
    if req.method() == http::Method::GET {
        let cache_key = format!("{}:{}", req.method(), req.uri());
        
        // Try to get from cache
        if let Some(cached_response) = cache.get(&cache_key).await {
            return Ok(cached_response);
        }
        
        // Execute request
        let response = next.run(req).await?;
        
        // Cache the response if appropriate
        if response.status().is_success() {
            cache.set(cache_key, response.clone()).await;
        }
        
        Ok(response)
    } else {
        // For non-GET requests, bypass cache
        next.run(req).await
    }
}

// Compression middleware
use brotli::enc::backward_references::BrotliEncoderParams;
use flate2::write::{GzEncoder, DeflateEncoder};
use flate2::Compression;

async fn compression_middleware(req: Request, next: Next) -> Result<Response> {
    let accept_encoding = req.headers()
        .get("accept-encoding")
        .and_then(|hv| hv.to_str().ok())
        .unwrap_or("");
    
    let response = next.run(req).await?;
    
    // Only compress if response is large enough and client accepts compression
    let body_size = get_body_size(&response);
    if body_size > 1024 && response.status().is_success() {
        let mut response = response;
        
        if accept_encoding.contains("br") {
            // Brotli compression
            compress_response_br(&mut response).await?;
        } else if accept_encoding.contains("gzip") {
            // Gzip compression
            compress_response_gzip(&mut response).await?;
        } else if accept_encoding.contains("deflate") {
            // Deflate compression
            compress_response_deflate(&mut response).await?;
        }
    }
    
    Ok(response)
}

fn get_body_size(response: &Response) -> usize {
    // Get the size of the response body
    // This is a simplified implementation
    0
}

async fn compress_response_br(response: &mut Response) -> Result<()> {
    // Brotli compression implementation
    response.headers_mut().insert(
        "Content-Encoding",
        "br".parse().unwrap()
    );
    Ok(())
}

async fn compress_response_gzip(response: &mut Response) -> Result<()> {
    // Gzip compression implementation
    response.headers_mut().insert(
        "Content-Encoding",
        "gzip".parse().unwrap()
    );
    Ok(())
}

async fn compress_response_deflate(response: &mut Response) -> Result<()> {
    // Deflate compression implementation
    response.headers_mut().insert(
        "Content-Encoding",
        "deflate".parse().unwrap()
    );
    Ok(())
}

// Static file serving with caching
use std::path::Path;
use tokio::fs;

async fn static_file_handler(Path(file_path): Path<String>) -> Result<Response> {
    // Validate path to prevent directory traversal
    if file_path.contains("..") || file_path.starts_with('/') {
        return Err(Error::BadRequest("Invalid file path".to_string()));
    }
    
    let full_path = format!("public/{}", file_path);
    
    // Check if file exists
    if !Path::new(&full_path).exists() {
        return Err(Error::NotFound);
    }
    
    // Read file
    let contents = fs::read(&full_path).await
        .map_err(|e| Error::InternalServerError(format!("Failed to read file: {}", e)))?;
    
    // Set appropriate content type
    let content_type = get_content_type(&file_path);
    
    let mut response = Response::html(String::from_utf8_lossy(&contents).to_string());
    response.headers_mut().insert(
        "Content-Type",
        content_type.parse().unwrap()
    );
    
    // Add caching headers
    response.headers_mut().insert(
        "Cache-Control",
        "public, max-age=31536000".parse().unwrap() // 1 year
    );
    
    // Add ETag for cache validation
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(&contents);
    let hash = format!("{:x}", hasher.finalize());
    response.headers_mut().insert(
        "ETag",
        format!("\"{}\"", hash).parse().unwrap()
    );
    
    Ok(response)
}

fn get_content_type(path: &str) -> &'static str {
    match std::path::Path::new(path).extension().and_then(|ext| ext.to_str()) {
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("eot") => "application/vnd.ms-fontobject",
        _ => "application/octet-stream",
    }
}
```

## Monitoring and Logging

Implement comprehensive monitoring:

```rust
use oxidite::prelude::*;
use serde_json::json;

// Structured logging middleware
async fn logging_middleware(req: Request, next: Next) -> Result<Response> {
    let start = std::time::Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();
    let user_agent = req.headers()
        .get("user-agent")
        .and_then(|hv| hv.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    
    let remote_addr = req.headers()
        .get("x-forwarded-for")
        .and_then(|hv| hv.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    
    let response = next.run(req).await;
    let duration = start.elapsed();
    
    let log_entry = json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "level": "info",
        "event": "http_request",
        "method": method.to_string(),
        "uri": uri.to_string(),
        "user_agent": user_agent,
        "remote_addr": remote_addr,
        "status": response.as_ref().map(|r| r.status().as_u16()).unwrap_or(500),
        "duration_ms": duration.as_millis(),
        "service": "oxidite-app"
    });
    
    // Log to stdout in JSON format
    println!("{}", log_entry);
    
    response
}

// Error logging middleware
async fn error_logging_middleware(req: Request, next: Next) -> Result<Response> {
    match next.run(req).await {
        Ok(response) => Ok(response),
        Err(error) => {
            let error_log = json!({
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "level": "error",
                "event": "http_error",
                "error": error.to_string(),
                "error_type": error_type_name(&error),
                "method": req.method().to_string(),
                "uri": req.uri().to_string(),
                "service": "oxidite-app"
            });
            
            eprintln!("{}", error_log);
            
            Err(error)
        }
    }
}

fn error_type_name(error: &Error) -> &'static str {
    match error {
        Error::NotFound => "NotFound",
        Error::BadRequest(_) => "BadRequest",
        Error::Unauthorized(_) => "Unauthorized",
        Error::Forbidden => "Forbidden",
        Error::TooManyRequests => "TooManyRequests",
        Error::InternalServerError => "InternalServerError",
        Error::InternalServerError(_) => "Server",
        Error::Validation(_) => "Validation",
        Error::RateLimited => "RateLimited",
        _ => "Unknown",
    }
}

// Metrics collection
use std::sync::atomic::{AtomicU64, Ordering};

pub struct RequestMetrics {
    pub total_requests: AtomicU64,
    pub total_errors: AtomicU64,
    pub total_2xx: AtomicU64,
    pub total_3xx: AtomicU64,
    pub total_4xx: AtomicU64,
    pub total_5xx: AtomicU64,
}

impl RequestMetrics {
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            total_errors: AtomicU64::new(0),
            total_2xx: AtomicU64::new(0),
            total_3xx: AtomicU64::new(0),
            total_4xx: AtomicU64::new(0),
            total_5xx: AtomicU64::new(0),
        }
    }
    
    pub fn increment_request(&self, status_code: u16) {
        self.total_requests.fetch_add(1, Ordering::SeqCst);
        
        match status_code {
            200..=299 => {
                self.total_2xx.fetch_add(1, Ordering::SeqCst);
            }
            300..=399 => {
                self.total_3xx.fetch_add(1, Ordering::SeqCst);
            }
            400..=499 => {
                self.total_4xx.fetch_add(1, Ordering::SeqCst);
            }
            500..=599 => {
                self.total_5xx.fetch_add(1, Ordering::SeqCst);
                self.total_errors.fetch_add(1, Ordering::SeqCst);
            }
            _ => {}
        }
    }
    
    pub fn get_stats(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            total_requests: self.total_requests.load(Ordering::SeqCst),
            total_errors: self.total_errors.load(Ordering::SeqCst),
            total_2xx: self.total_2xx.load(Ordering::SeqCst),
            total_3xx: self.total_3xx.load(Ordering::SeqCst),
            total_4xx: self.total_4xx.load(Ordering::SeqCst),
            total_5xx: self.total_5xx.load(Ordering::SeqCst),
        }
    }
}

pub struct MetricsSnapshot {
    pub total_requests: u64,
    pub total_errors: u64,
    pub total_2xx: u64,
    pub total_3xx: u64,
    pub total_4xx: u64,
    pub total_5xx: u64,
}

// Metrics endpoint
async fn metrics_endpoint(State(metrics): State<Arc<RequestMetrics>>) -> Result<Response> {
    let stats = metrics.get_stats();
    
    let metrics_json = json!({
        "uptime": get_uptime(),
        "requests": {
            "total": stats.total_requests,
            "2xx": stats.total_2xx,
            "3xx": stats.total_3xx,
            "4xx": stats.total_4xx,
            "5xx": stats.total_5xx,
        },
        "errors": {
            "total": stats.total_errors,
            "rate": if stats.total_requests > 0 {
                (stats.total_errors as f64 / stats.total_requests as f64) * 100.0
            } else {
                0.0
            }
        },
        "health": "healthy"
    });
    
    Ok(Response::json(metrics_json))
}

fn get_uptime() -> String {
    // Calculate application uptime
    // This would typically be tracked from application start time
    "0h 0m 0s".to_string()
}
```

## Deployment Strategies

Deploy your application with various strategies:

```bash
# Dockerfile for production deployment
FROM rust:1.92 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/my_app /usr/local/bin/my_app
EXPOSE 80
CMD ["my_app"]

# docker-compose.yml for production
version: '3.8'
services:
  app:
    build: .
    ports:
      - "80:80"
    environment:
      - APP_ENV=production
      - DATABASE_URL=postgresql://user:pass@db:5432/app_prod
      - REDIS_URL=redis://redis:6379
    depends_on:
      - db
      - redis
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

  db:
    image: postgres:15
    environment:
      POSTGRES_DB: app_prod
      POSTGRES_USER: user
      POSTGRES_PASSWORD: pass
    volumes:
      - postgres_data:/var/lib/postgresql/data
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    restart: unless-stopped

volumes:
  postgres_data:

# Kubernetes deployment example
apiVersion: apps/v1
kind: Deployment
metadata:
  name: oxidite-app
spec:
  replicas: 3
  selector:
    matchLabels:
      app: oxidite-app
  template:
    metadata:
      labels:
        app: oxidite-app
    spec:
      containers:
      - name: app
        image: my-org/oxidite-app:latest
        ports:
        - containerPort: 80
        env:
        - name: APP_ENV
          value: "production"
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: db-secret
              key: url
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 80
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 80
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: oxidite-app-service
spec:
  selector:
    app: oxidite-app
  ports:
    - protocol: TCP
      port: 80
      targetPort: 80
  type: LoadBalancer
```

## Health Checks

Implement health check endpoints:

```rust
use oxidite::prelude::*;

// Health check endpoint
async fn health_check(_req: Request) -> Result<Response> {
    // Perform health checks
    let db_healthy = check_database_health().await;
    let cache_healthy = check_cache_health().await;
    let disk_space_ok = check_disk_space().await;
    
    let healthy = db_healthy && cache_healthy && disk_space_ok;
    
    let status = if healthy { "healthy" } else { "unhealthy" };
    
    let health_response = serde_json::json!({
        "status": status,
        "checks": {
            "database": db_healthy,
            "cache": cache_healthy,
            "disk_space": disk_space_ok
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    if healthy {
        Ok(Response::json(health_response))
    } else {
        Ok(Response::json(health_response)) // Still return 200 but with unhealthy status
    }
}

async fn readiness_check(_req: Request) -> Result<Response> {
    // Readiness check - is the app ready to serve traffic?
    let ready = check_readiness_conditions().await;
    
    if ready {
        Ok(Response::ok())
    } else {
        Err(Error::ServiceUnavailable("Application not ready".to_string()))
    }
}

async fn liveness_check(_req: Request) -> Result<Response> {
    // Liveness check - is the app alive?
    // Usually just a simple response to indicate the process is running
    Ok(Response::ok())
}

async fn check_database_health() -> bool {
    // Check database connectivity
    // In a real app, this would make a simple query
    true
}

async fn check_cache_health() -> bool {
    // Check cache connectivity
    // In a real app, this would ping the cache
    true
}

async fn check_disk_space() -> bool {
    // Check available disk space
    // In a real app, this would check actual disk usage
    true
}

async fn check_readiness_conditions() -> bool {
    // Check if all prerequisites are met
    check_database_health().await && check_cache_health().await
}
```

## Backup and Recovery

Implement backup strategies:

```rust
use oxidite::prelude::*;
use tokio::fs;
use std::path::Path;

// Backup handler
async fn backup_handler(_req: Request) -> Result<Response> {
    // Trigger a backup
    let backup_result = create_backup().await;
    
    match backup_result {
        Ok(backup_info) => Ok(Response::json(serde_json::json!({
            "status": "success",
            "backup": backup_info
        }))),
        Err(e) => Err(Error::InternalServerError(format!("Backup failed: {}", e))),
    }
}

async fn create_backup() -> Result<BackupInfo> {
    // Create a database backup
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_filename = format!("backup_{}.sql", timestamp);
    let backup_path = format!("./backups/{}", backup_filename);
    
    // Ensure backup directory exists
    fs::create_dir_all("./backups").await?;
    
    // In a real app, this would export the database
    // For example: pg_dump for PostgreSQL
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await; // Simulate backup process
    
    // Create a dummy backup file for the example
    fs::write(&backup_path, "/* Database backup content */").await?;
    
    Ok(BackupInfo {
        filename: backup_filename,
        path: backup_path,
        size: 1024, // Size in bytes
        created_at: chrono::Utc::now().to_rfc3339(),
    })
}

#[derive(serde::Serialize)]
struct BackupInfo {
    filename: String,
    path: String,
    size: u64,
    created_at: String,
}

// Restore handler
async fn restore_handler(
    Path(backup_file): Path<String>,
    _req: Request
) -> Result<Response> {
    let backup_path = format!("./backups/{}", backup_file);
    
    if !Path::new(&backup_path).exists() {
        return Err(Error::NotFound);
    }
    
    // In a real app, this would restore the database from the backup
    let restore_result = restore_from_backup(&backup_path).await;
    
    match restore_result {
        Ok(_) => Ok(Response::json(serde_json::json!({
            "status": "success",
            "message": "Restore completed successfully"
        }))),
        Err(e) => Err(Error::InternalServerError(format!("Restore failed: {}", e))),
    }
}

async fn restore_from_backup(_backup_path: &str) -> Result<()> {
    // In a real app, this would restore the database
    // For example: psql to import a PostgreSQL dump
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await; // Simulate restore process
    Ok(())
}

// List backups
async fn list_backups(_req: Request) -> Result<Response> {
    let mut backups = Vec::new();
    
    // Scan backup directory
    let mut entries = fs::read_dir("./backups").await?;
    
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("sql") {
            if let Some(filename) = path.file_name().and_then(|name| name.to_str()) {
                let metadata = entry.metadata().await?;
                backups.push(serde_json::json!({
                    "filename": filename,
                    "size": metadata.len(),
                    "created_at": format!("{:?}", metadata.created())
                }));
            }
        }
    }
    
    // Sort by creation time (newest first)
    backups.sort_by(|a, b| {
        b["created_at"].as_str().cmp(&a["created_at"].as_str())
    });
    
    Ok(Response::json(serde_json::json!({
        "backups": backups,
        "count": backups.len()
    })))
}
```

## Scaling Considerations

Design for horizontal scaling:

```rust
use oxidite::prelude::*;

// Horizontal scaling considerations
pub struct ScalableAppState {
    pub app_id: String,
    pub instance_id: String,
    pub cluster_nodes: Vec<String>,
    pub shared_cache: Arc<dyn CacheProvider>,
    pub shared_database: Arc<dyn DatabaseProvider>,
}

// Cache provider trait for pluggable cache backends
pub trait CacheProvider: Send + Sync {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&self, key: String, value: String, ttl: std::time::Duration) -> Result<()>;
    fn delete(&self, key: &str) -> Result<()>;
    fn clear(&self) -> Result<()>;
}

// Database provider trait for pluggable database backends
pub trait DatabaseProvider: Send + Sync {
    fn query(&self, sql: &str) -> Result<Vec<serde_json::Value>>;
    fn execute(&self, sql: &str) -> Result<u64>;
    fn transaction<F, R>(&self, f: F) -> Result<R> 
    where 
        F: FnOnce(&dyn Transaction) -> Result<R>;
}

pub trait Transaction {
    fn query(&self, sql: &str) -> Result<Vec<serde_json::Value>>;
    fn execute(&self, sql: &str) -> Result<u64>;
}

// Distributed lock for coordination between instances
pub trait DistributedLock: Send + Sync {
    fn acquire(&self, key: &str, ttl: std::time::Duration) -> Result<LockGuard>;
}

pub struct LockGuard;

impl Drop for LockGuard {
    fn drop(&mut self) {
        // Release lock automatically
    }
}

// Example of scaling-aware handler
async fn distributed_handler(
    _req: Request,
    State(app_state): State<Arc<ScalableAppState>>
) -> Result<Response> {
    // Use distributed locks for critical sections
    let lock = app_state.shared_cache.acquire_lock("critical_section", 
        std::time::Duration::from_secs(30)).await?;
    
    // Perform critical operation
    let result = perform_critical_operation().await?;
    
    // Lock is automatically released when guard goes out of scope
    drop(lock);
    
    Ok(Response::json(result))
}

async fn perform_critical_operation() -> Result<serde_json::Value> {
    // Critical operation that should only run on one instance at a time
    Ok(serde_json::json!({ "status": "completed" }))
}

// Load balancing considerations
async fn load_balancer_health_check(_req: Request) -> Result<Response> {
    // Return light-weight health check for load balancers
    Ok(Response::text("OK".to_string()))
}

// Instance-specific information
async fn instance_info(
    State(app_state): State<Arc<ScalableAppState>>
) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "instance_id": app_state.instance_id,
        "app_id": app_state.app_id,
        "cluster_size": app_state.cluster_nodes.len(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}
```

## Summary

Production setup for Oxidite applications requires attention to:

- **Environment Configuration**: Proper configuration loading and environment variables
- **Security Hardening**: Headers, validation, rate limiting, and input sanitization
- **Performance Optimization**: Caching, compression, and connection pooling
- **Monitoring**: Logging, metrics, and health checks
- **Deployment**: Containerization, orchestration, and scaling
- **Backup and Recovery**: Regular backups and restore procedures
- **Scaling**: Horizontal scaling with shared resources

Following these practices ensures your Oxidite applications are secure, performant, and reliable in production environments.