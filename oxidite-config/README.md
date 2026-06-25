# oxidite-config

Configuration and environment variable management for Oxidite applications. Loads `oxidite.toml`, `.env` files, and OS environment variables with typed config sections and namespaced table support.

## Installation

```toml
[dependencies]
oxidite-config = "2.3.4"
```

## Feature Flags

This crate has no feature flags — all functionality is always available.

## Usage Examples

### Basic loading

```rust
use oxidite_config::Config;

let config = Config::load()
    .map_err(|e| eprintln!("config error: {e}")).unwrap_or_default();

let host: String = config.get("server.host")
    .unwrap_or_else(|| "127.0.0.1".to_string());
let port: u16 = config.get("server.port").unwrap_or(3000);
let debug: bool = config.get_bool("app.debug").unwrap_or(true);

println!("{host}:{port} (debug={debug})");
```

### Custom file path

```rust
use oxidite_config::Config;

let config = Config::load_from("/etc/myapp/production.toml")
    .map_err(|e| eprintln!("failed to load config: {e}")).unwrap();
```

### Namespaced custom tables

```toml
[google]
client_id     = "abc-123"
client_secret = "secret"

[google.oauth]
redirect_uri  = "http://localhost/callback"

[myapp]
port  = 8080
debug = true
```

```rust
use oxidite_config::Config;

let config = Config::load().unwrap();

// Access as nested config
let client_id: String = config.get("google.client_id").unwrap();
let redirect: String = config.get("google.oauth.redirect_uri").unwrap();

// Also injected as env vars: GOOGLE_CLIENT_ID, GOOGLE_OAUTH_REDIRECT_URI, MYAPP_PORT, MYAPP_DEBUG
let from_env = std::env::var("GOOGLE_CLIENT_ID").unwrap();
assert_eq!(from_env, "abc-123");
```

### Environment overrides

| Env var | Config field |
|---------|-------------|
| `APP_NAME` | `app.name` |
| `SERVER_HOST` | `server.host` |
| `SERVER_PORT` | `server.port` |
| `DATABASE_URL` | `database.url` |
| `REDIS_URL` | `cache.redis_url` + `queue.redis_url` |
| `JWT_SECRET` | `security.jwt_secret` |

## API Reference

### Types

| Type | Description |
|------|-------------|
| `Config` | Root config, contains all typed sections + custom namespaced tables |
| `ConfigError` | Error enum: `Io`, `TomlDe`, `YamlDe`, `InvalidEnvValue`, `MissingKey`, `InvalidType` |
| `Environment` | Enum: `Development`, `Testing`, `Production` |
| `AppConfig` | `name`, `version`, `environment`, `debug` |
| `ServerConfig` | `host`, `port`, `workers` |
| `DatabaseConfig` | `url`, `pool_size`, `ssl` |
| `CacheConfig` | `driver`, `redis_url`, `default_ttl` |
| `QueueConfig` | `driver`, `redis_url`, `workers` |
| `SecurityConfig` | `jwt_secret`, `jwt_expiry`, `cors_origins`, `cors_methods`, `cors_headers`, `rate_limit` |

### `Config` methods

| Method | Description |
|--------|-------------|
| `Config::load()` | Load from `oxidite.toml`, fallback to default |
| `Config::load_from(path)` | Load from a custom path |
| `config.get::<T>(key)` | Get typed value by dotted key path |
| `config.get_required::<T>(key)` | Get required value or `ConfigError` |
| `config.get_u16(key)` | Convenience for `get_required::<u16>` |
| `config.get_bool(key)` | Convenience for `get_required::<bool>` |
| `config.has_key(key)` | Check if a dotted key exists |

### `Environment` methods

| Method | Description |
|--------|-------------|
| `Environment::from_str(s)` | Parse `"production"`, `"prod"`, `"test"`, etc. |
| `env.as_str()` | Return `"development"`, `"testing"`, `"production"` |

### Config file resolution order

| Priority | Source |
|----------|--------|
| 1 (highest) | OS environment variables |
| 2 | `.env` file (dotenv) |
| 3 | `[env]` flat table in `oxidite.toml` |
| 4 | Namespaced tables (`[google]`, `[platform]`, etc.) |
