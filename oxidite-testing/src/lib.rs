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

pub mod request;
pub mod response;
pub mod server;

pub use request::TestRequest;
pub use response::TestResponse;
pub use server::{TestServer, test_router};

/// Re-export tokio::test for convenience
pub use tokio::test;
