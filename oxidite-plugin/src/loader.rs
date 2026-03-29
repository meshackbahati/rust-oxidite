use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use crate::Plugin;
use oxidite_core::Result;

/// Plugin loader responsible for loading plugins from disk
pub struct PluginLoader;

impl PluginLoader {
    pub fn new() -> Self {
        Self
    }
    
    /// Load a plugin from a shared library file
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_from_file<P: AsRef<Path>>(&self, _path: P) -> Result<Arc<dyn Plugin>> {
        // For now, just return an error since we don't have actual plugin loading implemented
        // This avoids the libloading error
        Err(oxidite_core::Error::InternalServerError(
            "Plugin loading from file not implemented in this version".to_string()
        ))
    }
    
    /// Scan a directory for plugin files
    pub fn scan_directory<P: AsRef<Path>>(&self, path: P) -> Result<Vec<PathBuf>> {
        let dir = path.as_ref();
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut plugins = Vec::new();
        for entry in fs::read_dir(dir)
            .map_err(|e| oxidite_core::Error::InternalServerError(e.to_string()))?
        {
            let entry = entry
                .map_err(|e| oxidite_core::Error::InternalServerError(e.to_string()))?;
            let path = entry.path();

            let is_plugin_file = path.extension()
                .and_then(|e| e.to_str())
                .map(|ext| matches!(ext, "so" | "dylib" | "dll"))
                .unwrap_or(false);
            if is_plugin_file {
                plugins.push(path);
            }
        }

        Ok(plugins)
    }
    
    /// Load all plugins from a directory
    pub async fn load_from_directory<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Arc<dyn Plugin>>> {
        println!("Scanning for plugins in: {:?}", path.as_ref());
        let mut plugins = Vec::new();
        for plugin_path in self.scan_directory(path)? {
            match self.load_from_file(&plugin_path) {
                Ok(plugin) => plugins.push(plugin),
                Err(e) => {
                    eprintln!("Failed to load plugin {:?}: {}", plugin_path, e);
                }
            }
        }
        Ok(plugins)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PluginInfo;
    
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
