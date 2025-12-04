use crate::{Context, TemplateNode, TemplateError, Result, filters::Filters};
use serde_json::Value;

/// Template renderer
pub struct Renderer<'a> {
    context: &'a Context,
    filters: Filters,
}

impl<'a> Renderer<'a> {
    pub fn new(context: &'a Context) -> Self {
        Self {
            context,
            filters: Filters::new(),
        }
    }

    pub fn render(&self, nodes: &[TemplateNode]) -> Result<String> {
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

    fn render_if(&self, condition: &str, then_branch: &[TemplateNode], else_branch: &Option<Vec<TemplateNode>>) -> Result<String> {
        // Evaluate condition (simple truthy check)
        let is_truthy = self.evaluate_condition(condition);

        if is_truthy {
            self.render(then_branch)
        } else if let Some(else_nodes) = else_branch {
            self.render(else_nodes)
        } else {
            Ok(String::new())
        }
    }

    fn render_for(&self, item: &str, iterable: &str, body: &[TemplateNode]) -> Result<String> {
        let array = self.context.get(iterable)
            .ok_or_else(|| TemplateError::VariableNotFound(iterable.to_string()))?;

        let mut output = String::new();

        if let Value::Array(items) = array {
            for item_value in items {
                // Create new context with loop variable
                let mut loop_context = self.context.clone();
                loop_context.data.insert(item.to_string(), item_value.clone());

                let renderer = Renderer::new(&loop_context);
                output.push_str(&renderer.render(body)?);
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
