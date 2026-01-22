# oxidite-config

Configuration management for the Oxidite web framework.

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/oxidite-config.svg)](https://crates.io/crates/oxidite-config)
[![Docs.rs](https://docs.rs/oxidite-config/badge.svg)](https://docs.rs/oxidite-config)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

</div>

## Overview

`oxidite-config` provides flexible configuration management for Oxidite applications. It supports multiple configuration sources including environment variables, configuration files, and command-line arguments. The crate offers type-safe configuration loading with automatic deserialization from various formats.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
oxidite-config = "2.0"
```

## Features

- **Multiple Configuration Sources**: Load configuration from environment variables, JSON, TOML, YAML files
- **Hierarchical Configuration**: Merge multiple configuration sources with precedence
- **Type-Safe Deserialization**: Automatic conversion to strongly-typed configuration structs
- **Environment Variable Support**: Easy mapping of environment variables to configuration values
- **Hot Reload**: Configuration reloading without application restart
- **Validation**: Built-in configuration validation and error reporting

## Usage

### Basic Configuration Loading

```rust
use oxidite_config::{Config, ConfigSource};

#[derive(serde::Deserialize, Clone)]
struct AppConfig {
    server_port: u16,
    database_url: String,
    debug_mode: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::new();
    
    // Load from environment variables
    config.load_source(ConfigSource::Environment);
    
    // Load from config file
    config.load_source(ConfigSource::File("config/app.json".to_string()));
    
    // Get typed configuration
    let app_config: AppConfig = config.get("app")?;
    
    println!("Server will run on port: {}", app_config.server_port);
    
    Ok(())
}
```

### Configuration with Defaults

```rust
use oxidite_config::{Config, ConfigBuilder};

#[derive(serde::Deserialize, Clone)]
struct ServerConfig {
    host: String,
    port: u16,
    ssl_enabled: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            ssl_enabled: false,
        }
    }
}

async fn load_server_config() -> Result<ServerConfig, Box<dyn std::error::Error>> {
    let config = ConfigBuilder::new()
        .add_default_source()  // Built-in defaults
        .add_env_source()      // Environment variables
        .add_file_source("config/server.toml")  // Config file
        .build()
        .await?;

    Ok(config.get_with_default("server", ServerConfig::default())?)
}
```

### Environment Variable Configuration

```rust
use oxidite_config::Config;

// Define environment variable mappings
std::env::set_var("DATABASE_URL", "postgres://localhost/myapp");
std::env::set_var("PORT", "8080");
std::env::set_var("DEBUG", "true");

let config = Config::from_env()
    .prefix("MYAPP_")  // Use MYAPP_DATABASE_URL, MYAPP_PORT, etc.
    .separator("_")
    .build()?;

let database_url: String = config.get("database_url")?;
let port: u16 = config.get("port")?;
let debug: bool = config.get("debug")?;
```

## Configuration Formats

The crate supports multiple configuration formats:

- **JSON**: Standard JSON configuration files
- **TOML**: Human-readable TOML format
- **YAML**: YAML configuration with complex nesting
- **Environment Variables**: Direct environment variable mapping
- **Inline Configuration**: Programmatic configuration definition

## Advanced Features

### Configuration Validation

```rust
use oxidite_config::{Config, ValidationError};

let config = Config::builder()
    .add_source(ConfigSource::File("config/app.json".to_string()))
    .validator(|config| {
        let port: u16 = config.get("server.port")?;
        if port == 0 || port > 65535 {
            return Err(ValidationError::new("server.port must be between 1 and 65535"));
        }
        Ok(())
    })
    .build()?;
```

### Dynamic Configuration Reloading

```rust
use oxidite_config::Config;

// Enable hot reloading
let config = Config::builder()
    .enable_hot_reload(true)
    .add_source(ConfigSource::File("config/app.toml".to_string()))
    .build()?;

// Configuration will automatically reload when the file changes
let updated_value = config.get::<String>("some_setting")?;
```

## License

MIT