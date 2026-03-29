# Security and Identity Crates

## `oxidite-auth`

Main modules/exports:

- password hashing: `PasswordHasher`, `hash_password`, `verify_password`
- JWT: `JwtManager`, `create_token`, `verify_token`, `Claims`
- middleware: `AuthMiddleware`
- RBAC: `Role`, `Permission`
- sessions: `Session`, `SessionStore`, `InMemorySessionStore`, `RedisSessionStore`, `SessionManager`
- session middleware: `SessionMiddleware`, `SessionLayer`
- OAuth helpers: `OAuth2Client`, `OAuth2Config`, `ProviderConfig`, `OAuth2Provider`
- authorization guards/services: `RequireRole`, `RequirePermission`, `AuthorizationService`
- API keys: `ApiKey`, `ApiKeyMiddleware`
- security flows: email verification, password reset, two-factor helpers

Error model:

- `AuthError`

## `oxidite-security`

Main APIs:

- symmetric crypto: `encrypt`, `decrypt`, `AesKey`
- hashing/HMAC: `sha256`, `sha512`, `hmac_sha256`, `verify_hmac_sha256`
- secure randomness: `random_bytes`, `random_hex`, `secure_token`, `random_alphanumeric`, `random_range`, `try_random_range`
- sanitization: `sanitize_html`, `escape_html`, `strip_tags`

Use for cryptographic primitives and input sanitization utilities.
