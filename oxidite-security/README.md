# oxidite-security

Security utilities and primitives for the Oxidite web framework. Provides cryptographic functions, secure random generation, input sanitization, and security best practices.

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/oxidite-security.svg)](https://crates.io/crates/oxidite-security)
[![Docs.rs](https://docs.rs/oxidite-security/badge.svg)](https://docs.rs/oxidite-security)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

</div>

## Overview

`oxidite-security` provides a comprehensive set of security utilities and primitives for the Oxidite web framework. It includes cryptographic functions, secure random generation, input sanitization, and other security-related utilities that follow best practices and industry standards.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
oxidite-security = "0.1"
```

## Features

- **Cryptographic functions** - Secure hashing, encryption, and digital signatures
- **Password hashing** - Industry-standard password hashing with Argon2, bcrypt, or scrypt
- **Secure random generation** - Cryptographically secure random number and token generation
- **Input sanitization** - Protection against XSS, SQL injection, and other injection attacks
- **Token generation** - Secure JWT and custom token creation
- **Hashing utilities** - Various hashing algorithms for different use cases
- **Data encryption** - Symmetric and asymmetric encryption utilities
- **Security headers** - Helper functions for setting security headers
- **Rate limiting** - Protection against brute-force and DoS attacks
- **CSP utilities** - Content Security Policy helper functions

## Usage

### Password Hashing

Securely hash and verify passwords:

```rust
use oxidite_security::hash::{Hasher, Algorithm};

// Create a hasher with Argon2 algorithm (recommended)
let hasher = Hasher::new(Algorithm::Argon2);

// Hash a password
let password_hash = hasher.hash("user_password")?;

// Verify a password against its hash
let is_valid = hasher.verify(&password_hash, "user_password")?;
if is_valid {
    println!("Password is valid!");
} else {
    println!("Invalid password!");
}

// You can also use bcrypt or scrypt
let bcrypt_hasher = Hasher::new(Algorithm::Bcrypt);
let bcrypt_hash = bcrypt_hasher.hash("another_password")?;
```

### Secure Random Generation

Generate cryptographically secure random values:

```rust
use oxidite_security::random::*;

// Generate a secure random token
let token = generate_secure_token(32); // 32 bytes = 256 bits
println!("Secure token: {}", token);

// Generate secure random bytes
let random_bytes = generate_random_bytes(16);
println!("Random bytes: {:?}", random_bytes);

// Generate a secure random number in range
let random_num = generate_random_u32_in_range(1, 100);
println!("Random number between 1-100: {}", random_num);

// Generate a secure random hex string
let hex_string = generate_random_hex(32); // 32 bytes in hex
println!("Random hex: {}", hex_string);
```

### Cryptographic Hashing

Compute secure hashes:

```rust
use oxidite_security::crypto::*;

// SHA-256 hashing
let sha256_hash = hash_sha256("data to hash");
println!("SHA-256: {}", sha256_hash);

// Blake3 hashing (faster alternative)
let blake3_hash = hash_blake3("data to hash");
println!("Blake3: {}", blake3_hash);

// HMAC with secret key
let hmac = compute_hmac_sha256("secret_key", "message");
println!("HMAC: {}", hmac);

// PBKDF2 for password stretching
let pbkdf2_hash = pbkdf2_hash("password", "salt", 100000);
println!("PBKDF2 hash: {}", pbkdf2_hash);
```

### Input Sanitization

Protect against XSS and injection attacks:

```rust
use oxidite_security::sanitize::*;

// HTML escape user input
let user_input = "<script>alert('xss')</script>";
let safe_html = html_escape(user_input);
println!("Safe HTML: {}", safe_html); // &lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;

// Sanitize user input for database queries
let user_search = "'; DROP TABLE users; --";
let sanitized = sql_sanitize(user_search);
println!("Sanitized: {}", sanitized);

// URL encode user input
let unsafe_url = "https://example.com/search?q=user input & more";
let encoded = url_encode(unsafe_url);
println!("Encoded: {}", encoded);

// Sanitize file names
let unsafe_filename = "../../../etc/passwd";
let safe_filename = sanitize_filename(unsafe_filename);
println!("Safe filename: {}", safe_filename);
```

### JWT Token Creation and Verification

Create and verify JSON Web Tokens:

```rust
use oxidite_security::crypto::{JwtSigner, JwtVerifier, Algorithm};

// Create a signer with HS256 algorithm
let signer = JwtSigner::new("your-secret-key".to_string(), Algorithm::HS256);

// Create claims
let claims = serde_json::json!({
    "sub": "user-id",
    "exp": chrono::Utc::now().timestamp() + 3600, // 1 hour from now
    "iat": chrono::Utc::now().timestamp(),
});

// Sign the token
let token = signer.sign(&claims)?;
println!("JWT: {}", token);

// Verify the token
let verifier = JwtVerifier::new("your-secret-key".to_string(), Algorithm::HS256);
let verified_claims = verifier.verify(&token)?;

println!("Verified claims: {:?}", verified_claims);
```

### Content Security Policy

Generate Content Security Policy headers:

```rust
use oxidite_security::sanitize::csp::*;

let mut csp = CspBuilder::new();

// Restrict sources
csp.default_src(CspSource::None)
   .script_src(vec![CspSource::SelfSrc])
   .style_src(vec![CspSource::SelfSrc, CspSource::UnsafeInline])
   .img_src(vec![CspSource::SelfSrc, CspSource::Https])
   .font_src(vec![CspSource::SelfSrc]);

let csp_header = csp.build();
println!("CSP Header: {}", csp_header);
// Output: "default-src 'none'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' https:; font-src 'self'"
```

### Rate Limiting Utilities

Implement rate limiting to prevent abuse:

```rust
use oxidite_security::rate_limit::*;

// Create a rate limiter (100 requests per minute per IP)
let mut limiter = RateLimiter::new(100, std::time::Duration::from_secs(60));

// Check if a request should be allowed
let client_ip = "192.168.1.1";
if limiter.is_allowed(client_ip) {
    println!("Request allowed");
    limiter.record_request(client_ip);
} else {
    println!("Rate limit exceeded");
}
```

### Encryption Utilities

Encrypt and decrypt data:

```rust
use oxidite_security::crypto::*;

// Generate a key for symmetric encryption
let key = generate_aes_key();
let iv = generate_iv();

// Encrypt data
let plaintext = "Sensitive data";
let ciphertext = encrypt_aes_gcm(plaintext.as_bytes(), &key, &iv)?;

// Decrypt data
let decrypted = decrypt_aes_gcm(&ciphertext, &key, &iv)?;
let decrypted_str = String::from_utf8(decrypted)?;

println!("Decrypted: {}", decrypted_str);
```

### Security Headers

Helper functions for setting security headers:

```rust
use oxidite_security::sanitize::headers::*;

// Create security headers
let security_headers = SecurityHeaders::new()
    .strict_transport_security("max-age=31536000; includeSubDomains; preload")
    .x_frame_options("SAMEORIGIN")
    .x_content_type_options("nosniff")
    .x_xss_protection("1; mode=block")
    .referrer_policy("strict-origin-when-cross-origin");

// Apply to HTTP response (integration with your framework)
// response.headers_mut().extend(security_headers.into_iter());
```

## Integration with Oxidite

The security utilities integrate seamlessly with Oxidite applications:

```rust
use oxidite::prelude::*;
use oxidite_security::hash::Hasher;

async fn register_user(
    Json(payload): Json<RegisterRequest>
) -> Result<OxiditeResponse> {
    // Hash the password securely
    let hasher = Hasher::new(oxidite_security::hash::Algorithm::Argon2);
    let password_hash = hasher.hash(&payload.password)?;
    
    // Sanitize user input
    let username = oxidite_security::sanitize::sanitize_username(&payload.username)?;
    
    // Create the user with hashed password
    // ... save to database ...
    
    Ok(response::json(serde_json::json!({
        "message": "User registered successfully"
    })))
}

async fn login(
    Json(payload): Json<LoginRequest>
) -> Result<OxiditeResponse> {
    // Retrieve user from database (assuming we have the stored hash)
    // let stored_hash = get_password_hash_from_db(&payload.username).await?;
    
    // Verify password
    let hasher = Hasher::new(oxidite_security::hash::Algorithm::Argon2);
    // let is_valid = hasher.verify(&stored_hash, &payload.password)?;
    
    if is_valid {
        // Generate secure JWT token
        let token = oxidite_security::crypto::JwtSigner::new(
            std::env::var("JWT_SECRET").unwrap_or("fallback_secret".to_string()),
            oxidite_security::crypto::Algorithm::HS256
        ).sign(&serde_json::json!({
            "sub": "user-id",
            "exp": chrono::Utc::now().timestamp() + 3600
        }))?;
        
        Ok(response::json(serde_json::json!({
            "token": token,
            "message": "Login successful"
        })))
    } else {
        Err(OxiditeError::Unauthorized("Invalid credentials".to_string()))
    }
}
```

## Security Best Practices

The library implements security best practices:

- **Defense in depth**: Multiple layers of protection
- **Principle of least privilege**: Minimal permissions by default
- **Secure defaults**: Safe configurations out of the box
- **Constant-time operations**: Protection against timing attacks
- **Proper entropy**: High-quality randomness for security-sensitive operations
- **Memory safety**: No memory leaks or buffer overflows

## Performance Considerations

Security operations are optimized for performance:

- Asynchronous operations where appropriate
- Efficient algorithms that balance security and performance
- Proper resource management
- Minimal overhead for security checks

## License

MIT