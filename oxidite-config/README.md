# oxidite-config

Configuration and environment variable management for Oxidite applications.

## Installation

```toml
[dependencies]
oxidite-config = "2.3.3"
```

## Usage

```rust
use oxidite_config::Config;

let config = Config::load()?;

let host: String = config.get("server.host").unwrap();
let port: u16 = config.get_u16("server.port")?;
let debug: bool = config.get_bool("app.debug")?;

println!("{}:{} (debug={})", host, port, debug);
# Ok::<(), oxidite_config::ConfigError>(())
```

## Environment Variables

Oxidite supports three strategies for defining environment variables in `oxidite.toml`. All strategies produce the same `std::env::var()` results at runtime.

### 1. Flat `[env]` table

Keys map directly to environment variables:

```toml
[env]
DATABASE_URL = "postgres://..."
JWT_SECRET   = "change-me"
NAME         = "my-app"
```

### 2. Namespaced tables

Any root-level TOML table that is not a known config section (`app`, `server`, `database`, `cache`, `queue`, `security`) becomes an env namespace. The table name is uppercased and prepended to each key:

```toml
[google]
client_id     = "abc"       # → GOOGLE_CLIENT_ID
client_secret = "xyz"       # → GOOGLE_CLIENT_SECRET

[platform]
name = "my-app"             # → PLATFORM_NAME
```

### 3. Nested tables

Tables can nest to any depth. Each level adds another `_SEGMENT`:

```toml
[google.oauth]
client_id = "abc"           # → GOOGLE_OAUTH_CLIENT_ID
```

Non-string TOML values (integers, booleans) are converted automatically:

```toml
[myapp]
port  = 8080                # → MYAPP_PORT = "8080"
debug = true                # → MYAPP_DEBUG = "true"
```

### Resolution order

| Priority | Source |
|----------|--------|
| 1 (highest) | OS environment variables |
| 2 | `.env` file (dotenv) |
| 3 | `[env]` flat table |
| 4 | Namespaced tables |

### Reading config directly

Namespaced tables are also accessible without going through `std::env`:

```rust
let config = Config::load()?;

let client_id: String = config.get("google.client_id").unwrap();
let redirect: String  = config.get("google.oauth.redirect_uri").unwrap();

if config.has_key("stripe.api_key") {
    // ...
}
```

## Environment Overrides

The loader also applies these environment variable overrides to known config fields:

- `OXIDITE_ENV` / `ENVIRONMENT` → `app.environment`
- `APP_NAME` → `app.name`
- `SERVER_HOST` → `server.host`
- `SERVER_PORT` → `server.port`
- `DATABASE_URL` → `database.url`
- `REDIS_URL` → `cache.redis_url` and `queue.redis_url`
- `JWT_SECRET` → `security.jwt_secret`

Invalid values (for example a non-numeric `SERVER_PORT`) return a typed `ConfigError`.
