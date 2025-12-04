//! Validation utilities

use regex::Regex;

/// Check if a string is a valid email address
pub fn is_email(s: &str) -> bool {
    let re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    re.is_match(s)
}

/// Check if a string is a valid URL
pub fn is_url(s: &str) -> bool {
    let re = Regex::new(r"^https?://[a-zA-Z0-9][-a-zA-Z0-9]*(\.[a-zA-Z0-9][-a-zA-Z0-9]*)*(:\d+)?(/.*)?$").unwrap();
    re.is_match(s)
}

/// Check if a string is a valid phone number (basic international format)
pub fn is_phone(s: &str) -> bool {
    let re = Regex::new(r"^\+?[0-9]{10,15}$").unwrap();
    re.is_match(s.replace(['-', ' ', '(', ')'], "").as_str())
}

/// Check if a string is alphanumeric
pub fn is_alphanumeric(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_alphanumeric())
}

/// Check if a string is numeric
pub fn is_numeric(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_numeric())
}

/// Check if a string has minimum length
pub fn min_length(s: &str, min: usize) -> bool {
    s.len() >= min
}

/// Check if a string has maximum length
pub fn max_length(s: &str, max: usize) -> bool {
    s.len() <= max
}

/// Check if a string is within length bounds
pub fn length_between(s: &str, min: usize, max: usize) -> bool {
    min_length(s, min) && max_length(s, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_email() {
        assert!(is_email("test@example.com"));
        assert!(is_email("user.name+tag@domain.co.uk"));
        assert!(!is_email("invalid"));
        assert!(!is_email("@example.com"));
    }

    #[test]
    fn test_is_url() {
        assert!(is_url("https://example.com"));
        assert!(is_url("http://localhost:3000/path"));
        assert!(!is_url("not-a-url"));
        assert!(!is_url("ftp://example.com"));
    }

    #[test]
    fn test_is_phone() {
        assert!(is_phone("+1234567890"));
        assert!(is_phone("123-456-7890"));
        assert!(is_phone("(123) 456-7890"));
        assert!(!is_phone("12345"));
    }

    #[test]
    fn test_length_validators() {
        assert!(min_length("hello", 3));
        assert!(!min_length("hi", 3));
        assert!(max_length("hi", 5));
        assert!(!max_length("hello world", 5));
        assert!(length_between("hello", 3, 10));
    }
}
