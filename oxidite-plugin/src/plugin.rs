use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use oxidite_core::{OxiditeResponse, Error, Result};

// Don't import the types that cause conflicts


/// Plugin trait that defines the interface for all plugins
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Get plugin information
    fn info(&self) -> PluginInfo;
    
    /// Called when the plugin is loaded
    async fn on_load(&self) -> Result<()> {
        Ok(())
    }
    
    /// Called when the plugin is unloaded
    async fn on_unload(&self) -> Result<()> {
        Ok(())
    }
    
    /// Called when the plugin is enabled
    async fn on_enable(&self) -> Result<()> {
        Ok(())
    }
    
    /// Called when the plugin is disabled
    async fn on_disable(&self) -> Result<()> {
        Ok(())
    }
    
    /// Called before the application starts
    async fn on_startup(&self) -> Result<()> {
        Ok(())
    }
    
    /// Called after the application shuts down
    async fn on_shutdown(&self) -> Result<()> {
        Ok(())
    }
    
    /// Hook into various parts of the application lifecycle
    async fn hook(&self, hook: PluginHook) -> HookResult {
        HookResult::Continue
    }
}

/// Information about a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub enabled: bool,
}

impl PluginInfo {
    pub fn new(id: &str, name: &str, version: &str, description: &str, author: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            version: version.to_string(),
            description: description.to_string(),
            author: author.to_string(),
            license: "MIT".to_string(),
            homepage: None,
            repository: None,
            enabled: false,
        }
    }
}

/// Different hooks that plugins can implement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginHook {
    /// Called before a request is processed
    PreRequest { path: String, method: String },
    
    /// Called after a response is generated
    PostResponse { path: String, method: String, status: u16 },
    
    /// Called when a user authenticates
    OnAuth { user_id: String },
    
    /// Called when a user logs out
    OnLogout { user_id: String },
    
    /// Called when a model is created
    OnModelCreate { model: String, id: String },
    
    /// Called when a model is updated
    OnModelUpdate { model: String, id: String },
    
    /// Called when a model is deleted
    OnModelDelete { model: String, id: String },
    
    /// Custom hook with arbitrary data
    Custom { name: String, data: serde_json::Value },
}

/// Result of a hook execution
// Changed to avoid Debug requirement on OxiditeResponse
#[derive(Debug)]
pub enum HookResult {
    /// Continue with normal execution
    Continue,
    
    /// Stop execution and return early
    Stop,
    
    /// Return a modified response - using a placeholder to avoid Debug issue
    Response(String),
    
    /// Return an error
    Error(Error),
    
    /// Transform data in the hook chain
    Transform(serde_json::Value),
}