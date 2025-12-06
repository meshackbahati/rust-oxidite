use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::fs;

pub mod parser;
pub mod renderer;
pub mod filters;
pub mod static_files;

pub use parser::{Parser, TemplateNode};
pub use renderer::Renderer;
pub use filters::Filters;
pub use static_files::{StaticFiles, serve_static};

/// Template context for variable interpolation
#[derive(Debug, Clone)]
pub struct Context {
    data: HashMap<String, Value>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn set<T: serde::Serialize>(&mut self, key: impl Into<String>, value: T) {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.data.insert(key.into(), json_value);
        }
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        // Support dotted notation: user.name
        let parts: Vec<&str> = key.split('.').collect();
        let mut current = self.data.get(parts[0])?;

        for part in &parts[1..] {
            current = current.get(part)?;
        }

        Some(current)
    }

    pub fn from_json(json: Value) -> Self {
        let mut context = Self::new();
        if let Value::Object(map) = json {
            for (key, value) in map {
                context.data.insert(key, value);
            }
        }
        context
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

/// Template engine to manage multiple templates
pub struct TemplateEngine {
    templates: HashMap<String, Template>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    pub fn add_template(&mut self, name: impl Into<String>, source: impl Into<String>) -> Result<()> {
        let template = Template::new(source)?;
        self.templates.insert(name.into(), template);
        Ok(())
    }

    pub fn get_template(&self, name: &str) -> Option<&Template> {
        self.templates.get(name)
    }

    pub fn render(&self, name: &str, context: &Context) -> Result<String> {
        let template = self.get_template(name)
            .ok_or_else(|| TemplateError::RenderError(format!("Template not found: {}", name)))?;
        
        let mut renderer = Renderer::new(context, Some(self));
        renderer.render(template)
    }
    
    /// Load all templates from a directory (recursive)
    pub fn load_dir(&mut self, dir: impl AsRef<Path>) -> Result<usize> {
        let dir = dir.as_ref();
        let mut count = 0;
        
        if !dir.is_dir() {
            return Err(TemplateError::RenderError(format!("Not a directory: {:?}", dir)));
        }
        
        self.load_dir_recursive(dir, dir, &mut count)?;
        Ok(count)
    }
    
    fn load_dir_recursive(&mut self, base_dir: &Path, current_dir: &Path, count: &mut usize) -> Result<()> {
        for entry in fs::read_dir(current_dir)
            .map_err(|e| TemplateError::RenderError(format!("Failed to read directory: {}", e)))? 
        {
            let entry = entry.map_err(|e| TemplateError::RenderError(e.to_string()))?;
            let path = entry.path();
            
            if path.is_dir() {
                // Recursively load templates from subdirectories
                self.load_dir_recursive(base_dir, &path, count)?;
            } else if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "html" || ext == "htm" {
                        let content = fs::read_to_string(&path)
                            .map_err(|e| TemplateError::RenderError(format!("Failed to read file: {}", e)))?;
                        
                        // Get relative path from base_dir to preserve directory structure
                        let relative_path = path.strip_prefix(base_dir)
                            .map_err(|e| TemplateError::RenderError(e.to_string()))?;
                        
                        let name = relative_path.to_str()
                            .ok_or_else(|| TemplateError::RenderError("Invalid filename".to_string()))?;
                        
                        self.add_template(name, content)?;
                        *count += 1;
                    }
                }
            }
        }
        
        Ok(())
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Template
#[derive(Debug, Clone)]
pub struct Template {
    source: String,
    parsed: Vec<TemplateNode>,
}

impl Template {
    pub fn new(source: impl Into<String>) -> Result<Self> {
        let source = source.into();
        let parser = Parser::new(&source);
        let parsed = parser.parse()?;

        Ok(Self { source, parsed })
    }

    pub fn render(&self, context: &Context) -> Result<String> {
        let mut renderer = Renderer::new(context, None);
        renderer.render(self)
    }
}

/// Template errors
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Render error: {0}")]
    RenderError(String),

    #[error("Variable not found: {0}")]
    VariableNotFound(String),

    #[error("Filter not found: {0}")]
    FilterNotFound(String),
}

pub type Result<T> = std::result::Result<T, TemplateError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_variable() {
        let tmpl = Template::new("Hello {{ name }}!").unwrap();
        let mut ctx = Context::new();
        ctx.set("name", "World");
        
        let result = tmpl.render(&ctx).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_dotted_notation() {
        let tmpl = Template::new("Hello {{ user.name }}!").unwrap();
        let mut ctx = Context::new();
        ctx.set("user", serde_json::json!({ "name": "Alice" }));
        
        let result = tmpl.render(&ctx).unwrap();
        assert_eq!(result, "Hello Alice!");
    }
}
