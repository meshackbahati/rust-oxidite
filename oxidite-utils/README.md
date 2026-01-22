# oxidite-utils

Utility functions and helpers for the Oxidite web framework. Provides common utilities for date handling, string manipulation, ID generation, and validation.

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/oxidite-utils.svg)](https://crates.io/crates/oxidite-utils)
[![Docs.rs](https://docs.rs/oxidite-utils/badge.svg)](https://docs.rs/oxidite-utils)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

</div>

## Overview

`oxidite-utils` is a collection of general-purpose utility functions and helpers that are commonly needed in web applications. It provides utilities for date/time handling, string manipulation, unique ID generation, validation, and other common tasks.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
oxidite-utils = "0.1"
```

## Features

- **Date/Time utilities** - Easy date/time manipulation and formatting
- **String utilities** - Common string operations and transformations
- **ID generation** - Unique identifier generation (UUID, snowflake IDs, etc.)
- **Validation helpers** - Common validation functions for web applications
- **Random utilities** - Secure random generation for tokens and secrets
- **Formatting helpers** - Number, currency, and text formatting
- **File utilities** - Common file operations and path manipulations
- **URL utilities** - URL parsing, encoding, and manipulation
- **Collection helpers** - Common operations on vectors, hashes, and other collections

## Usage

### Date/Time Utilities

Work with dates and times in a convenient way:

```rust
use oxidite_utils::date::*;

// Get current timestamp
let now = current_timestamp();
println!("Current timestamp: {}", now);

// Format a timestamp
let formatted = format_timestamp(now, "%Y-%m-%d %H:%M:%S");
println!("Formatted time: {}", formatted);

// Parse a date string
let parsed = parse_date("2023-12-25", "%Y-%m-%d")?;
println!("Parsed date: {:?}", parsed);

// Calculate time differences
let duration = time_since(chrono::Utc::now() - chrono::Duration::hours(2));
println!("Hours since: {}", duration.num_hours());
```

### String Utilities

Common string operations:

```rust
use oxidite_utils::string::*;

// Generate slugs
let slug = slugify("Hello World! This is a test.");
println!("Slug: {}", slug); // Output: "hello-world-this-is-a-test"

// Generate random strings
let random_str = random_string(10);
println!("Random string: {}", random_str);

// Case conversion
let snake_case = to_snake_case("camelCaseString");
println!("Snake case: {}", snake_case); // Output: "camel_case_string"

let kebab_case = to_kebab_case("snake_case_string");
println!("Kebab case: {}", kebab_case); // Output: "snake-case-string"

// Safe string truncation
let long_text = "This is a very long text that needs to be truncated";
let truncated = truncate_safe(long_text, 20, "...");
println!("Truncated: {}", truncated);
```

### ID Generation

Generate unique identifiers:

```rust
use oxidite_utils::id::*;

// Generate UUIDs
let uuid = generate_uuid();
println!("Generated UUID: {}", uuid);

// Generate ULIDs (Universally Unique Lexicographically Sortable Identifiers)
let ulid = generate_ulid();
println!("Generated ULID: {}", ulid);

// Generate short random IDs
let short_id = generate_short_id(8);
println!("Short ID: {}", short_id);

// Generate incrementing IDs (useful for databases)
let incrementing_id = generate_incrementing_id();
println!("Incrementing ID: {}", incrementing_id);
```

### Validation Helpers

Validate common input formats:

```rust
use oxidite_utils::validation::*;

// Validate email addresses
let is_email_valid = is_valid_email("user@example.com");
println!("Email valid: {}", is_email_valid);

// Validate URLs
let is_url_valid = is_valid_url("https://example.com/path");
println!("URL valid: {}", is_url_valid);

// Validate phone numbers
let is_phone_valid = is_valid_phone("+1-555-123-4567");
println!("Phone valid: {}", is_phone_valid);

// Validate passwords
let is_password_strong = is_strong_password("SecurePass123!", &[
    ValidationRule::MinLength(8),
    ValidationRule::HasUppercase,
    ValidationRule::HasLowercase,
    ValidationRule::HasNumber,
    ValidationRule::HasSpecialChar,
]);
println!("Password strong: {}", is_password_strong);
```

### Random Utilities

Secure random generation:

```rust
use oxidite_utils::random::*;

// Generate secure random bytes
let random_bytes = generate_random_bytes(32);
println!("Random bytes length: {}", random_bytes.len());

// Generate secure random tokens
let token = generate_secure_token(32);
println!("Secure token: {}", token);

// Generate random numbers in range
let random_num = random_range(1, 100);
println!("Random number: {}", random_num);

// Generate random boolean
let coin_flip = random_bool();
println!("Coin flip: {}", coin_flip);
```

### Collection Helpers

Utilities for working with collections:

```rust
use oxidite_utils::collection::*;

// Shuffle a vector
let mut numbers = vec![1, 2, 3, 4, 5];
shuffle_vec(&mut numbers);
println!("Shuffled: {:?}", numbers);

// Get random elements
let random_elements = random_choice(&[1, 2, 3, 4, 5], 3);
println!("Random choices: {:?}", random_elements);

// Chunk a vector
let chunks = chunk_vec(&[1, 2, 3, 4, 5, 6, 7], 3);
println!("Chunks: {:?}", chunks);

// Filter duplicates
let unique = unique_items(&[1, 2, 2, 3, 3, 4]);
println!("Unique items: {:?}", unique);
```

### Formatting Helpers

Format values for display:

```rust
use oxidite_utils::string::*;

// Format numbers
let formatted_num = format_number(1234567.89);
println!("Formatted number: {}", formatted_num);

// Format currency
let formatted_currency = format_currency(1234.56, "USD");
println!("Currency: {}", formatted_currency);

// Format file sizes
let file_size = format_file_size(1024 * 1024 * 5); // 5 MB
println!("File size: {}", file_size);
```

### URL Utilities

Work with URLs:

```rust
use oxidite_utils::string::*;

// Build query strings
let params = vec![
    ("name", "John"),
    ("age", "30"),
    ("city", "New York")
];
let query_string = build_query_string(&params);
println!("Query string: {}", query_string);

// Sanitize URLs
let sanitized = sanitize_url("https://example.com/path?param=value<script>");
println!("Sanitized URL: {}", sanitized);
```

### Custom Validation

Create custom validation rules:

```rust
use oxidite_utils::validation::*;

// Define custom validation rules
fn validate_username(username: &str) -> bool {
    username.len() >= 3 && 
    username.len() <= 20 && 
    username.chars().all(|c| c.is_alphanumeric() || c == '_')
}

let is_valid = validate_username("john_doe123");
println!("Username valid: {}", is_valid);
```

## Integration with Oxidite

The utilities are designed to work seamlessly with Oxidite applications:

```rust
use oxidite::prelude::*;
use oxidite_utils::id::generate_uuid;

async fn create_user(
    Json(payload): Json<CreateUserRequest>
) -> Result<OxiditeResponse> {
    // Generate a unique ID for the user
    let user_id = generate_uuid();
    
    // Validate the input
    if !oxidite_utils::validation::is_valid_email(&payload.email) {
        return Err(OxiditeError::BadRequest("Invalid email address".to_string()));
    }
    
    // Create the user with the generated ID
    // ... implementation ...
    
    Ok(response::json(serde_json::json!({
        "id": user_id,
        "message": "User created successfully"
    })))
}
```

## Performance

The utilities are optimized for performance:

- Minimal allocations where possible
- Efficient algorithms for common operations
- Zero-cost abstractions where applicable
- Proper error handling without performance penalties

## Security

Security considerations are built into the utilities:

- Secure random generation for tokens and IDs
- Input sanitization for user-provided data
- Safe string operations that prevent buffer overflows
- Proper validation to prevent injection attacks

## License

MIT