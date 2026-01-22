use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use oxidite_core::{Result, Error};

use crate::{Plugin, PluginInfo, PluginHook, HookResult, PluginLoader, PluginConfig};

/// Main plugin manager
pub struct PluginManager {
    plugins: HashMap<String, Arc<dyn Plugin>>,
    config: PluginConfig,
    hooks: HashMap<String, Vec<Arc<dyn Plugin>>>,
}

impl PluginManager {
    pub fn new(config: PluginConfig) -> Self {
        Self {
            plugins: HashMap::new(),
            config,
            hooks: HashMap::new(),
        }
    }
    
    /// Load plugins from a directory
    pub async fn load_plugins_from_directory<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let loader = PluginLoader::new();
        
        // For now, we'll just simulate loading
        // In a real implementation, this would dynamically load .so/.dll files
        println!("Loading plugins from: {:?}", path.as_ref());
        
        Ok(())
    }
    
    /// Register a plugin
    pub fn register_plugin(&mut self, plugin: Arc<dyn Plugin>) -> Result<()> {
        let info = plugin.info();
        
        if self.plugins.contains_key(&info.id) {
            return Err(Error::InternalServerError(
                format!("Plugin with id '{}' already exists", info.id)
            ));
        }
        
        self.plugins.insert(info.id.clone(), plugin);
        
        Ok(())
    }
    
    /// Enable a plugin
    pub async fn enable_plugin(&mut self, plugin_id: &str) -> Result<()> {
        if let Some(plugin) = self.plugins.get(plugin_id) {
            plugin.on_enable().await?;
            
            // Update plugin info to enabled
            // Note: In a real implementation, we'd need mutable access to update the info
            
            Ok(())
        } else {
            Err(Error::NotFound(format!("Plugin '{}' not found", plugin_id)))
        }
    }
    
    /// Disable a plugin
    pub async fn disable_plugin(&mut self, plugin_id: &str) -> Result<()> {
        if let Some(plugin) = self.plugins.get(plugin_id) {
            plugin.on_disable().await?;
            Ok(())
        } else {
            Err(Error::NotFound(format!("Plugin '{}' not found", plugin_id)))
        }
    }
    
    /// Execute a hook across all registered plugins
    pub async fn execute_hook(&self, hook: PluginHook) -> Result<HookResult> {
        let mut result = HookResult::Continue;
        
        for plugin in self.plugins.values() {
            if !plugin.info().enabled {
                continue;
            }
            
            result = plugin.hook(hook.clone()).await;
            
            match result {
                HookResult::Stop => break,
                HookResult::Response(_) => return Ok(result),
                HookResult::Error(_) => return Ok(result),
                _ => continue,
            }
        }
        
        Ok(result)
    }
    
    /// Get a list of all plugins
    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        self.plugins.values()
            .map(|p| p.info())
            .collect()
    }
    
    /// Initialize all enabled plugins
    pub async fn initialize(&self) -> Result<()> {
        for plugin in self.plugins.values() {
            if plugin.info().enabled {
                plugin.on_load().await?;
            }
        }
        
        Ok(())
    }
    
    /// Shutdown all plugins
    pub async fn shutdown(&self) -> Result<()> {
        for plugin in self.plugins.values() {
            plugin.on_unload().await?;
        }
        
        Ok(())
    }
}

/// Helper function to create a plugin manager
pub fn create_manager(config: PluginConfig) -> PluginManager {
    PluginManager::new(config)
}

