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
pub use static_files::{StaticFiles, serve_static, static_handler};
use oxidite_core::types::OxiditeResponse;

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
    filters: Filters,
}

impl TemplateEngine {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            filters: Filters::new(),
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

    /// Register a custom template filter.
    ///
    /// The filter function receives the string value of the variable and returns a new string.
    /// Once registered, the filter can be used in templates with `{{ var | filter_name }}`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut engine = TemplateEngine::new();
    /// engine.register_filter("shout", |s: &str| s.to_uppercase() + "!!!");
    /// ```
    pub fn register_filter(&mut self, name: impl Into<String>, filter: fn(&str) -> String) {
        self.filters.register(name.into(), filter);
    }

    pub fn render(&self, name: &str, context: &Context) -> Result<String> {
        let template = self.get_template(name)
            .ok_or_else(|| TemplateError::RenderError(format!("Template not found: {}", name)))?;
        
        let mut renderer = Renderer::new(context, Some(self));
        renderer.render(template)
    }
    
    /// Render a template as an HTML response
    pub fn render_response(&self, name: &str, context: &Context) -> Result<OxiditeResponse> {
        let html = self.render(name, context)?;
        Ok(OxiditeResponse::html(html))
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
    _source: String,
    parsed: Vec<TemplateNode>,
}

impl Template {
    pub fn new(source: impl Into<String>) -> Result<Self> {
        let source = source.into();
        let parser = Parser::new(&source);
        let parsed = parser.parse()?;

        Ok(Self { _source: source, parsed })
    }

    pub fn render(&self, context: &Context) -> Result<String> {
        let mut renderer = Renderer::new(context, None);
        renderer.render(self)
    }
    
    /// Render the template as an HTML response
    pub fn render_response(&self, context: &Context) -> Result<OxiditeResponse> {
        let html = self.render(context)?;
        Ok(OxiditeResponse::html(html))
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

    // ISSUE 1: safe filter tests
    #[test]
    fn test_safe_filter_skips_html_escaping() {
        let tmpl = Template::new("{{ content | safe }}").unwrap();
        let mut ctx = Context::new();
        ctx.set("content", "<p>Hello</p>");

        let result = tmpl.render(&ctx).unwrap();
        assert_eq!(result, "<p>Hello</p>");
    }

    #[test]
    fn test_raw_filter_skips_html_escaping() {
        let tmpl = Template::new("{{ content | raw }}").unwrap();
        let mut ctx = Context::new();
        ctx.set("content", "<b>Bold</b>");

        let result = tmpl.render(&ctx).unwrap();
        assert_eq!(result, "<b>Bold</b>");
    }

    #[test]
    fn test_auto_escape_without_safe_filter() {
        let tmpl = Template::new("{{ content }}").unwrap();
        let mut ctx = Context::new();
        ctx.set("content", "<p>Hello</p>");

        let result = tmpl.render(&ctx).unwrap();
        assert_eq!(result, "&lt;p&gt;Hello&lt;/p&gt;");
    }

    // ISSUE 3: comparison operator tests
    #[test]
    fn test_if_equality_comparison() {
        let mut engine = TemplateEngine::new();
        engine.add_template("test", r#"{% if status == "active" %}ON{% else %}OFF{% endif %}"#).unwrap();

        let mut ctx = Context::new();
        ctx.set("status", "active");
        assert_eq!(engine.render("test", &ctx).unwrap(), "ON");

        let mut ctx2 = Context::new();
        ctx2.set("status", "inactive");
        assert_eq!(engine.render("test", &ctx2).unwrap(), "OFF");
    }

    #[test]
    fn test_if_inequality_comparison() {
        let mut engine = TemplateEngine::new();
        engine.add_template("test", r#"{% if status != "draft" %}Published{% else %}Draft{% endif %}"#).unwrap();

        let mut ctx = Context::new();
        ctx.set("status", "published");
        assert_eq!(engine.render("test", &ctx).unwrap(), "Published");

        let mut ctx2 = Context::new();
        ctx2.set("status", "draft");
        assert_eq!(engine.render("test", &ctx2).unwrap(), "Draft");
    }

    #[test]
    fn test_if_numeric_comparison() {
        let mut engine = TemplateEngine::new();
        engine.add_template("test", "{% if count > 5 %}many{% else %}few{% endif %}").unwrap();

        let mut ctx = Context::new();
        ctx.set("count", 10);
        assert_eq!(engine.render("test", &ctx).unwrap(), "many");

        let mut ctx2 = Context::new();
        ctx2.set("count", 3);
        assert_eq!(engine.render("test", &ctx2).unwrap(), "few");
    }

    // ISSUE 5: elif tests
    #[test]
    fn test_elif_branches() {
        let mut engine = TemplateEngine::new();
        engine.add_template("test", r#"{% if color == "red" %}RED{% elif color == "blue" %}BLUE{% elif color == "green" %}GREEN{% else %}OTHER{% endif %}"#).unwrap();

        let mut ctx = Context::new();
        ctx.set("color", "red");
        assert_eq!(engine.render("test", &ctx).unwrap(), "RED");

        let mut ctx2 = Context::new();
        ctx2.set("color", "blue");
        assert_eq!(engine.render("test", &ctx2).unwrap(), "BLUE");

        let mut ctx3 = Context::new();
        ctx3.set("color", "green");
        assert_eq!(engine.render("test", &ctx3).unwrap(), "GREEN");

        let mut ctx4 = Context::new();
        ctx4.set("color", "yellow");
        assert_eq!(engine.render("test", &ctx4).unwrap(), "OTHER");
    }

    // ISSUE 6: custom filter registration test
    #[test]
    fn test_register_custom_filter() {
        let mut engine = TemplateEngine::new();
        engine.register_filter("shout", |s: &str| s.to_uppercase() + "!!!");
        engine.add_template("test", "{{ greeting | shout }}").unwrap();

        let mut ctx = Context::new();
        ctx.set("greeting", "hello");

        let result = engine.render("test", &ctx).unwrap();
        assert_eq!(result, "HELLO!!!");
    }

    // ISSUE 2: variable include test
    #[test]
    fn test_include_with_variable() {
        let mut engine = TemplateEngine::new();
        engine.add_template("partials/header.html", "<header>HEADER</header>").unwrap();
        engine.add_template("partials/footer.html", "<footer>FOOTER</footer>").unwrap();
        engine.add_template("page", "{% include partial_path %}").unwrap();

        let mut ctx = Context::new();
        ctx.set("partial_path", "partials/header.html");
        assert_eq!(engine.render("page", &ctx).unwrap(), "<header>HEADER</header>");

        let mut ctx2 = Context::new();
        ctx2.set("partial_path", "partials/footer.html");
        assert_eq!(engine.render("page", &ctx2).unwrap(), "<footer>FOOTER</footer>");
    }

    #[test]
    fn test_include_with_string_literal_still_works() {
        let mut engine = TemplateEngine::new();
        engine.add_template("partials/nav.html", "<nav>NAV</nav>").unwrap();
        engine.add_template("page", r#"{% include "partials/nav.html" %}"#).unwrap();

        let ctx = Context::new();
        assert_eq!(engine.render("page", &ctx).unwrap(), "<nav>NAV</nav>");
    }
}
