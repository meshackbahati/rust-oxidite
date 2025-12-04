# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.x.x   | :white_check_mark: |

As Oxidite is currently in alpha/beta, we support only the latest version.

---

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability, please report it responsibly:

### DO NOT

- Open a public GitHub issue
- Disclose the vulnerability publicly before we've had a chance to address it

### DO

1. **Email**: Send details to security@oxidite.dev _(or create a private security advisory on GitHub)_
2. **Include**:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if you have one)

### Response Timeline

- **Initial Response**: Within 48 hours
- **Status Update**: Within 7 days
- **Fix Timeline**: Depends on severity
  - **Critical**: 1-3 days
  - **High**: 1-2 weeks
  - **Medium**: 2-4 weeks
  - **Low**: Next release cycle

---

## Security Features

Oxidite is designed with security as a top priority:

### Memory Safety
- **No buffer overflows**: Rust's ownership system prevents them
- **No use-after-free**: Borrow checker ensures safety
- **No data races**: Compile-time race detection

### Cryptography
- **Argon2id**: Password hashing with recommended parameters
- **Constant-time comparisons**: Prevents timing attacks
- **Secure randomness**: Uses OS-provided CSPRNG

### Web Security

#### OWASP Top 10 Mitigations

1. **Injection** ✅
   - Prepared statements for all SQL queries
   - Type-safe parameter binding
   - Input validation via serde

2. **Broken Authentication** ✅
   - Secure session management
   - Password strength requirements
   - Account lockout after failed attempts
   - JWT with short expiration

3. **Sensitive Data Exposure** ✅
   - TLS 1.3 enforced
   - Encrypted secrets storage
   - No sensitive data in logs

4. **XML External Entities (XXE)** ✅
   - No XML parsing by default
   - JSON-only APIs

5. **Broken Access Control** ✅
   - RBAC/PBAC built-in
   - Authorization middleware
   - Principle of least privilege

6. **Security Misconfiguration** ✅
   - Secure defaults
   - Security headers enabled by default
   - Configuration validation

7. **Cross-Site Scripting (XSS)** ✅
   - Auto-escaping in templates
   - Content-Security-Policy headers
   - JSON APIs (not HTML)

8. **Insecure Deserialization** ✅
   - Type-safe deserialization with serde
   - Size limits on request bodies
   - Validation on all inputs

9. **Using Components with Known Vulnerabilities** ✅
   - Regular dependency audits
   - `cargo audit` in CI
   - Automatic dependabot updates

10. **Insufficient Logging & Monitoring** ✅
    - Structured logging
    - Request ID tracking
    - Audit logs for sensitive operations

---

## Secure Configuration

### Development
```toml
# oxidite.toml
[security]
tls_enabled = false  # OK for dev
cors_allow_all = true
debug_mode = true
```

### Production
```toml
[security]
tls_enabled = true
tls_min_version = "1.3"
cors_allow_all = false
cors_allowed_origins = ["https://example.com"]
debug_mode = false
hsts_enabled = true
csp_enabled = true
```

---

## Security Headers

Oxidite automatically adds:

```
Strict-Transport-Security: max-age=31536000; includeSubDomains
X-Frame-Options: DENY
X-Content-Type-Options: nosniff
X-XSS-Protection: 1; mode=block
Content-Security-Policy: default-src 'self'
Referrer-Policy: strict-origin-when-cross-origin
Permissions-Policy: geolocation=(), microphone=(), camera=()
```

---

## Rate Limiting

Protect against brute force and DoS:

```rust
use oxidite_middleware::RateLimitLayer;

let service = ServiceBuilder::new()
    .layer(RateLimitLayer::new(
        100,  // requests
        Duration::from_secs(60),  // per minute
    ))
    .service(router);
```

---

## Authentication Best Practices

### Password Requirements
- Minimum 12 characters
- Mix of uppercase, lowercase, numbers, symbols
- Check against common password lists
- Argon2id with memory=19456, iterations=2, parallelism=1

### JWT Tokens
- Short expiration (15 minutes for access tokens)
- Refresh tokens (7 days)
- Signed with HS256 or RS256
- Include `exp`, `nbf`, `iat` claims

### Sessions
- HTTP-only cookies
- Secure flag in production
- SameSite=Strict or Lax
- Random session IDs (256 bits)

---

## Database Security

- **Prepared statements only**: No string concatenation
- **Connection encryption**: TLS for PostgreSQL/MySQL
- **Least privilege**: Application user has minimal permissions
- **Backup encryption**: Encrypt database backups

---

## Secrets Management

Never commit secrets to version control:

```bash
# .env (not committed)
DATABASE_URL=postgres://user:password@localhost/db
JWT_SECRET=your-secret-key
```

In production, use:
- AWS Secrets Manager
- HashiCorp Vault
- Environment variables from secure sources

---

## Security Checklist

Before deploying to production:

- [ ] TLS 1.3 enabled
- [ ] Secrets in environment variables (not code)
- [ ] CORS configured (not `allow_all`)
- [ ] Rate limiting enabled
- [ ] Security headers configured
- [ ] Database connections encrypted
- [ ] Logs don't contain sensitive data
- [ ] Dependencies audited (`cargo audit`)
- [ ] Input validation on all endpoints
- [ ] Error messages don't leak information
- [ ] Health check doesn't expose internals

---

## Security Audits

We plan to conduct:
- **Internal audits**: Quarterly
- **External audits**: Before v1.0 release
- **Penetration testing**: Before production use

---

## Disclosure Policy

When we fix a security vulnerability:

1. **Patch released**: Fix deployed to main branch
2. **Release notes**: Security fix mentioned (not details)
3. **Advisory published**: 7 days after patch
4. **CVE requested**: For critical issues
5. **Credit given**: To reporter (if desired)

---

## Security Resources

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Rust Security Working Group](https://www.rust-lang.org/governance/wgs/wg-security-response)
- [CWE Top 25](https://cwe.mitre.org/top25/)

---

Thank you for helping keep Oxidite and its users safe! 🔒
