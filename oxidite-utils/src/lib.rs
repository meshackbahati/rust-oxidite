//! # Oxidite Utils
//!
//! Common utilities for the Oxidite framework including string helpers,
//! date/time utilities, ID generation, and validation helpers.

pub mod date;
pub mod id;
pub mod string;
pub mod validation;

pub use date::{now, format_date, parse_date, Duration};
pub use id::{generate_id, generate_uuid, generate_short_id};
pub use string::{slugify, truncate, capitalize, random_string};
pub use validation::{is_email, is_url, is_phone};
