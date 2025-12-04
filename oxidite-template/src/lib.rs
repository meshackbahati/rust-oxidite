use serde_json::Value;
use std::collections::HashMap;

pub mod parser;
pub mod renderer;
pub mod filters;

pub use parser::{Parser, TemplateNode};
pub use renderer::Renderer;
pub use filters::Filters;

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

/// Template engine
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
        let renderer = Renderer::new(context);
        renderer.render(&self.parsed)
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
