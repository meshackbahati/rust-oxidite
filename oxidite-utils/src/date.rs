//! Date and time utilities

use chrono::{DateTime, Utc, NaiveDateTime};

pub use chrono::Duration;

/// Get the current UTC timestamp
pub fn now() -> DateTime<Utc> {
    Utc::now()
}

/// Format a datetime to a string
pub fn format_date(dt: &DateTime<Utc>, format: &str) -> String {
    dt.format(format).to_string()
}

/// Parse a date string
pub fn parse_date(s: &str, format: &str) -> Option<DateTime<Utc>> {
    NaiveDateTime::parse_from_str(s, format)
        .ok()
        .map(|dt| dt.and_utc())
}

/// Get Unix timestamp in seconds
pub fn unix_timestamp() -> i64 {
    Utc::now().timestamp()
}

/// Get Unix timestamp in milliseconds
pub fn unix_timestamp_millis() -> i64 {
    Utc::now().timestamp_millis()
}

/// Check if a timestamp is expired
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
