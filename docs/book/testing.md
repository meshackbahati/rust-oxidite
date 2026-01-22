# Testing

Testing is a crucial part of the Oxidite framework, providing comprehensive tools for unit testing, integration testing, and end-to-end testing. This chapter covers all aspects of testing in Oxidite applications.

## Overview

Oxidite provides:
- Unit testing for individual components
- Integration testing for routes and middleware
- End-to-end testing with simulated HTTP requests
- Test utilities for mocking dependencies
- Test fixtures and factories
- Property-based testing support

## Setting Up Tests

Basic test setup in your project:

```rust
// In your Cargo.toml
[dev-dependencies]
tokio = { version = "1.0", features = ["full"] }
oxidite-testing = "2.0.0"
serial_test = "3.0"

// In your src/lib.rs or src/main.rs
#[cfg(test)]
mod tests {
    use super::*;
    use oxidite_testing::TestServer;
    
    #[tokio::test]
    async fn test_basic_functionality() {
        assert_eq!(2 + 2, 4);
    }
}
```

## Unit Testing

Test individual functions and components:

```rust
use oxidite::prelude::*;

// Function to test
pub fn calculate_discount(price: f64, discount_percent: f64) -> f64 {
    if discount_percent <= 0.0 || discount_percent > 100.0 {
        return price;
    }
    
    price * (1.0 - discount_percent / 100.0)
}

pub fn is_valid_email(email: &str) -> bool {
    email.contains('@') && email.contains('.') && email.len() > 5
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_calculate_discount() {
        assert_eq!(calculate_discount(100.0, 10.0), 90.0);
        assert_eq!(calculate_discount(50.0, 20.0), 40.0);
        assert_eq!(calculate_discount(100.0, 0.0), 100.0);
        assert_eq!(calculate_discount(100.0, 100.0), 0.0);
    }

    #[test]
    fn test_calculate_discount_edge_cases() {
        assert_eq!(calculate_discount(100.0, -10.0), 100.0); // Invalid discount
        assert_eq!(calculate_discount(100.0, 150.0), 100.0); // Too high discount
        assert_eq!(calculate_discount(0.0, 50.0), 0.0); // Zero price
    }

    #[test]
    fn test_is_valid_email() {
        assert!(is_valid_email("user@example.com"));
        assert!(is_valid_email("test.user@domain.co.uk"));
        assert!(!is_valid_email("invalid-email"));
        assert!(!is_valid_email("missing@dot"));
        assert!(!is_valid_email("short@x"));
    }
}
```

## Integration Testing

Test routes and middleware integration:

```rust
use oxidite::prelude::*;
use oxidite_testing::{TestServer, RequestBuilder};

// Sample route handler
async fn hello_handler(_req: Request) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "message": "Hello, World!",
        "status": "success"
    })))
}

async fn user_handler(Path(user_id): Path<u32>) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "id": user_id,
        "name": format!("User {}", user_id),
        "email": format!("user{}@example.com", user_id)
    })))
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_hello_endpoint() {
        let server = TestServer::new(|router| {
            router.get("/hello", hello_handler);
        }).await;

        let response = server.get("/hello").send().await;
        
        assert_eq!(response.status(), 200);
        
        let json: serde_json::Value = response.json().await;
        assert_eq!(json["message"], "Hello, World!");
        assert_eq!(json["status"], "success");
    }

    #[tokio::test]
    async fn test_user_endpoint() {
        let server = TestServer::new(|router| {
            router.get("/users/:id", user_handler);
        }).await;

        let response = server.get("/users/123").send().await;
        
        assert_eq!(response.status(), 200);
        
        let json: serde_json::Value = response.json().await;
        assert_eq!(json["id"], 123);
        assert_eq!(json["name"], "User 123");
        assert_eq!(json["email"], "user123@example.com");
    }

    #[tokio::test]
    async fn test_not_found() {
        let server = TestServer::new(|router| {
            router.get("/hello", hello_handler);
        }).await;

        let response = server.get("/nonexistent").send().await;
        
        assert_eq!(response.status(), 404);
    }
}
```

## Testing with State and Dependencies

Test routes that use application state:

```rust
use oxidite::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    app_name: String,
    version: String,
}

async fn stateful_handler(
    _req: Request,
    State(state): State<Arc<AppState>>
) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "app_name": state.app_name,
        "version": state.version
    })))
}

#[cfg(test)]
mod state_tests {
    use super::*;

    #[tokio::test]
    async fn test_stateful_handler() {
        let app_state = Arc::new(AppState {
            app_name: "Test App".to_string(),
            version: "1.0.0".to_string(),
        });

        let server = TestServer::new(move |router| {
            let state_clone = app_state.clone();
            router.with_state(state_clone);
            router.get("/info", stateful_handler);
        }).await;

        let response = server.get("/info").send().await;
        
        assert_eq!(response.status(), 200);
        
        let json: serde_json::Value = response.json().await;
        assert_eq!(json["app_name"], "Test App");
        assert_eq!(json["version"], "1.0.0");
    }
}
```

## Testing Middleware

Test middleware functionality:

```rust
use oxidite::prelude::*;

async fn logging_middleware(req: Request, next: Next) -> Result<Response> {
    println!("Request: {} {}", req.method(), req.uri());
    let response = next.run(req).await?;
    println!("Response: {}", response.status());
    Ok(response)
}

async fn auth_middleware(req: Request, next: Next) -> Result<Response> {
    // Check for auth header
    let auth_header = req.headers()
        .get("authorization")
        .and_then(|hv| hv.to_str().ok());
    
    if auth_header.is_none() {
        return Err(Error::Unauthorized("Missing authorization header".to_string()));
    }
    
    next.run(req).await
}

#[cfg(test)]
mod middleware_tests {
    use super::*;

    #[tokio::test]
    async fn test_auth_middleware_success() {
        let server = TestServer::new(|router| {
            router.get("/protected")
                .middleware(auth_middleware)
                .handler(|_req| async { Ok(Response::text("Protected content".to_string())) });
        }).await;

        let response = server
            .get("/protected")
            .header("Authorization", "Bearer token123")
            .send()
            .await;
        
        assert_eq!(response.status(), 200);
        assert_eq!(response.text().await, "Protected content");
    }

    #[tokio::test]
    async fn test_auth_middleware_failure() {
        let server = TestServer::new(|router| {
            router.get("/protected")
                .middleware(auth_middleware)
                .handler(|_req| async { Ok(Response::text("Protected content".to_string())) });
        }).await;

        let response = server.get("/protected").send().await;
        
        assert_eq!(response.status(), 401);
    }
}
```

## Database Testing

Test database operations with test databases:

```rust
use oxidite::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Model, Serialize, Deserialize)]
#[model(table = "test_users")]
pub struct TestUser {
    #[model(primary_key)]
    pub id: i32,
    #[model(unique, not_null)]
    pub email: String,
    #[model(not_null)]
    pub name: String,
}

#[cfg(test)]
mod database_tests {
    use super::*;

    async fn setup_test_db() -> Result<()> {
        // Create test database schema
        // This would typically run migrations or create tables
        Ok(())
    }

    async fn teardown_test_db() -> Result<()> {
        // Clean up test database
        Ok(())
    }

    #[tokio::test]
    async fn test_user_crud_operations() {
        setup_test_db().await.unwrap();

        // Test create
        let user = TestUser {
            id: 0,
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
        };

        let saved_user = user.save().await.unwrap();
        assert!(!saved_user.id == 0);

        // Test read
        let found_user = TestUser::find_by_id(saved_user.id).await.unwrap().unwrap();
        assert_eq!(found_user.email, "test@example.com");
        assert_eq!(found_user.name, "Test User");

        // Test update
        let mut updated_user = found_user;
        updated_user.name = "Updated Name".to_string();
        let updated_user = updated_user.save().await.unwrap();
        assert_eq!(updated_user.name, "Updated Name");

        // Test delete
        updated_user.delete().await.unwrap();
        let deleted_user = TestUser::find_by_id(updated_user.id).await.unwrap();
        assert!(deleted_user.is_none());

        teardown_test_db().await.unwrap();
    }

    #[tokio::test]
    async fn test_duplicate_email_fails() {
        setup_test_db().await.unwrap();

        let user1 = TestUser {
            id: 0,
            email: "duplicate@example.com".to_string(),
            name: "User 1".to_string(),
        };
        user1.save().await.unwrap();

        let user2 = TestUser {
            id: 0,
            email: "duplicate@example.com".to_string(), // Same email
            name: "User 2".to_string(),
        };
        
        // This should fail due to unique constraint
        let result = user2.save().await;
        assert!(result.is_err());

        teardown_test_db().await.unwrap();
    }
}
```

## Mocking and Test Doubles

Create mocks for external dependencies:

```rust
use oxidite::prelude::*;

// Service to be mocked
pub trait EmailService: Send + Sync {
    async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), String>;
}

pub struct RealEmailService;

impl EmailService for RealEmailService {
    async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), String> {
        // Actually send email
        println!("Sending email to: {}, subject: {}, body: {}", to, subject, body);
        Ok(())
    }
}

// Handler that uses the service
async fn contact_handler(
    Json(payload): Json<ContactRequest>,
    State(email_service): State<Arc<dyn EmailService>>
) -> Result<Response> {
    email_service
        .send_email(&payload.email, &payload.subject, &payload.message)
        .await
        .map_err(|e| Error::Server(e))?;

    Ok(Response::json(serde_json::json!({
        "status": "sent",
        "message": "Email sent successfully"
    })))
}

#[derive(serde::Deserialize)]
struct ContactRequest {
    email: String,
    subject: String,
    message: String,
}

// Mock implementation for testing
pub struct MockEmailService {
    pub sent_emails: std::sync::Arc<tokio::sync::Mutex<Vec<SentEmail>>>,
}

#[derive(Clone)]
pub struct SentEmail {
    pub to: String,
    pub subject: String,
    pub body: String,
}

impl MockEmailService {
    pub fn new() -> Self {
        Self {
            sent_emails: std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }
    
    pub async fn get_sent_emails(&self) -> Vec<SentEmail> {
        self.sent_emails.lock().await.clone()
    }
}

#[async_trait::async_trait]
impl EmailService for MockEmailService {
    async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), String> {
        let mut emails = self.sent_emails.lock().await;
        emails.push(SentEmail {
            to: to.to_string(),
            subject: subject.to_string(),
            body: body.to_string(),
        });
        Ok(())
    }
}

#[cfg(test)]
mod mock_tests {
    use super::*;

    #[tokio::test]
    async fn test_contact_handler_with_mock() {
        let mock_service = std::sync::Arc::new(MockEmailService::new());
        let service_clone = mock_service.clone();

        let server = TestServer::new(move |router| {
            router.post("/contact")
                .with_state(service_clone.clone() as Arc<dyn EmailService>)
                .handler(contact_handler);
        }).await;

        let response = server
            .post("/contact")
            .json(&serde_json::json!({
                "email": "user@example.com",
                "subject": "Test Subject",
                "message": "Test message"
            }))
            .send()
            .await;

        assert_eq!(response.status(), 200);
        
        let json: serde_json::Value = response.json().await;
        assert_eq!(json["status"], "sent");

        // Verify email was sent via mock
        let sent_emails = mock_service.get_sent_emails().await;
        assert_eq!(sent_emails.len(), 1);
        assert_eq!(sent_emails[0].to, "user@example.com");
        assert_eq!(sent_emails[0].subject, "Test Subject");
        assert_eq!(sent_emails[0].body, "Test message");
    }
}
```

## Property-Based Testing

Use property-based testing for comprehensive validation:

```rust
use oxidite::prelude::*;

// Function to test with property-based testing
pub fn reverse_string(s: &str) -> String {
    s.chars().rev().collect()
}

pub fn is_palindrome(s: &str) -> bool {
    let cleaned: String = s.chars()
        .filter(|c| c.is_alphanumeric())
        .map(|c| c.to_lowercase().next().unwrap())
        .collect();
    
    cleaned == reverse_string(&cleaned)
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Test that reversing a string twice gives the original
    proptest! {
        #[test]
        fn test_reverse_twice_is_identity(s in ".*") {
            let reversed_once = reverse_string(&s);
            let reversed_twice = reverse_string(&reversed_once);
            prop_assert_eq!(s, reversed_twice);
        }
    }

    // Test palindrome properties
    proptest! {
        #[test]
        fn test_palindromes(s in "[a-zA-Z]{1,10}") {
            // A string concatenated with its reverse should be a palindrome
            let reversed = reverse_string(&s);
            let palindrome = format!("{}{}", s, reversed);
            prop_assert!(is_palindrome(&palindrome));
        }
    }
}
```

## Test Fixtures and Factories

Create reusable test data:

```rust
use oxidite::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Model, Serialize, Deserialize, Clone)]
#[model(table = "test_posts")]
pub struct TestPost {
    #[model(primary_key)]
    pub id: i32,
    #[model(not_null)]
    pub title: String,
    #[model(not_null)]
    pub content: String,
    pub user_id: i32,
}

#[derive(Model, Serialize, Deserialize, Clone)]
#[model(table = "test_comments")]
pub struct TestComment {
    #[model(primary_key)]
    pub id: i32,
    #[model(not_null)]
    pub content: String,
    pub post_id: i32,
    pub user_id: i32,
}

// Test factory for creating test data
pub struct TestFactory;

impl TestFactory {
    pub fn create_user(email: &str, name: &str) -> TestUser {
        TestUser {
            id: 0,
            email: email.to_string(),
            name: name.to_string(),
        }
    }

    pub fn create_post(title: &str, content: &str, user_id: i32) -> TestPost {
        TestPost {
            id: 0,
            title: title.to_string(),
            content: content.to_string(),
            user_id,
        }
    }

    pub fn create_comment(content: &str, post_id: i32, user_id: i32) -> TestComment {
        TestComment {
            id: 0,
            content: content.to_string(),
            post_id,
            user_id,
        }
    }
}

#[cfg(test)]
mod fixture_tests {
    use super::*;

    #[tokio::test]
    async fn test_blog_post_with_comments() {
        setup_test_db().await.unwrap();

        // Create test data using factory
        let user = TestFactory::create_user("author@example.com", "Author Name");
        let saved_user = user.save().await.unwrap();

        let post = TestFactory::create_post("Test Post", "Post content", saved_user.id);
        let saved_post = post.save().await.unwrap();

        let comment = TestFactory::create_comment("Great post!", saved_post.id, saved_user.id);
        let saved_comment = comment.save().await.unwrap();

        // Verify relationships
        assert_eq!(saved_comment.post_id, saved_post.id);
        assert_eq!(saved_comment.user_id, saved_user.id);

        // Clean up
        saved_comment.delete().await.unwrap();
        saved_post.delete().await.unwrap();
        saved_user.delete().await.unwrap();

        teardown_test_db().await.unwrap();
    }
}
```

## Test Configuration

Configure test-specific settings:

```rust
// In your Cargo.toml
[features]
test_utils = []

// Test utilities module
#[cfg(any(test, feature = "test_utils"))]
pub mod test_utils {
    use oxidite::prelude::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[derive(Clone)]
    pub struct TestContext {
        pub db_url: String,
        pub temp_dir: tempfile::TempDir,
        pub cleanup_hooks: Arc<Mutex<Vec<Box<dyn FnMut() -> () + Send>>>>,
    }

    impl TestContext {
        pub async fn new() -> Self {
            let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
            Self {
                db_url: format!("sqlite://{}/test.db", temp_dir.path().display()),
                temp_dir,
                cleanup_hooks: Arc::new(Mutex::new(Vec::new())),
            }
        }

        pub async fn add_cleanup_hook<F>(&self, hook: F) 
        where 
            F: FnMut() -> () + Send + 'static 
        {
            let mut hooks = self.cleanup_hooks.lock().await;
            hooks.push(Box::new(hook));
        }

        pub async fn run_cleanup(&self) {
            let mut hooks = self.cleanup_hooks.lock().await;
            for hook in hooks.iter_mut() {
                hook();
            }
        }
    }

    // Test server wrapper with context
    pub struct TestServerWithContext {
        pub server: TestServer,
        pub context: TestContext,
    }

    impl TestServerWithContext {
        pub async fn new<F>(setup_fn: F) -> Self 
        where 
            F: FnOnce(&mut Router, TestContext) + Send + 'static 
        {
            let context = TestContext::new().await;
            let context_clone = context.clone();
            
            let server = TestServer::new(move |router| {
                setup_fn(router, context_clone);
            }).await;

            Self { server, context }
        }
    }
}

#[cfg(test)]
mod configured_tests {
    use super::*;
    use test_utils::*;

    #[tokio::test]
    async fn test_with_context() {
        let test_server = TestServerWithContext::new(|router, _ctx| {
            router.get("/test", |_req| async { 
                Ok(Response::text("Test response".to_string())) 
            });
        }).await;

        let response = test_server.server.get("/test").send().await;
        assert_eq!(response.status(), 200);
        assert_eq!(response.text().await, "Test response");

        test_server.context.run_cleanup().await;
    }
}
```

## Parallel Test Execution

Handle parallel test execution safely:

```rust
use oxidite::prelude::*;
use serial_test::serial;

// Use serial_test attribute for tests that can't run in parallel
#[tokio::test]
#[serial]
async fn test_shared_resource() {
    // This test accesses a shared resource and must run serially
    // For example, a test that modifies global configuration
    println!("Running serial test");
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_independent_functionality() {
    // This test can run in parallel with others
    assert_eq!(2 + 2, 4);
}

// Test isolation utilities
pub mod test_isolation {
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

    pub fn get_unique_test_id() -> String {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("test_{}", id)
    }

    pub fn get_unique_table_name() -> String {
        format!("test_table_{}", get_unique_test_id())
    }

    pub fn get_unique_db_name() -> String {
        format!("test_db_{}.db", get_unique_test_id())
    }
}

#[cfg(test)]
mod isolated_tests {
    use super::*;
    use test_isolation::*;

    #[tokio::test]
    async fn test_with_unique_resources() {
        let unique_id = get_unique_test_id();
        let table_name = get_unique_table_name();
        
        println!("Using unique resources: {} - {}", unique_id, table_name);
        
        // Test using isolated resources
        assert!(table_name.starts_with("test_table_test_"));
    }
}
```

## Test Coverage

Measure and improve test coverage:

```rust
// In your .cargo/config.toml
// [target.'cfg(coverage)']
// rustflags = ["-Zinstrument-coverage"]

use oxidite::prelude::*;

// Complex function to test thoroughly
pub fn process_order(
    amount: f64,
    tax_rate: f64,
    discount_percent: f64,
    shipping_cost: f64,
    is_international: bool
) -> Result<OrderSummary, String> {
    if amount <= 0.0 {
        return Err("Amount must be positive".to_string());
    }
    
    if tax_rate < 0.0 || tax_rate > 1.0 {
        return Err("Tax rate must be between 0 and 1".to_string());
    }
    
    if discount_percent < 0.0 || discount_percent > 100.0 {
        return Err("Discount percent must be between 0 and 100".to_string());
    }
    
    let discount_amount = amount * (discount_percent / 100.0);
    let subtotal = amount - discount_amount;
    let tax_amount = subtotal * tax_rate;
    let total = subtotal + tax_amount + shipping_cost;
    
    let international_fee = if is_international { total * 0.05 } else { 0.0 };
    let final_total = total + international_fee;

    Ok(OrderSummary {
        subtotal,
        tax_amount,
        shipping_cost,
        discount_amount,
        international_fee,
        total: final_total,
    })
}

#[derive(Debug, PartialEq)]
pub struct OrderSummary {
    pub subtotal: f64,
    pub tax_amount: f64,
    pub shipping_cost: f64,
    pub discount_amount: f64,
    pub international_fee: f64,
    pub total: f64,
}

#[cfg(test)]
mod coverage_tests {
    use super::*;

    #[test]
    fn test_process_order_normal_case() {
        let result = process_order(100.0, 0.1, 10.0, 5.0, false).unwrap();
        
        assert_eq!(result.subtotal, 90.0); // 100 - 10% discount
        assert_eq!(result.tax_amount, 9.0); // 90 * 10% tax
        assert_eq!(result.shipping_cost, 5.0);
        assert_eq!(result.discount_amount, 10.0);
        assert_eq!(result.international_fee, 0.0);
        assert_eq!(result.total, 104.0); // 90 + 9 + 5 + 0
    }

    #[test]
    fn test_process_order_international() {
        let result = process_order(100.0, 0.1, 0.0, 5.0, true).unwrap();
        
        assert_eq!(result.subtotal, 100.0);
        assert_eq!(result.tax_amount, 10.0);
        assert_eq!(result.international_fee, 5.75); // (100 + 10 + 5) * 5%
        assert_eq!(result.total, 120.75);
    }

    #[test]
    fn test_process_order_zero_values() {
        let result = process_order(100.0, 0.0, 0.0, 0.0, false).unwrap();
        
        assert_eq!(result.subtotal, 100.0);
        assert_eq!(result.tax_amount, 0.0);
        assert_eq!(result.total, 100.0);
    }

    #[test]
    fn test_process_order_edge_cases() {
        // Test with very small values
        let result = process_order(0.01, 0.01, 0.01, 0.01, false).unwrap();
        assert!(result.total > 0.0);
        
        // Test maximum values within bounds
        let result = process_order(1000000.0, 0.99, 99.99, 1000.0, true).unwrap();
        assert!(result.total > 0.0);
    }

    #[test]
    fn test_process_order_errors() {
        // Test negative amount
        assert!(process_order(-1.0, 0.1, 10.0, 5.0, false).is_err());
        
        // Test invalid tax rate
        assert!(process_order(100.0, -0.1, 10.0, 5.0, false).is_err());
        assert!(process_order(100.0, 1.5, 10.0, 5.0, false).is_err());
        
        // Test invalid discount percent
        assert!(process_order(100.0, 0.1, -1.0, 5.0, false).is_err());
        assert!(process_order(100.0, 0.1, 101.0, 5.0, false).is_err());
    }
}
```

## Test Reporting

Generate test reports and summaries:

```rust
use oxidite::prelude::*;

// Test result aggregator
#[derive(Default)]
pub struct TestResults {
    pub passed: usize,
    pub failed: usize,
    pub ignored: usize,
    pub measured: usize,
}

impl TestResults {
    pub fn add_result(&mut self, result: TestResult) {
        match result.status {
            TestStatus::Passed => self.passed += 1,
            TestStatus::Failed => self.failed += 1,
            TestStatus::Ignored => self.ignored += 1,
            TestStatus::Measured => self.measured += 1,
        }
    }
    
    pub fn total(&self) -> usize {
        self.passed + self.failed + self.ignored + self.measured
    }
    
    pub fn success_rate(&self) -> f64 {
        if self.total() == 0 {
            0.0
        } else {
            (self.passed as f64 / self.total() as f64) * 100.0
        }
    }
    
    pub fn print_summary(&self) {
        println!("Test Results Summary:");
        println!("  Total: {}", self.total());
        println!("  Passed: {} ({:.1}%)", self.passed, self.success_rate());
        println!("  Failed: {}", self.failed);
        println!("  Ignored: {}", self.ignored);
        println!("  Measured: {}", self.measured);
    }
}

pub struct TestResult {
    pub name: String,
    pub status: TestStatus,
    pub duration: std::time::Duration,
    pub error: Option<String>,
}

pub enum TestStatus {
    Passed,
    Failed,
    Ignored,
    Measured,
}

// Example of integrating with a test runner
pub struct TestRunner {
    pub results: TestResults,
}

impl TestRunner {
    pub fn new() -> Self {
        Self {
            results: TestResults::default(),
        }
    }
    
    pub async fn run_test<F>(&mut self, name: &str, test_fn: F) 
    where 
        F: std::future::Future<Output = Result<(), String>> 
    {
        let start = std::time::Instant::now();
        
        match test_fn.await {
            Ok(()) => {
                let result = TestResult {
                    name: name.to_string(),
                    status: TestStatus::Passed,
                    duration: start.elapsed(),
                    error: None,
                };
                self.results.add_result(result);
            }
            Err(error) => {
                let result = TestResult {
                    name: name.to_string(),
                    status: TestStatus::Failed,
                    duration: start.elapsed(),
                    error: Some(error),
                };
                self.results.add_result(result);
            }
        }
    }
}

#[cfg(test)]
mod runner_tests {
    use super::*;

    #[tokio::test]
    async fn test_runner_functionality() {
        let mut runner = TestRunner::new();
        
        // Run a passing test
        runner.run_test("passing_test", async { Ok(()) }).await;
        
        // Run a failing test
        runner.run_test("failing_test", async { 
            Err("Test failed intentionally".to_string()) 
        }).await;
        
        // Run another passing test
        runner.run_test("another_passing_test", async { Ok(()) }).await;
        
        assert_eq!(runner.results.passed, 2);
        assert_eq!(runner.results.failed, 1);
        assert_eq!(runner.results.total(), 3);
        
        let success_rate = runner.results.success_rate();
        assert_eq!(success_rate, 66.66666666666666);
    }
}
```

## Summary

Testing in Oxidite provides comprehensive tools for:

- **Unit Testing**: Individual function and component testing
- **Integration Testing**: Route and middleware integration
- **Database Testing**: ORM and database operation testing
- **Mocking**: External dependency simulation
- **Property-Based Testing**: Comprehensive validation
- **Fixtures**: Reusable test data creation
- **Parallel Execution**: Safe concurrent test running
- **Coverage Analysis**: Thorough testing measurement
- **Reporting**: Detailed test results and summaries

Following testing best practices ensures reliable, maintainable Oxidite applications with high quality and confidence in code changes.