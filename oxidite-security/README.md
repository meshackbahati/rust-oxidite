# oxidite-security

Security-focused utilities for Oxidite.

## Installation

```toml
[dependencies]
oxidite-security = "2.1.0"
```

## Available Utilities

- `crypto`: AES-256-GCM encrypt/decrypt helpers
- `hash`: SHA-256, SHA-512, HMAC-SHA256 (+ verification)
- `random`: secure random bytes/tokens/hex/alphanumeric
- `sanitize`: HTML escaping, sanitization, and tag stripping

## Quick Examples

```rust
use oxidite_security::{encrypt, decrypt, sha256, secure_token, sanitize_html};

let key = [0u8; 32];
let encrypted = encrypt(&key, b"secret")?;
let decrypted = decrypt(&key, &encrypted)?;
assert_eq!(decrypted, b"secret");

let digest = sha256(b"hello");
assert_eq!(digest.len(), 64);

let token = secure_token(32);
assert!(!token.is_empty());

let sanitized = sanitize_html("<p>ok</p><script>alert(1)</script>");
assert!(!sanitized.contains("script"));
# Ok::<(), oxidite_security::SecurityError>(())
```

## Notes

- `try_random_range(min, max)` provides validated range generation.
- `random_range(min, max)` is a convenience wrapper that falls back to `min` for invalid ranges.
