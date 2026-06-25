# oxidite-utils

Utility functions and helpers for the Oxidite web framework. Provides common utilities for date handling, string manipulation, ID generation, validation, and metrics.

## Installation

```toml
[dependencies]
oxidite-utils = "2.3.4"
```

## Feature Flags

This crate has no feature flags — all modules are always available.

| Module | Description |
|--------|-------------|
| `date` | Timestamps, formatting, parsing, expiry checks |
| `id` | UUIDv4, short base64 IDs, alphanumeric/numeric IDs |
| `string` | Slugify, truncate, capitalize, random strings, case conversion |
| `validation` | Email, URL, phone, alphanumeric, length validators |
| `metrics` | Atomic per-route request counters and duration tracking |

## Usage Examples

```rust
use oxidite_utils::{
    generate_uuid,
    slugify,
    unix_timestamp,
    is_email,
    MetricsRegistry,
};

// Generate a unique ID
let user_id = generate_uuid();
println!("User ID: {}", user_id);

// Create a URL-friendly slug
let slug = slugify("Hello World!");
assert_eq!(slug, "hello-world");

// Unix timestamp for expiry checks
let now = unix_timestamp();
let is_expired = now >= 1735689600;
println!("Expired: {}", is_expired);

// Validate an email
if !is_email("user@example.com") {
    eprintln!("Invalid email");
}

// Track metrics
let metrics = MetricsRegistry::new();
metrics.record_request("/api/users", 42, true);
println!("Requests: {:?}", metrics.get_snapshot());
```

## API Reference

### `date` module

| Function | Description |
|----------|-------------|
| `now()` | Current UTC `DateTime<Utc>` |
| `format_date(dt, fmt)` | Format datetime with strftime pattern |
| `parse_date(s, fmt)` | Parse datetime string -> `Option<DateTime<Utc>>` |
| `unix_timestamp()` | Current Unix time in seconds |
| `unix_timestamp_millis()` | Current Unix time in milliseconds |
| `is_expired(ts)` | `true` if `unix_timestamp() >= ts` |

### `id` module

| Function | Description |
|----------|-------------|
| `generate_uuid()` | UUID v4 string (36 chars) |
| `generate_id()` | URL-safe base64 UUID (22 chars) |
| `generate_short_id(len)` | Random alphanumeric of given length |
| `generate_numeric_id(len)` | Random numeric digits of given length |

### `string` module

| Function | Description |
|----------|-------------|
| `slugify(s)` | URL-friendly slug (lowercase, `-` delimited) |
| `truncate(s, max_len)` | Truncate with `...` ellipsis |
| `capitalize(s)` | Uppercase first character |
| `random_string(len)` | Random alphanumeric string |
| `camel_case(s)` | Convert to `camelCase` |
| `snake_case(s)` | Convert to `snake_case` |

### `validation` module

| Function | Description |
|----------|-------------|
| `is_email(s)` | Validate email format |
| `is_url(s)` | Validate HTTP/HTTPS URL |
| `is_phone(s)` | Validate international phone number |
| `is_alphanumeric(s)` | Check all chars are alphanumeric |
| `is_numeric(s)` | Check all chars are ASCII digits |
| `min_length(s, n)` | `s.len() >= n` |
| `max_length(s, n)` | `s.len() <= n` |
| `length_between(s, min, max)` | Inclusive byte-length range |

### `metrics` module

| Type / Function | Description |
|-----------------|-------------|
| `RouteMetrics` | Per-route atomic counters (`request_count`, `success_count`, `error_count`, `total_duration_ms`) |
| `MetricsRegistry` | Registry with `record_request()`, `get_snapshot()`, `concurrent_requests()` |
| `GLOBAL_METRICS` | `Lazy<MetricsRegistry>` singleton for app-wide metrics |
