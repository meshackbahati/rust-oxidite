//! Plugin system for Oxidite
//!
//! Provides dynamic loading and extension capabilities for Oxidite applications

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use oxidite_core::{Router, Result, Error};
use oxidite_config::Config;

pub mod plugin;
pub mod loader;
pub mod manager;

// Re-export types from plugin module but avoid conflicts
pub use plugin::{Plugin, PluginInfo, PluginHook, HookResult};
pub use loader::PluginLoader;
pub use manager::PluginManager;

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled_plugins: Vec<String>,
    pub plugin_directory: String,
    pub auto_reload: bool,
}

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