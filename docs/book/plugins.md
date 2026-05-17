# Plugin System

Oxidite features a powerful, extensible plugin system that allows you to hook into the framework's lifecycle, add global middleware, register custom routes, and extend the application state.

## Overview

A plugin in Oxidite is any type that implements the `Plugin` trait. Plugins are registered during the application setup and are executed in the order they are added.

## The Plugin Trait

```rust,ignore
#[async_trait]
pub trait Plugin: Send + Sync {
    /// The name of the plugin for logging and diagnostics.
    fn name(&self) -> &'static str;

    /// Called during the initial configuration of the router.
    async fn on_setup(&self, router: &mut Router) -> Result<()> {
        Ok(())
    }

    /// Called after the server has started listening.
    async fn on_startup(&self) -> Result<()> {
        Ok(())
    }

    /// Called when the server is shutting down.
    async fn on_shutdown(&self) -> Result<()> {
        Ok(())
    }
}
```

## Creating a Plugin

Here's an example of a simple `RequestTracker` plugin that logs request statistics.

```rust,ignore
use oxidite::prelude::*;
use async_trait::async_trait;

pub struct RequestTrackerPlugin;

#[async_trait]
impl Plugin for RequestTrackerPlugin {
    fn name(&self) -> &'static str {
        "RequestTracker"
    }

    async fn on_setup(&self, router: &mut Router) -> Result<()> {
        // Add a global middleware
        router.middleware(|req, next| async move {
            let start = std::time::Instant::now();
            let response = next.run(req).await?;
            println!("Request processed in {:?}", start.elapsed());
            Ok(response)
        });

        // Register a diagnostic route
        router.get("/_sys/health", |_req| async {
            Ok(Response::json(serde_json::json!({ "status": "healthy" })))
        });

        Ok(())
    }

    async fn on_startup(&self) -> Result<()> {
        println!("RequestTracker plugin started!");
        Ok(())
    }
}
```

## Registering Plugins

Register your plugins using the `router.plugin()` method:

```rust,ignore
#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Router::new();

    // Register plugins
    router.plugin(RequestTrackerPlugin);
    router.plugin(DatabasePlugin::new("sqlite::memory:"));

    Server::new(router)
        .listen("127.0.0.1:3000".parse()?)
        .await
}
```

## Advanced Plugin Features

### State Injection

Plugins can inject their own state into the application, making it available to all handlers.

```rust,ignore
async fn on_setup(&self, router: &mut Router) -> Result<()> {
    let service = MyCustomService::new();
    router.with_state(service);
    Ok(())
}
```

### Hooking into Events

If your application uses the `oxidite-events` system, plugins can register listeners for global events.

```rust,ignore
async fn on_setup(&self, router: &mut Router) -> Result<()> {
    router.on("user.signed_up", |event| async move {
        println!("User signed up: {:?}", event.payload);
        Ok(())
    });
    Ok(())
}
```

## Best Practices

1. **Naming**: Use clear, descriptive names for your plugins.
2. **Order of Registration**: Remember that plugins registered later can override or wrap behavior of plugins registered earlier.
3. **Error Handling**: Implement robust error handling in `on_setup` to prevent the application from starting in an inconsistent state.
4. **Lightweight Startup**: Keep `on_startup` and `on_shutdown` logic lightweight to ensure fast application lifecycles.