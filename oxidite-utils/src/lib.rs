//! # Oxidite Utils
//!
//! Common utilities for the Oxidite framework including string helpers,
//! date/time utilities, ID generation, and validation helpers.
//!
//! # Usage
//!
//! ```rust
//! use oxidite_utils::generate_uuid;
//!
//! let id = generate_uuid();
//! assert_eq!(id.len(), 36);
//! ```

/// Date and time utilities — `now`, `format_date`, `parse_date`, timestamps, expiry checks
///
/// ```rust
/// use oxidite_utils::date::{now, unix_timestamp, is_expired};
///
/// let ts = unix_timestamp();
/// assert!(!is_expired(ts + 3600));
/// ```
pub mod date;

/// ID generation — UUIDv4, short IDs, numeric IDs
///
/// ```rust
/// use oxidite_utils::id::generate_uuid;
///
/// let id = generate_uuid();
/// assert!(id.contains('-'));
/// ```
pub mod id;

/// String manipulation — slugify, truncate, capitalize, random strings, case conversion
///
/// ```rust
/// use oxidite_utils::string::slugify;
///
/// assert_eq!(slugify("Hello World"), "hello-world");
/// ```
pub mod string;

/// Validation helpers — email, URL, phone, alphanumeric, length checks
///
/// ```rust
/// use oxidite_utils::validation::is_email;
///
/// assert!(is_email("user@example.com"));
/// ```
pub mod validation;

pub use date::{
    now, format_date, parse_date, unix_timestamp, unix_timestamp_millis, is_expired, Duration,
};
pub use id::{generate_id, generate_uuid, generate_short_id, generate_numeric_id};
pub use string::{slugify, truncate, capitalize, random_string, camel_case, snake_case};
pub use validation::{
    is_email, is_url, is_phone, is_alphanumeric, is_numeric, min_length, max_length,
    length_between,
};

/// Request-level metrics registry — route counts, durations, error tracking
///
/// ```rust
/// use oxidite_utils::metrics::{MetricsRegistry, RouteMetrics};
///
/// let registry = MetricsRegistry::new();
/// registry.record_request("/api/health", 42, true);
/// let snapshot = registry.get_snapshot();
/// assert_eq!(snapshot.get("/api/health"), Some(&(1, 1, 0, 42)));
/// ```
pub mod metrics;
pub use metrics::{GLOBAL_METRICS, MetricsRegistry, RouteMetrics};
