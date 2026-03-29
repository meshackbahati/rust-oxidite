//! Backwards-compatible cookie/form module.
//!
//! Canonical extractor implementations live in `crate::extract`.
//! This module re-exports them to preserve existing import paths.

pub use crate::extract::{Cookies, Form};
