use crate::{Context, TemplateNode, TemplateError, Result, filters::Filters, TemplateEngine, Template};
use serde_json::Value;
use std::collections::HashMap;

/// Template renderer
pub struct Renderer<'a> {
    context: &'a Context,
    filters: Filters,
    engine: Option<&'a TemplateEngine>,
    blocks: HashMap<String, Vec<TemplateNode>>,
}

impl<'a> Renderer<'a> {
    pub fn new(context: &'a Context, engine: Option<&'a TemplateEngine>) -> Self {
        Self {
            context,
            filters: Filters::new(),
            engine,
            blocks: HashMap::new(),
        }
    }

    pub fn render(&mut self, template: &Template) -> Result<String> {
        // Check for Extends (ignoring leading whitespace)
        let extends_node = template.parsed.iter().find(|node| {
            match node {
                TemplateNode::Text(t) => !t.trim().is_empty(),
                _ => true,
            }
        });

        if let Some(TemplateNode::Extends(parent_name)) = extends_node {
            // Collect blocks from current template (child)
            // We only collect top-level blocks in the child template
            for node in &template.parsed {
                if let TemplateNode::Block { name, body } = node {
                    // Only insert if not already present (child overrides parent, but we are going up)
                    // Wait, we start at child. Child blocks should override everything.
                    // So we insert. But if we are in a chain C -> B -> A.
                    // We render C. C extends B. We collect C blocks. Recurse to B.
                    // B extends A. We collect B blocks. If B defines "content" and C defined "content", C wins.
                    // So we use entry().or_insert().
                    self.blocks.entry(name.clone()).or_insert(body.clone());
                }
            }

            if let Some(engine) = self.engine {
                let parent = engine.get_template(parent_name)
                    .ok_or_else(|| TemplateError::RenderError(format!("Parent template not found: {}", parent_name)))?;
                return self.render(parent);
            } else {
                return Err(TemplateError::RenderError("Extends used without TemplateEngine".to_string()));
            }
        }

        self.render_nodes(&template.parsed)
    }

    fn render_nodes(&mut self, nodes: &[TemplateNode]) -> Result<String> {
        let mut output = String::new();

        for node in nodes {
            match node {
                TemplateNode::Text(text) => {
                    output.push_str(text);
                }
                TemplateNode::Variable { name, filters } => {
                    let value = self.render_variable(name, filters)?;
                    output.push_str(&value);
                }
                TemplateNode::If { condition, then_branch, else_branch } => {
                    let value = self.render_if(condition, then_branch, else_branch)?;
                    output.push_str(&value);
                }
                TemplateNode::For { item, iterable, body } => {
                    let value = self.render_for(item, iterable, body)?;
                    output.push_str(&value);
                }
                TemplateNode::Block { name, body } => {
                    // If block is overridden, use that, else use default body
                    if let Some(override_body) = self.blocks.get(name).cloned() {
                        // We need to render the override body
                        let nodes = override_body; 
                        output.push_str(&self.render_nodes(&nodes)?);
                    } else {
                        output.push_str(&self.render_nodes(body)?);
                    }
                }
                TemplateNode::Extends(_) => {
                    // Should not happen inside render_nodes (only at top level)
                    // But if it does, ignore or error?
                    // Ignore for now.
                }
                TemplateNode::Include(template_name) => {
                    if let Some(engine) = self.engine {
                        let template = engine.get_template(template_name)
                            .ok_or_else(|| TemplateError::RenderError(format!("Included template not found: {}", template_name)))?;
                        
                        // Includes are rendered in-place with current context
                        // They do NOT inherit blocks (usually).
                        // So we create a new renderer for the include, but share context/engine.
                        // But we don't pass `self.blocks`?
                        // Correct, includes are isolated from inheritance chain usually.
                        let mut sub_renderer = Renderer::new(self.context, self.engine);
                        output.push_str(&sub_renderer.render(template)?);
                    } else {
                         return Err(TemplateError::RenderError("Include used without TemplateEngine".to_string()));
                    }
                }
            }
        }

        Ok(output)
    }

    fn render_variable(&self, name: &str, filter_names: &[String]) -> Result<String> {
        let value = self.context.get(name)
            .ok_or_else(|| TemplateError::VariableNotFound(name.to_string()))?;

        let mut result = self.value_to_string(value);

        // Apply filters
        for filter_name in filter_names {
            result = self.filters.apply(filter_name, &result)?;
        }

        // Auto-escape HTML
        result = html_escape(&result);

        Ok(result)
    }

    fn render_if(&mut self, condition: &str, then_branch: &[TemplateNode], else_branch: &Option<Vec<TemplateNode>>) -> Result<String> {
        // Evaluate condition (simple truthy check)
        let is_truthy = self.evaluate_condition(condition);

        if is_truthy {
            self.render_nodes(then_branch)
        } else if let Some(else_nodes) = else_branch {
            self.render_nodes(else_nodes)
        } else {
            Ok(String::new())
        }
    }

    fn render_for(&mut self, item: &str, iterable: &str, body: &[TemplateNode]) -> Result<String> {
        let array = self.context.get(iterable)
            .ok_or_else(|| TemplateError::VariableNotFound(iterable.to_string()))?;

        let mut output = String::new();

        if let Value::Array(items) = array {
            for item_value in items {
                // Create new context with loop variable
                let mut loop_context = self.context.clone();
                loop_context.data.insert(item.to_string(), item_value.clone());

                let mut renderer = Renderer::new(&loop_context, self.engine);
                // Pass blocks to loop renderer?
                // Loops are inside the template, so they should have access to blocks?
                // Yes, if I use a block inside a loop?
                renderer.blocks = self.blocks.clone();
                output.push_str(&renderer.render_nodes(body)?);
            }
        }

        Ok(output)
    }

    fn evaluate_condition(&self, condition: &str) -> bool {
        if let Some(value) = self.context.get(condition) {
            match value {
                Value::Bool(b) => *b,
                Value::Null => false,
                Value::String(s) => !s.is_empty(),
                Value::Number(_) => true,
                Value::Array(a) => !a.is_empty(),
                Value::Object(o) => !o.is_empty(),
            }
        } else {
            false
        }
    }

    fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => String::new(),
            _ => serde_json::to_string(value).unwrap_or_default(),
        }
    }
}

/// HTML escape for XSS protection
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
