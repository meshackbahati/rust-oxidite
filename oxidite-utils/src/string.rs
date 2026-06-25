//! String manipulation utilities
//!
//! Provides functions for slugification, truncation, capitalization,
//! random string generation, and case conversion (camelCase, snake_case).
//!
//! # Examples
//!
//! ```rust
//! use oxidite_utils::string::{slugify, truncate, capitalize, random_string, camel_case, snake_case};
//!
//! assert_eq!(slugify("Hello World"), "hello-world");
//! assert_eq!(truncate("Hello World", 8), "Hello...");
//! assert_eq!(capitalize("hello"), "Hello");
//! assert_eq!(camel_case("hello_world"), "helloWorld");
//! ```

use rand::Rng;

/// Convert a string to a URL-friendly slug
///
/// Replaces whitespace, hyphens, and underscores with `-`, strips non-alphanumeric
/// characters, and lowercases the result.
///
/// ```rust
/// use oxidite_utils::string::slugify;
///
/// assert_eq!(slugify("Hello World!"), "hello-world");
/// assert_eq!(slugify("  Multiple   Spaces  "), "multiple-spaces");
/// ```
pub fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                '\0'
            }
        })
        .filter(|c| *c != '\0')
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Truncate a string to a maximum length, appending `"..."` if truncated
///
/// The ellipsis is counted in the max length. If `max_len` ≤ 3, no ellipsis is added.
///
/// ```rust
/// use oxidite_utils::string::truncate;
///
/// assert_eq!(truncate("Hello World", 5), "He...");
/// assert_eq!(truncate("Hi", 5), "Hi");
/// assert_eq!(truncate("Hello", 5), "Hello");
/// ```
pub fn truncate(s: &str, max_len: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        s.chars().take(max_len).collect()
    } else {
        format!("{}...", s.chars().take(max_len - 3).collect::<String>())
    }
}

/// Capitalize the first character of a string
///
/// ```rust
/// use oxidite_utils::string::capitalize;
///
/// assert_eq!(capitalize("hello"), "Hello");
/// assert_eq!(capitalize(""), "");
/// ```
pub fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

/// Generate a random alphanumeric string of a specified length
///
/// Uses `A-Z`, `a-z`, `0-9` as the character set.
///
/// ```rust
/// use oxidite_utils::string::random_string;
///
/// let s = random_string(16);
/// assert_eq!(s.len(), 16);
/// assert!(s.chars().all(|c| c.is_alphanumeric()));
/// ```
pub fn random_string(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();

    (0..length)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Convert a delimited string (snake_case, kebab-case, or space-separated) to camelCase
///
/// ```rust
/// use oxidite_utils::string::camel_case;
///
/// assert_eq!(camel_case("hello_world"), "helloWorld");
/// assert_eq!(camel_case("user-name"), "userName");
/// assert_eq!(camel_case("foo bar"), "fooBar");
/// ```
pub fn camel_case(s: &str) -> String {
    let words: Vec<&str> = s.split(|c: char| c == '_' || c == '-' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .collect();

    if words.is_empty() {
        return String::new();
    }

    let mut result = words[0].to_lowercase();
    for word in &words[1..] {
        result.push_str(&capitalize(&word.to_lowercase()));
    }
    result
}

/// Convert a camelCase or kebab-case string to snake_case
///
/// ```rust
/// use oxidite_utils::string::snake_case;
///
/// assert_eq!(snake_case("helloWorld"), "hello_world");
/// assert_eq!(snake_case("UserName"), "user_name");
/// ```
pub fn snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else if c == '-' || c.is_whitespace() {
            result.push('_');
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("This is a Test!"), "this-is-a-test");
        assert_eq!(slugify("  Multiple   Spaces  "), "multiple-spaces");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("Hello World", 5), "He...");
        assert_eq!(truncate("Hi", 5), "Hi");
        assert_eq!(truncate("Hello", 5), "Hello");
        assert_eq!(truncate("éééé", 3), "ééé");
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("hello"), "Hello");
        assert_eq!(capitalize("HELLO"), "HELLO");
        assert_eq!(capitalize(""), "");
    }

    #[test]
    fn test_camel_case() {
        assert_eq!(camel_case("hello_world"), "helloWorld");
        assert_eq!(camel_case("user-name"), "userName");
    }

    #[test]
    fn test_snake_case() {
        assert_eq!(snake_case("helloWorld"), "hello_world");
        assert_eq!(snake_case("UserName"), "user_name");
    }
}
