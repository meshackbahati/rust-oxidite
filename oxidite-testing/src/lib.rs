//! Testing utilities for Oxidite web applications
//!
//! This crate provides test helpers, mocks, and utilities for testing
//! Oxidite web applications.
//!
//! # Examples
//!
//! ```no_run
//! use oxidite_testing::*;
//!
//! #[tokio::test]
//! async fn test_endpoint() {
//!     let request = TestRequest::get("/api/users").build();
//!     // Test your handlers
//! }
//! ```

/// Test request builder module — provides [`TestRequest`] and [`TestRequestError`].
pub mod request;
/// Test response wrapper module — provides [`TestResponse`].
pub mod response;
/// Test server module — provides [`TestServer`] and [`test_router`].
pub mod server;

/// Re-export [`TestRequest`] and [`TestRequestError`] at crate root.
pub use request::{TestRequest, TestRequestError};
/// Re-export [`TestResponse`] at crate root.
pub use response::TestResponse;
/// Re-export [`TestServer`] and [`test_router`] at crate root.
pub use server::{TestServer, test_router};

/// Re-export tokio::test for convenience
pub use tokio::test;
