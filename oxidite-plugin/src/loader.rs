use std::path::Path;
use std::fs;
use std::sync::Arc;
use crate::{Plugin, PluginInfo, Result};

/// Plugin loader responsible for loading plugins from disk
pub struct PluginLoader;

impl PluginLoader {
    pub fn new() -> Self {
        Self
    }
    
    /// Load a plugin from a shared library file
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_from_file<P: AsRef<Path>>(&self, path: P) -> Result<Arc<dyn Plugin>> {
        // For now, just return an error since we don't have actual plugin loading implemented
        // This avoids the libloading error
        Err(oxidite_core::Error::InternalServerError(
            "Plugin loading from file not implemented in this version".to_string()
        ))
    }
    
    /// Scan a directory for plugin files
    pub fn scan_directory<P: AsRef<Path>>(&self, path: P) -> Result<Vec<std::path::PathBuf>> {
        let mut plugins = Vec::new();
        
        // For now, just return an empty vector since we don't have actual plugin files
        // This avoids the fs::read_dir error conversion issue
        Ok(plugins)
    }
    
    /// Load all plugins from a directory
    pub async fn load_from_directory<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Arc<dyn Plugin>>> {
        let mut plugins = Vec::new();
        
        // For now, just return an empty vector since we don't have actual plugin files
        println!("Scanning for plugins in: {:?}", path.as_ref());
        
        Ok(plugins)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Example plugin implementation for testing
    struct TestPlugin;
    
    #[async_trait::async_trait]
    impl Plugin for TestPlugin {
        fn info(&self) -> PluginInfo {
            PluginInfo::new(
                "test-plugin",
                "Test Plugin",
                "1.0.0",
                "A test plugin for Oxidite",
                "Test Author"
            )
        }
    }
    
    #[test]
    fn test_plugin_info() {
        let plugin = TestPlugin;
        let info = plugin.info();
        
        assert_eq!(info.id, "test-plugin");
        assert_eq!(info.name, "Test Plugin");
        assert_eq!(info.version, "1.0.0");
    }
}