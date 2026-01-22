# oxidite-plugin

Plugin system for the Oxidite web framework.

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/oxidite-plugin.svg)](https://crates.io/crates/oxidite-plugin)
[![Docs.rs](https://docs.rs/oxidite-plugin/badge.svg)](https://docs.rs/oxidite-plugin)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

</div>

## Overview

`oxidite-plugin` provides a flexible plugin system that enables extending Oxidite applications with additional functionality. The plugin architecture supports dynamic loading, lifecycle management, and hook-based extension points. Plugins can add routes, middleware, database migrations, and more to your Oxidite application.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
oxidite-plugin = "2.0"
oxidite = "2.0"
```

## Features

- **Dynamic Plugin Loading**: Load plugins at runtime without recompilation
- **Hook System**: Extensive hook points for intercepting application lifecycle events
- **Dependency Management**: Plugin dependency resolution and version compatibility
- **Secure Sandboxing**: Isolated plugin execution environment
- **Hot Reloading**: Plugin updates without application restart
- **Configuration Integration**: Plugin-specific configuration management
- **Event Broadcasting**: Inter-plugin communication via events
- **Asset Serving**: Plugin-specific static assets and resources

## Usage

### Creating a Basic Plugin

```rust
use oxidite_plugin::{Plugin, PluginInfo, HookRegistry, PluginResult};
use oxidite::prelude::*;

pub struct HelloWorldPlugin;

impl Plugin for HelloWorldPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "hello_world".to_string(),
            version: "1.0.0".to_string(),
            author: "Developer".to_string(),
            description: "A simple hello world plugin".to_string(),
        }
    }

    fn register_hooks(&self, hooks: &mut HookRegistry) -> PluginResult<()> {
        // Register plugin hooks
        hooks.register("before_startup", |ctx| {
            println!("HelloWorldPlugin: Running before startup");
            Ok(())
        })?;
        
        hooks.register("after_startup", |ctx| {
            println!("HelloWorldPlugin: Running after startup");
            Ok(())
        })?;
        
        Ok(())
    }

    fn init(&self, app: &mut Router) -> PluginResult<()> {
        // Add routes to the application
        app.get("/hello", |_: Request| async {
            Ok(response::text("Hello from plugin!"))
        });
        
        Ok(())
    }
}

// Register the plugin
#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    
    // Load and initialize the plugin
    let plugin = HelloWorldPlugin;
    plugin.init(&mut router)?;
    
    Server::new(router)
        .listen("127.0.0.1:3000".parse().unwrap())
        .await
}
```

### Plugin with Configuration

```rust
use oxidite_plugin::{Plugin, PluginInfo, PluginResult};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
struct HelloConfig {
    greeting: String,
    target: String,
}

pub struct ConfigurableHelloPlugin {
    config: HelloConfig,
}

impl ConfigurableHelloPlugin {
    pub fn new(config: HelloConfig) -> Self {
        Self { config }
    }
}

impl Plugin for ConfigurableHelloPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "configurable_hello".to_string(),
            version: "1.0.0".to_string(),
            author: "Developer".to_string(),
            description: "A configurable hello plugin".to_string(),
        }
    }

    fn init(&self, app: &mut Router) -> PluginResult<()> {
        let greeting = self.config.greeting.clone();
        let target = self.config.target.clone();
        
        app.get("/custom-hello", move |_: Request| {
            let greeting = greeting.clone();
            let target = target.clone();
            async move {
                Ok(response::text(format!("{} {}!", greeting, target)))
            }
        });
        
        Ok(())
    }
}
```

### Plugin with Database Operations

```rust
use oxidite_plugin::{Plugin, PluginInfo, PluginResult};
use oxidite_db::{Model, Database};

#[derive(Model)]
#[model(table = "plugin_data")]
struct PluginData {
    #[model(primary_key)]
    id: i32,
    key: String,
    value: String,
}

pub struct DatabasePlugin {
    db: Database,
}

impl DatabasePlugin {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl Plugin for DatabasePlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "database_plugin".to_string(),
            version: "1.0.0".to_string(),
            author: "Developer".to_string(),
            description: "A plugin with database operations".to_string(),
        }
    }

    fn init(&self, app: &mut Router) -> PluginResult<()> {
        // Add routes that use database
        app.get("/plugin-data", {
            let db = self.db.clone();
            move |_: Request| {
                let db = db.clone();
                async move {
                    let data: Vec<PluginData> = db.find_all()?;
                    Ok(response::json(serde_json::json!(data)))
                }
            }
        });
        
        Ok(())
    }
}
```

## Hook System

The plugin system provides various hooks for different lifecycle events:

### Application Lifecycle Hooks

```rust
use oxidite_plugin::{HookRegistry, PluginContext};

fn register_app_lifecycle_hooks(hooks: &mut HookRegistry) -> PluginResult<()> {
    // Called before application starts
    hooks.register("before_startup", |ctx: &PluginContext| {
        println!("Application starting...");
        Ok(())
    })?;
    
    // Called after application starts
    hooks.register("after_startup", |ctx: &PluginContext| {
        println!("Application started successfully");
        Ok(())
    })?;
    
    // Called before application shuts down
    hooks.register("before_shutdown", |ctx: &PluginContext| {
        println!("Application shutting down...");
        Ok(())
    })?;
    
    Ok(())
}
```

### Request Processing Hooks

```rust
fn register_request_hooks(hooks: &mut HookRegistry) -> PluginResult<()> {
    // Called before request processing
    hooks.register("before_request", |ctx: &PluginContext| {
        let req = ctx.request()?;
        println!("Processing request: {} {}", req.method(), req.uri());
        Ok(())
    })?;
    
    // Called after response generation
    hooks.register("after_response", |ctx: &PluginContext| {
        let resp = ctx.response()?;
        println!("Response status: {}", resp.status());
        Ok(())
    })?;
    
    Ok(())
}
```

### Route Registration Hooks

```rust
fn register_route_hooks(hooks: &mut HookRegistry) -> PluginResult<()> {
    // Called when routes are being registered
    hooks.register("before_route_registration", |ctx: &PluginContext| {
        // Modify route registration behavior
        Ok(())
    })?;
    
    Ok(())
}
```

## Plugin Manager

The plugin manager handles loading, initialization, and management of plugins:

```rust
use oxidite_plugin::{PluginManager, PluginLoader};

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();
    let mut plugin_manager = PluginManager::new();
    
    // Load plugins from configuration
    plugin_manager
        .load_plugin(Box::new(HelloWorldPlugin))
        .await?;
    
    // Initialize all plugins
    plugin_manager.initialize(&mut router).await?;
    
    Server::new(router)
        .listen("127.0.0.1:3000".parse().unwrap())
        .await
}
```

## Advanced Features

### Plugin Dependencies

```rust
use oxidite_plugin::{Plugin, PluginDependencies};

impl Plugin for MyPlugin {
    fn dependencies(&self) -> PluginDependencies {
        PluginDependencies::new()
            .requires("auth_plugin", ">=1.0.0")
            .optional("logging_plugin", ">=1.0.0")
    }
}
```

### Asset Serving

```rust
impl Plugin for AssetPlugin {
    fn assets(&self) -> Option<AssetProvider> {
        Some(AssetProvider::new()
            .add_directory("static", "./assets")
            .add_file("logo.png", "./assets/logo.png"))
    }
}
```

### Event System

```rust
// Emit events from plugins
hooks.emit("user_created", &UserData { id: 1, name: "John" })?;

// Listen for events
hooks.on("user_created", |data: &UserData| {
    println!("User created: {}", data.name);
    Ok(())
})?;
```

### Configuration Schema

```rust
use oxidite_plugin::ConfigSchema;

impl Plugin for ConfigurablePlugin {
    fn config_schema(&self) -> Option<ConfigSchema> {
        Some(ConfigSchema::new()
            .field("api_key", "API key for external service", "string")
            .field("timeout", "Request timeout in seconds", "integer")
            .field("enabled", "Enable plugin functionality", "boolean"))
    }
}
```

## Security

- **Sandboxed Execution**: Plugins run in a restricted environment
- **Permission System**: Fine-grained permission controls for plugin capabilities
- **Isolation**: Plugins are isolated from each other and the core application
- **Validation**: Input validation for all plugin-provided data

## Performance

- **Lazy Loading**: Plugins are loaded only when needed
- **Caching**: Cached plugin metadata and configurations
- **Efficient Hooks**: Optimized hook execution with minimal overhead
- **Resource Management**: Proper cleanup of plugin resources

## Best Practices

1. **Keep Plugins Focused**: Each plugin should have a single, well-defined purpose
2. **Use Configuration**: Make plugins configurable rather than hardcoded
3. **Handle Errors Gracefully**: Ensure plugins fail gracefully without affecting the main application
4. **Document Hooks**: Clearly document which hooks your plugin uses and provides
5. **Version Dependencies**: Specify compatible versions for plugin dependencies

## License

MIT