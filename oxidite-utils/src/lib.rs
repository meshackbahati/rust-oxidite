//! # Oxidite Utils
//!
//! Common utilities for the Oxidite framework including string helpers,
//! date/time utilities, ID generation, and validation helpers.

pub mod date;
pub mod id;
pub mod string;
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
