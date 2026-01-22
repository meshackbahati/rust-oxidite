# Performance

Performance optimization is crucial for delivering fast, responsive Oxidite applications. This chapter covers various techniques and strategies to optimize your application's performance.

## Overview

Performance optimization includes:
- Request handling optimization
- Database query optimization
- Caching strategies
- Memory management
- Concurrency and parallelism
- Network optimizations
- Profiling and monitoring

## Request Handling Optimization

Optimize how your application handles incoming requests:

```rust
use oxidite::prelude::*;
use std::sync::Arc;

// Efficient request handler with minimal allocations
async fn optimized_handler(req: Request) -> Result<Response> {
    // Pre-allocate response data structures
    let mut response_data = String::with_capacity(1024);
    
    // Use efficient string operations
    response_data.push_str("{\"message\":\"Hello, World!\",\"timestamp\":\"");
    response_data.push_str(&chrono::Utc::now().to_rfc3339());
    response_data.push_str("\"}");
    
    Ok(Response::json(serde_json::Value::String(response_data)))
}

// Lazy evaluation for expensive operations
async fn lazy_evaluation_handler(req: Request) -> Result<Response> {
    // Only perform expensive operation if needed
    let include_details = req.uri().query()
        .map(|q| q.contains("details=true"))
        .unwrap_or(false);
    
    let mut response = serde_json::json!({
        "simple": "data"
    });
    
    if include_details {
        // Only execute expensive operation when necessary
        let expensive_data = expensive_computation().await;
        response["expensive_data"] = expensive_data;
    }
    
    Ok(Response::json(response))
}

async fn expensive_computation() -> serde_json::Value {
    // Simulate expensive computation
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    serde_json::json!({ "computed": true, "value": 42 })
}

// Request preprocessing middleware
async fn preprocessing_middleware(req: Request, next: Next) -> Result<Response> {
    // Parse and validate request early
    if req.method() == http::Method::POST || req.method() == http::Method::PUT {
        // Check content length before processing
        if let Some(content_length) = req.headers().get("content-length") {
            if let Ok(length_str) = content_length.to_str() {
                if let Ok(length) = length_str.parse::<usize>() {
                    const MAX_SIZE: usize = 10 * 1024 * 1024; // 10MB
                    
                    if length > MAX_SIZE {
                        return Err(Error::PayloadTooLarge);
                    }
                }
            }
        }
    }
    
    next.run(req).await
}
```

## Database Query Optimization

Optimize database interactions:

```rust
use oxidite::prelude::*;
use serde::{Deserialize, Serialize};

// Optimized model with proper indexing hints
#[derive(Model, Serialize, Deserialize)]
#[model(table = "optimized_users")]
pub struct OptimizedUser {
    #[model(primary_key)]
    pub id: i32,
    #[model(unique, not_null, indexed)]  // Add index hint
    pub email: String,
    #[model(not_null, indexed)]         // Add index hint
    pub name: String,
    #[model(indexed)]                   // Add index hint
    pub created_at: String,
}

// Batch operations for better performance
impl OptimizedUser {
    pub async fn create_batch(users: Vec<Self>) -> Result<Vec<Self>> {
        // In a real implementation, this would use bulk insert
        let mut results = Vec::with_capacity(users.len());
        
        for mut user in users {
            // Simulate batch insert
            user.id = rand::random::<i32>(); // Simulate auto-increment
            results.push(user);
        }
        
        Ok(results)
    }
    
    pub async fn find_by_ids(ids: &[i32]) -> Result<Vec<Self>> {
        // Use IN clause instead of multiple individual queries
        let id_list: String = ids.iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        
        // In a real implementation, this would execute: 
        // SELECT * FROM optimized_users WHERE id IN (...)
        Ok(vec![]) // Placeholder
    }
    
    pub async fn find_with_pagination(page: u32, per_page: u32) -> Result<(Vec<Self>, u32)> {
        // Implement efficient pagination
        let offset = (page - 1) * per_page;
        
        // In a real implementation, this would execute:
        // SELECT * FROM optimized_users LIMIT per_page OFFSET offset
        let users = vec![]; // Placeholder
        let total_count = 100; // Placeholder
        
        Ok((users, total_count))
    }
    
    pub async fn update_batch(updates: Vec<(i32, String)>) -> Result<u32> {
        // Batch update implementation
        let mut affected_rows = 0;
        
        for (id, name) in updates {
            // In a real implementation, this would execute:
            // UPDATE optimized_users SET name = ? WHERE id = ?
            affected_rows += 1; // Placeholder
        }
        
        Ok(affected_rows)
    }
}

// Connection pooling optimization
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Clone)]
pub struct OptimizedDbPool {
    semaphore: Arc<Semaphore>,
    connections: Vec<Arc<dyn DatabaseConnection>>,
}

pub trait DatabaseConnection: Send + Sync {
    fn execute(&self, query: &str) -> Result<()>;
    fn query(&self, query: &str) -> Result<Vec<serde_json::Value>>;
}

impl OptimizedDbPool {
    pub fn new(max_connections: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_connections)),
            connections: Vec::new(), // In a real implementation, populate with actual connections
        }
    }
    
    pub async fn with_connection<F, R>(&self, operation: F) -> Result<R>
    where
        F: FnOnce(&dyn DatabaseConnection) -> Result<R>,
    {
        let _permit = self.semaphore.acquire().await
            .map_err(|_| Error::Server("Connection pool error".to_string()))?;
        
        // In a real implementation, lease a connection and execute the operation
        // This is a simplified example
        let conn = self.get_connection()?;
        operation(conn.as_ref())
    }
    
    fn get_connection(&self) -> Result<Arc<dyn DatabaseConnection>> {
        // In a real implementation, return an available connection
        Err(Error::Server("Not implemented".to_string()))
    }
}

// Query optimization with prepared statements
pub struct PreparedStatement {
    query: String,
    param_types: Vec<DbType>,
}

#[derive(Debug)]
pub enum DbType {
    Integer,
    Text,
    Boolean,
    Timestamp,
}

impl PreparedStatement {
    pub fn new(query: &str) -> Self {
        // Parse query to identify parameter types
        Self {
            query: query.to_string(),
            param_types: vec![], // In a real implementation, parse parameter types
        }
    }
    
    pub async fn execute(&self, params: &[&dyn ToSql]) -> Result<()> {
        // Execute prepared statement with parameters
        // This would use the actual database driver
        Ok(())
    }
}

pub trait ToSql {
    fn to_sql(&self) -> String;
}
```

## Caching Strategies

Implement effective caching:

```rust
use oxidite::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use std::time::{Duration, Instant};

// In-memory cache with TTL
#[derive(Clone)]
pub struct InMemoryCache {
    store: Arc<RwLock<HashMap<String, CacheEntry>>>,
    capacity: usize,
    default_ttl: Duration,
}

#[derive(Clone)]
struct CacheEntry {
    value: String,
    timestamp: Instant,
    ttl: Duration,
}

impl InMemoryCache {
    pub fn new(capacity: usize, default_ttl: Duration) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            capacity,
            default_ttl,
        }
    }
    
    pub async fn get(&self, key: &str) -> Option<String> {
        let store = self.store.read().await;
        if let Some(entry) = store.get(key) {
            if entry.timestamp.elapsed() < entry.ttl {
                Some(entry.value.clone())
            } else {
                // Entry expired, will be cleaned up later
                None
            }
        } else {
            None
        }
    }
    
    pub async fn set(&self, key: String, value: String, ttl: Option<Duration>) -> Result<()> {
        let ttl = ttl.unwrap_or(self.default_ttl);
        
        let mut store = self.store.write().await;
        
        // Clean up expired entries if capacity is exceeded
        if store.len() >= self.capacity {
            store.retain(|_, entry| entry.timestamp.elapsed() < entry.ttl);
        }
        
        store.insert(key, CacheEntry {
            value,
            timestamp: Instant::now(),
            ttl,
        });
        
        Ok(())
    }
    
    pub async fn delete(&self, key: &str) -> Result<bool> {
        let mut store = self.store.write().await;
        Ok(store.remove(key).is_some())
    }
    
    pub async fn clear_expired(&self) -> Result<usize> {
        let mut store = self.store.write().await;
        let mut removed_count = 0;
        
        store.retain(|_, entry| {
            if entry.timestamp.elapsed() >= entry.ttl {
                removed_count += 1;
                false
            } else {
                true
            }
        });
        
        Ok(removed_count)
    }
}

// Redis-like cache implementation
pub struct RedisCache {
    client: Arc<MockRedisClient>, // In a real implementation, use actual Redis client
}

struct MockRedisClient;

impl MockRedisClient {
    pub async fn get(&self, _key: &str) -> Option<String> {
        Some("cached_value".to_string()) // Placeholder
    }
    
    pub async fn set(&self, _key: &str, _value: &str, _ttl: Duration) -> Result<()> {
        Ok(()) // Placeholder
    }
    
    pub async fn del(&self, _key: &str) -> Result<bool> {
        Ok(true) // Placeholder
    }
}

impl RedisCache {
    pub fn new() -> Self {
        Self {
            client: Arc::new(MockRedisClient),
        }
    }
    
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        Ok(self.client.get(key).await)
    }
    
    pub async fn set(&self, key: &str, value: &str, ttl: Duration) -> Result<()> {
        self.client.set(key, value, ttl).await
    }
    
    pub async fn delete(&self, key: &str) -> Result<bool> {
        self.client.del(key).await
    }
}

// Cache middleware
async fn caching_middleware(
    req: Request,
    next: Next,
    State(cache): State<Arc<InMemoryCache>>
) -> Result<Response> {
    if req.method() != http::Method::GET {
        // Only cache GET requests
        return next.run(req).await;
    }
    
    let cache_key = format!("response_{}_{}", req.method(), req.uri());
    
    // Try to get from cache
    if let Some(cached_response) = cache.get(&cache_key).await {
        return Ok(Response::html(cached_response));
    }
    
    // Execute request
    let response = next.run(req).await?;
    
    // Cache successful responses
    if response.status().is_success() {
        let response_clone = response.clone(); // In a real implementation, serialize response
        cache.set(cache_key, "cached_response".to_string(), Some(Duration::from_secs(300))).await?;
    }
    
    Ok(response)
}

// Cache-aside pattern implementation
pub struct CachedRepository {
    cache: Arc<InMemoryCache>,
    db_pool: OptimizedDbPool,
}

impl CachedRepository {
    pub fn new(cache: Arc<InMemoryCache>, db_pool: OptimizedDbPool) -> Self {
        Self { cache, db_pool }
    }
    
    pub async fn get_user(&self, id: i32) -> Result<Option<OptimizedUser>> {
        let cache_key = format!("user_{}", id);
        
        // Try cache first
        if let Some(cached) = self.cache.get(&cache_key).await {
            return Ok(serde_json::from_str(&cached).ok());
        }
        
        // Cache miss, query database
        let user = self.db_pool.with_connection(|conn| {
            // Execute SELECT * FROM users WHERE id = ?
            Ok(None::<OptimizedUser>) // Placeholder
        }).await?;
        
        // Cache the result if found
        if let Some(ref user) = user {
            if let Ok(serialized) = serde_json::to_string(user) {
                self.cache.set(cache_key, serialized, Some(Duration::from_secs(600))).await?;
            }
        }
        
        Ok(user)
    }
    
    pub async fn invalidate_user_cache(&self, id: i32) -> Result<()> {
        let cache_key = format!("user_{}", id);
        self.cache.delete(&cache_key).await?;
        Ok(())
    }
}
```

## Memory Management

Optimize memory usage:

```rust
use oxidite::prelude::*;
use std::sync::Arc;

// Efficient data structures
pub struct EfficientDataStructures;

impl EfficientDataStructures {
    // Use SmallVec for small collections that may grow
    pub fn use_small_collections() -> Result<()> {
        use smallvec::SmallVec;
        
        // Stack-allocated for small arrays, heap-allocated for larger ones
        let mut small_vec: SmallVec<[u32; 4]> = SmallVec::new();
        small_vec.push(1);
        small_vec.push(2);
        small_vec.push(3);
        small_vec.push(4);
        // If we add a 5th element, it moves to heap allocation
        
        Ok(())
    }
    
    // Use String instead of &str when ownership is needed
    pub fn efficient_string_handling() -> Result<()> {
        let mut buffer = String::with_capacity(1024); // Pre-allocate
        
        // Efficient string building
        buffer.push_str("Hello");
        buffer.push(' ');
        buffer.push_str("World");
        
        // Avoid unnecessary clones
        let shared_string = Arc::new(buffer);
        
        Ok(())
    }
    
    // Use Cow (Clone on Write) for flexible string handling
    pub fn cow_example(input: &str) -> std::borrow::Cow<str> {
        if input.contains("transform") {
            std::borrow::Cow::Owned(input.replace("transform", "optimized"))
        } else {
            std::borrow::Cow::Borrowed(input)
        }
    }
    
    // Use interned strings for repeated values
    pub fn interned_strings_example() -> Result<()> {
        use std::collections::HashMap;
        
        // For repeated string values, consider interning
        let mut string_interner = HashMap::new();
        string_interner.insert("status_active", 1);
        string_interner.insert("status_inactive", 2);
        
        Ok(())
    }
}

// Memory pool for frequently allocated objects
pub struct ObjectPool<T> {
    objects: Arc<Mutex<Vec<T>>>,
    factory: Box<dyn Fn() -> T + Send + Sync>,
}

impl<T: Send + 'static> ObjectPool<T> {
    pub fn new(factory: Box<dyn Fn() -> T + Send + Sync>, initial_size: usize) -> Self {
        let mut objects = Vec::with_capacity(initial_size);
        for _ in 0..initial_size {
            objects.push(factory());
        }
        
        Self {
            objects: Arc::new(Mutex::new(objects)),
            factory,
        }
    }
    
    pub async fn get(&self) -> PooledObject<T> {
        let mut objects = self.objects.lock().await;
        
        if let Some(obj) = objects.pop() {
            PooledObject {
                obj: Some(obj),
                pool: self.objects.clone(),
            }
        } else {
            // Create new object if pool is empty
            PooledObject {
                obj: Some((self.factory)()),
                pool: self.objects.clone(),
            }
        }
    }
}

pub struct PooledObject<T> {
    obj: Option<T>,
    pool: Arc<Mutex<Vec<T>>>,
}

impl<T> std::ops::Deref for PooledObject<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        self.obj.as_ref().unwrap()
    }
}

impl<T> std::ops::DerefMut for PooledObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.obj.as_mut().unwrap()
    }
}

impl<T> Drop for PooledObject<T> {
    fn drop(&mut self) {
        if let Some(obj) = self.obj.take() {
            // Return object to pool
            let mut pool = self.pool.blocking_lock();
            pool.push(obj);
        }
    }
}

// Memory-efficient JSON handling
use serde_json;

pub struct EfficientJsonHandler;

impl EfficientJsonHandler {
    pub async fn handle_large_json(req: Request) -> Result<Response> {
        use bytes::Bytes;
        
        // For large JSON payloads, process incrementally
        let body_bytes = hyper::body::to_bytes(req.into_body()).await
            .map_err(|e| Error::Server(e.to_string()))?;
        
        // Parse JSON efficiently
        let parsed: serde_json::Value = serde_json::from_slice(&body_bytes)
            .map_err(|e| Error::BadRequest(e.to_string()))?;
        
        // Process only needed fields to avoid memory overhead
        let result = process_needed_fields(&parsed);
        
        Ok(Response::json(result))
    }
    
    pub fn process_needed_fields(value: &serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Object(map) => {
                // Only extract needed fields
                let mut result = serde_json::Map::new();
                
                if let Some(id) = map.get("id") {
                    result.insert("id".to_string(), id.clone());
                }
                
                if let Some(name) = map.get("name") {
                    result.insert("name".to_string(), name.clone());
                }
                
                serde_json::Value::Object(result)
            }
            _ => value.clone(),
        }
    }
}
```

## Concurrency and Parallelism

Optimize concurrent operations:

```rust
use oxidite::prelude::*;
use tokio::task;
use std::sync::Arc;

// Parallel request processing
pub struct ParallelProcessor;

impl ParallelProcessor {
    pub async fn process_requests_in_parallel(requests: Vec<Request>) -> Result<Vec<Response>> {
        let mut handles = Vec::new();
        
        for request in requests {
            let handle = task::spawn(async move {
                // Process each request in parallel
                process_single_request(request).await
            });
            
            handles.push(handle);
        }
        
        let mut responses = Vec::with_capacity(handles.len());
        
        for handle in handles {
            match handle.await {
                Ok(response_result) => {
                    if let Ok(response) = response_result {
                        responses.push(response);
                    }
                }
                Err(e) => {
                    // Handle task panic
                    eprintln!("Task failed: {}", e);
                }
            }
        }
        
        Ok(responses)
    }
    
    pub async fn process_data_streams(data_chunks: Vec<Vec<u8>>) -> Result<Vec<Vec<u8>>> {
        // Process data chunks in parallel
        let handles: Vec<_> = data_chunks
            .into_iter()
            .map(|chunk| {
                task::spawn(async move {
                    process_chunk(chunk).await
                })
            })
            .collect();
        
        let mut results = Vec::new();
        
        for handle in handles {
            if let Ok(processed_chunk) = handle.await {
                if let Ok(chunk) = processed_chunk {
                    results.push(chunk);
                }
            }
        }
        
        Ok(results)
    }
}

async fn process_single_request(_req: Request) -> Result<Response> {
    // Simulate request processing
    Ok(Response::ok())
}

async fn process_chunk(chunk: Vec<u8>) -> Result<Vec<u8>> {
    // Simulate chunk processing
    Ok(chunk)
}

// Semaphore for controlling concurrent operations
use tokio::sync::Semaphore;

pub struct RateLimitedProcessor {
    semaphore: Arc<Semaphore>,
}

impl RateLimitedProcessor {
    pub fn new(concurrent_limit: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(concurrent_limit)),
        }
    }
    
    pub async fn process_with_limit<F, R>(&self, operation: F) -> Result<R>
    where
        F: std::future::Future<Output = Result<R>>,
    {
        let _permit = self.semaphore.acquire().await
            .map_err(|_| Error::Server("Semaphore error".to_string()))?;
        
        operation.await
    }
    
    pub async fn process_batch_with_limit<T, F, R>(
        &self,
        items: Vec<T>,
        processor: impl Fn(T) -> F,
    ) -> Result<Vec<R>>
    where
        F: std::future::Future<Output = Result<R>>,
    {
        let mut handles = Vec::new();
        
        for item in items {
            let semaphore = self.semaphore.clone();
            let future = async move {
                let _permit = semaphore.acquire().await
                    .map_err(|_| Error::Server("Semaphore error".to_string()))?;
                
                processor(item).await
            };
            
            handles.push(task::spawn(future));
        }
        
        let mut results = Vec::new();
        
        for handle in handles {
            match handle.await {
                Ok(result) => {
                    if let Ok(val) = result {
                        results.push(val);
                    }
                }
                Err(e) => {
                    eprintln!("Task failed: {}", e);
                }
            }
        }
        
        Ok(results)
    }
}

// Async/await best practices
pub struct AsyncBestPractices;

impl AsyncBestPractices {
    // Use join! for independent operations
    pub async fn parallel_independent_operations() -> Result<()> {
        use tokio::join;
        
        let user_future = fetch_user_data();
        let product_future = fetch_product_data();
        let order_future = fetch_order_data();
        
        let (user_result, product_result, order_result) = join!(user_future, product_future, order_future);
        
        // Process results
        let _user = user_result?;
        let _product = product_result?;
        let _order = order_result?;
        
        Ok(())
    }
    
    // Use select! for racing operations
    pub async fn racing_operations() -> Result<String> {
        use tokio::select;
        
        select! {
            result = fetch_from_primary_db() => {
                Ok(result?)
            }
            result = fetch_from_backup_db() => {
                eprintln!("Primary DB failed, using backup");
                Ok(result?)
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(5)) => {
                Err(Error::Timeout)
            }
        }
    }
    
    // Use spawn_blocking for CPU-intensive work
    pub async fn cpu_intensive_work() -> Result<()> {
        let result = task::spawn_blocking(|| {
            // CPU-intensive work that shouldn't block the async runtime
            perform_cpu_intensive_calculation()
        }).await
        .map_err(|e| Error::Server(e.to_string()))?;
        
        // Process result
        let _processed = result;
        
        Ok(())
    }
}

fn perform_cpu_intensive_calculation() -> String {
    // Simulate CPU-intensive work
    (0..1000).fold(0, |acc, x| acc + x * x).to_string()
}

async fn fetch_user_data() -> Result<String> { Ok("user_data".to_string()) }
async fn fetch_product_data() -> Result<String> { Ok("product_data".to_string()) }
async fn fetch_order_data() -> Result<String> { Ok("order_data".to_string()) }
async fn fetch_from_primary_db() -> Result<String> { Ok("primary_data".to_string()) }
async fn fetch_from_backup_db() -> Result<String> { Ok("backup_data".to_string()) }
```

## Network Optimizations

Optimize network performance:

```rust
use oxidite::prelude::*;

// HTTP/2 and HTTP/3 optimizations
pub struct HttpOptimizations;

impl HttpOptimizations {
    // Enable HTTP/2 server push (when available)
    pub fn configure_http2_server() -> Result<()> {
        // In a real implementation, configure HTTP/2 settings
        // Enable server push, header compression, multiplexing
        
        Ok(())
    }
    
    // Connection reuse and keep-alive
    pub fn configure_keep_alive() -> Result<()> {
        // In a real implementation, configure connection pooling
        // Set appropriate keep-alive timeouts
        
        Ok(())
    }
    
    // Enable compression
    pub fn enable_compression() -> Result<()> {
        // In a real implementation, enable gzip/brotli compression
        // Set appropriate compression levels
        
        Ok(())
    }
}

// Response streaming for large data
async fn stream_large_data(_req: Request) -> Result<Response> {
    use futures::stream::{self, StreamExt};
    use tokio_util::codec::{FramedWrite, LinesCodec};
    use tokio_util::io::StreamReader;
    
    // Create a stream of data chunks
    let chunks: Vec<Result<bytes::Bytes, std::io::Error>> = vec![
        Ok(bytes::Bytes::from("Line 1\n")),
        Ok(bytes::Bytes::from("Line 2\n")),
        Ok(bytes::Bytes::from("Line 3\n")),
    ];
    
    let stream = stream::iter(chunks);
    
    // Convert stream to response
    // In a real implementation, this would create a streaming response
    Ok(Response::text("Streaming response".to_string()))
}

// Chunked transfer encoding for large responses
async fn chunked_response(_req: Request) -> Result<Response> {
    // For responses that are built incrementally
    let mut response_builder = Response::builder();
    
    // Set chunked encoding header
    response_builder.header("Transfer-Encoding", "chunked");
    
    // In a real implementation, this would return a chunked response
    Ok(Response::text("Chunked response content".to_string()))
}

// CDN-friendly headers
async fn cdn_optimized_response(_req: Request) -> Result<Response> {
    let mut response = Response::html("<h1>Hello World</h1>");
    
    // Add CDN-friendly headers
    response.headers_mut().insert(
        "Cache-Control",
        "public, max-age=3600".parse().unwrap() // Cache for 1 hour
    );
    
    response.headers_mut().insert(
        "Vary",
        "Accept-Encoding".parse().unwrap() // Important for compression
    );
    
    // Add ETag for validation
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update("<h1>Hello World</h1>");
    let hash = format!("{:x}", hasher.finalize());
    response.headers_mut().insert(
        "ETag",
        format!("\"{}\"", hash).parse().unwrap()
    );
    
    Ok(response)
}

// Optimized static file serving
use tokio::fs::File;
use tokio::io::AsyncReadExt;

async fn optimized_static_file_handler(Path(file_path): Path<String>) -> Result<Response> {
    // Validate path to prevent directory traversal
    if file_path.contains("..") || file_path.starts_with('/') {
        return Err(Error::BadRequest("Invalid file path".to_string()));
    }
    
    let full_path = format!("public/{}", file_path);
    
    // Check if file exists
    if !std::path::Path::new(&full_path).exists() {
        return Err(Error::NotFound);
    }
    
    // Open file
    let mut file = File::open(&full_path).await
        .map_err(|e| Error::Server(format!("Failed to open file: {}", e)))?;
    
    // Get file metadata
    let metadata = file.metadata().await
        .map_err(|e| Error::Server(format!("Failed to get metadata: {}", e)))?;
    
    // Read file content
    let mut contents = vec![0; metadata.len() as usize];
    file.read_exact(&mut contents).await
        .map_err(|e| Error::Server(format!("Failed to read file: {}", e)))?;
    
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
    
    // Add ETag
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

## Profiling and Monitoring

Monitor and profile your application:

```rust
use oxidite::prelude::*;
use std::sync::Arc;
use tokio::time::{Duration, Instant};

// Performance monitoring middleware
async fn performance_monitoring_middleware(req: Request, next: Next) -> Result<Response> {
    let start_time = Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();
    
    let response = next.run(req).await;
    let elapsed = start_time.elapsed();
    
    // Log performance metrics
    log_performance_metrics(&method, &uri, elapsed, response.as_ref().map(|r| r.status().as_u16()).unwrap_or(500));
    
    response
}

fn log_performance_metrics(method: &http::Method, uri: &http::Uri, elapsed: Duration, status_code: u16) {
    println!(
        "PERFORMANCE - {} {} - {}ms - Status: {}",
        method,
        uri.path(),
        elapsed.as_millis(),
        status_code
    );
    
    // In a real implementation, send to metrics collection system
    // like Prometheus, DataDog, etc.
}

// Request tracing middleware
async fn request_tracing_middleware(req: Request, next: Next) -> Result<Response> {
    let trace_id = uuid::Uuid::new_v4().to_string();
    let span_id = uuid::Uuid::new_v4().to_string();
    
    // Add trace headers to response
    let mut response = next.run(req).await?;
    response.headers_mut().insert(
        "X-Trace-ID",
        trace_id.parse().unwrap()
    );
    response.headers_mut().insert(
        "X-Span-ID",
        span_id.parse().unwrap()
    );
    
    Ok(response)
}

// Memory usage monitoring
use sysinfo::{System, SystemExt, ProcessExt};

pub struct MemoryMonitor;

impl MemoryMonitor {
    pub fn get_memory_usage() -> MemoryUsage {
        let mut system = System::new_all();
        system.refresh_all();
        
        MemoryUsage {
            total_memory: system.total_memory(),
            used_memory: system.used_memory(),
            free_memory: system.free_memory(),
        }
    }
    
    pub fn get_process_memory_usage() -> ProcessMemoryUsage {
        let mut system = System::new_all();
        system.refresh_processes();
        
        if let Some(process) = system.process(sysinfo::get_current_pid().expect("Failed to get PID")) {
            ProcessMemoryUsage {
                memory: process.memory(),
                virtual_memory: process.virtual_memory(),
            }
        } else {
            ProcessMemoryUsage {
                memory: 0,
                virtual_memory: 0,
            }
        }
    }
}

pub struct MemoryUsage {
    pub total_memory: u64,
    pub used_memory: u64,
    pub free_memory: u64,
}

pub struct ProcessMemoryUsage {
    pub memory: u64,
    pub virtual_memory: u64,
}

// Performance metrics endpoint
async fn performance_metrics_endpoint(_req: Request) -> Result<Response> {
    let memory_usage = MemoryMonitor::get_memory_usage();
    let process_memory = MemoryMonitor::get_process_memory_usage();
    
    let metrics = serde_json::json!({
        "memory": {
            "total": memory_usage.total_memory,
            "used": memory_usage.used_memory,
            "free": memory_usage.free_memory,
            "process_used": process_memory.memory,
            "process_virtual": process_memory.virtual_memory
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    Ok(Response::json(metrics))
}

// Slow query detection
pub struct QueryProfiler {
    slow_query_threshold: Duration,
}

impl QueryProfiler {
    pub fn new(slow_query_threshold_ms: u64) -> Self {
        Self {
            slow_query_threshold: Duration::from_millis(slow_query_threshold_ms),
        }
    }
    
    pub async fn execute_with_profiling<F, R>(&self, query_name: &str, operation: F) -> Result<R>
    where
        F: std::future::Future<Output = Result<R>>,
    {
        let start = Instant::now();
        let result = operation.await;
        let elapsed = start.elapsed();
        
        if elapsed > self.slow_query_threshold {
            eprintln!(
                "SLOW QUERY WARNING - {}: {:?}ms",
                query_name,
                elapsed.as_millis()
            );
        }
        
        result
    }
}

// Benchmark utilities
pub struct Benchmarker;

impl Benchmarker {
    pub async fn benchmark<F, R>(name: &str, iterations: usize, operation: F) -> BenchmarkResult
    where
        F: Fn() -> R + Copy,
    {
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _result = operation();
        }
        
        let elapsed = start.elapsed();
        let avg_time = elapsed / iterations as u32;
        
        println!(
            "BENCHMARK - {}: {} iterations in {:?} (avg: {:?} per iteration)",
            name, iterations, elapsed, avg_time
        );
        
        BenchmarkResult {
            name: name.to_string(),
            iterations,
            total_time: elapsed,
            avg_time,
        }
    }
}

pub struct BenchmarkResult {
    pub name: String,
    pub iterations: usize,
    pub total_time: Duration,
    pub avg_time: Duration,
}

// Example benchmark usage
async fn run_benchmarks() -> Result<()> {
    // Benchmark different operations
    Benchmarker::benchmark("string_concatenation", 10000, || {
        let mut s = String::new();
        s.push_str("hello");
        s.push(' ');
        s.push_str("world");
        s
    }).await;
    
    Benchmarker::benchmark("vector_creation", 10000, || {
        let mut v = Vec::with_capacity(10);
        for i in 0..10 {
            v.push(i);
        }
        v
    }).await;
    
    Ok(())
}
```

## Summary

Performance optimization in Oxidite applications involves:

- **Request Handling**: Efficient request processing and middleware
- **Database Optimization**: Query optimization, connection pooling, batching
- **Caching**: In-memory and distributed caching strategies
- **Memory Management**: Efficient data structures and allocation
- **Concurrency**: Proper use of async/await and parallel processing
- **Network Optimization**: HTTP/2, compression, and streaming
- **Profiling**: Monitoring and measuring performance metrics

Following these optimization techniques will help you build fast, efficient Oxidite applications that can handle high loads while maintaining responsiveness.