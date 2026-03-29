# oxidite-config

Configuration management for Oxidite applications.

## Installation

```toml
[dependencies]
oxidite-config = "2.1.0"
```

## Usage

```rust
use oxidite_config::Config;

let config = Config::load()?;

let host = config.get_string("server.host")?;
let port = config.get_u16("server.port")?;
let debug = config.get_bool("app.debug")?;

println!("{}:{} (debug={})", host, port, debug);
# Ok::<(), oxidite_config::ConfigError>(())
```

## Environment Overrides

The loader supports `.env` and selected environment variable overrides such as:

- `OXIDITE_ENV` / `ENVIRONMENT`
- `APP_NAME`
- `SERVER_HOST`
- `SERVER_PORT`
- `DATABASE_URL`
- `REDIS_URL`
- `JWT_SECRET`

Invalid values (for example a non-numeric `SERVER_PORT`) return a typed `ConfigError`.
