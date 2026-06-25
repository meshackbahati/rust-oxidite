//! Plugin system for Oxidite
//!
//! Provides dynamic loading and extension capabilities for Oxidite applications

use serde::{Deserialize, Serialize};

/// Plugin trait and type definitions.
pub mod plugin;
/// Plugin loading from disk.
pub mod loader;
/// Plugin lifecycle management.
pub mod manager;

/// Re-export of plugin types.
pub use plugin::{Plugin, PluginInfo, PluginHook, HookResult};
/// Re-export of the plugin loader.
pub use loader::PluginLoader;
/// Re-export of the plugin manager.
pub use manager::PluginManager;

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// List of enabled plugin IDs.
    pub enabled_plugins: Vec<String>,
    /// Directory to scan for plugin files.
    pub plugin_directory: String,
    /// Whether to automatically reload plugins on file changes.
    pub auto_reload: bool,
}

/// Returns a default configuration with an empty plugin list, `./plugins` directory, and auto-reload disabled.
impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled_plugins: Vec::new(),
            plugin_directory: "./plugins".to_string(),
            auto_reload: false,
        }
    }
}

/// Helper function to create a plugin manager
pub fn create_manager(config: PluginConfig) -> PluginManager {
    PluginManager::new(config)
}
