//! Date and time utilities
//!
//! Provides helpers for timestamps, formatting, parsing, and expiry checks.
//!
//! # Examples
//!
//! ```rust
//! use oxidite_utils::date::{now, format_date, unix_timestamp};
//!
//! let formatted = format_date(&now(), "%Y-%m-%d");
//! assert_eq!(formatted.len(), 10);
//! ```

use chrono::{DateTime, Utc, NaiveDateTime};

/// Re-export of `chrono::Duration` for use in expiry calculations
pub use chrono::Duration;

/// Get the current UTC timestamp
///
/// ```rust
/// use oxidite_utils::date::now;
///
/// let dt = now();
/// assert!(dt.timestamp() > 0);
/// ```
pub fn now() -> DateTime<Utc> {
    Utc::now()
}

/// Format a datetime to a string using a strftime-style format string
///
/// ```rust
/// use oxidite_utils::date::{now, format_date};
///
/// let formatted = format_date(&now(), "%Y-%m-%d");
/// assert_eq!(formatted.len(), 10);
/// ```
pub fn format_date(dt: &DateTime<Utc>, format: &str) -> String {
    dt.format(format).to_string()
}

/// Parse a date string using a strftime-style format string
///
/// Returns `None` if parsing fails.
///
/// ```rust
/// use oxidite_utils::date::parse_date;
///
/// let dt = parse_date("2024-01-15 10:30:00", "%Y-%m-%d %H:%M:%S")
///     .expect("valid date string");
/// assert_eq!(dt.timestamp(), 1705314600);
/// ```
pub fn parse_date(s: &str, format: &str) -> Option<DateTime<Utc>> {
    NaiveDateTime::parse_from_str(s, format)
        .ok()
        .map(|dt| dt.and_utc())
}

/// Get Unix timestamp in seconds
///
/// ```rust
/// use oxidite_utils::date::unix_timestamp;
///
/// let ts = unix_timestamp();
/// assert!(ts > 1_700_000_000);
/// ```
pub fn unix_timestamp() -> i64 {
    Utc::now().timestamp()
}

/// Get Unix timestamp in milliseconds
///
/// ```rust
/// use oxidite_utils::date::unix_timestamp_millis;
///
/// let ts = unix_timestamp_millis();
/// assert!(ts > 1_700_000_000_000);
/// ```
pub fn unix_timestamp_millis() -> i64 {
    Utc::now().timestamp_millis()
}

/// Check if a timestamp (Unix seconds) has expired relative to now
///
/// ```rust
/// use oxidite_utils::date::{unix_timestamp, is_expired};
///
/// let past = unix_timestamp() - 100;
/// let future = unix_timestamp() + 100;
/// assert!(is_expired(past));
/// assert!(!is_expired(future));
/// ```
pub fn is_expired(expires_at: i64) -> bool {
    unix_timestamp() >= expires_at
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now() {
        let dt = now();
        assert!(dt.timestamp() > 0);
    }

    #[test]
    fn test_format_date() {
        let dt = now();
        let formatted = format_date(&dt, "%Y-%m-%d");
        assert!(formatted.len() == 10);
    }

    #[test]
    fn test_is_expired() {
        let past = unix_timestamp() - 100;
        let future = unix_timestamp() + 100;

        assert!(is_expired(past));
        assert!(!is_expired(future));
    }
}
