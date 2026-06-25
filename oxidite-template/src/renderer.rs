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
    /// Create a new renderer bound to the given context and optional engine
    pub fn new(context: &'a Context, engine: Option<&'a TemplateEngine>) -> Self {
        let filters = if let Some(eng) = engine {
            eng.filters.clone()
        } else {
            Filters::new()
        };
        Self {
            context,
            filters,
            engine,
            blocks: HashMap::new(),
        }
    }

    /// Render a compiled template node tree into a final output string
    ///
    /// Handles inheritance (`{% extends %}`), blocks, includes, variables,
    /// conditionals, and loops.
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
                TemplateNode::If { condition, then_branch, elif_branches, else_branch } => {
                    let value = self.render_if(condition, then_branch, elif_branches, else_branch)?;
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
                        // Resolve include path: check if it's a context variable first
                        let resolved_path = if let Some(val) = self.context.get(template_name) {
                            self.value_to_string(val)
                        } else {
                            template_name.clone()
                        };

                        let template = engine.get_template(&resolved_path)
                            .ok_or_else(|| TemplateError::RenderError(format!("Included template not found: {}", resolved_path)))?;
                        
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

        // Check if the `safe` filter is present (skip HTML escaping if so)
        let skip_escape = filter_names.iter().any(|f| f == "safe" || f == "raw");

        // Apply filters
        for filter_name in filter_names {
            // `safe` and `raw` are marker filters, not actual transformations
            if filter_name == "safe" || filter_name == "raw" {
                continue;
            }
            result = self.filters.apply(filter_name, &result)?;
        }

        // Auto-escape HTML unless `safe` or `raw` filter was applied
        if !skip_escape {
            result = html_escape(&result);
        }

        Ok(result)
    }

    fn render_if(
        &mut self,
        condition: &str,
        then_branch: &[TemplateNode],
        elif_branches: &[(String, Vec<TemplateNode>)],
        else_branch: &Option<Vec<TemplateNode>>,
    ) -> Result<String> {
        // Evaluate main condition
        if self.evaluate_condition(condition) {
            return self.render_nodes(then_branch);
        }

        // Evaluate elif branches in order
        for (elif_condition, elif_body) in elif_branches {
            if self.evaluate_condition(elif_condition) {
                return self.render_nodes(elif_body);
            }
        }

        // Fall back to else branch
        if let Some(else_nodes) = else_branch {
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
        let condition = condition.trim();

        // Handle `not` prefix
        if let Some(rest) = condition.strip_prefix("not ") {
            return !self.evaluate_condition(rest.trim());
        }

        // Handle `and` operator
        if let Some(pos) = self.find_operator(condition, " and ") {
            let left = &condition[..pos];
            let right = &condition[pos + 5..];
            return self.evaluate_condition(left) && self.evaluate_condition(right);
        }

        // Handle `or` operator
        if let Some(pos) = self.find_operator(condition, " or ") {
            let left = &condition[..pos];
            let right = &condition[pos + 4..];
            return self.evaluate_condition(left) || self.evaluate_condition(right);
        }

        // Handle comparison operators
        if let Some(result) = self.evaluate_comparison(condition) {
            return result;
        }

        // Simple truthy check (original behavior)
        self.is_truthy(condition)
    }

    /// Evaluate comparison expressions (==, !=, >, <, >=, <=)
    fn evaluate_comparison(&self, condition: &str) -> Option<bool> {
        // Try each operator in order (longest first to avoid ambiguity with >= vs >)
        let operators: &[(&str, fn(&Value, &Value) -> bool)] = &[
            ("==", |a: &Value, b: &Value| a == b),
            ("!=", |a: &Value, b: &Value| a != b),
            (">=", |a: &Value, b: &Value| Self::compare_values(a, b) >= 0),
            ("<=", |a: &Value, b: &Value| Self::compare_values(a, b) <= 0),
            (">",  |a: &Value, b: &Value| Self::compare_values(a, b) > 0),
            ("<",  |a: &Value, b: &Value| Self::compare_values(a, b) < 0),
        ];

        for &(op_str, op_fn) in operators {
            if let Some(pos) = condition.find(op_str) {
                let left = condition[..pos].trim();
                let right = condition[pos + op_str.len()..].trim();
                let left_val = self.resolve_value(left);
                let right_val = self.resolve_value(right);
                return Some(op_fn(&left_val, &right_val));
            }
        }

        None
    }

    /// Find an operator in the condition string, respecting quotes
    fn find_operator(&self, condition: &str, op: &str) -> Option<usize> {
        let mut in_quotes = false;
        let mut quote_char = '"';
        let chars: Vec<char> = condition.chars().collect();
        let op_chars: Vec<char> = op.chars().collect();

        for i in 0..chars.len() {
            if !in_quotes && (chars[i] == '"' || chars[i] == '\'') {
                in_quotes = true;
                quote_char = chars[i];
            } else if in_quotes && chars[i] == quote_char {
                in_quotes = false;
            } else if !in_quotes && i + op_chars.len() <= chars.len() {
                if chars[i..i + op_chars.len()] == op_chars[..] {
                    return Some(i);
                }
            }
        }
        None
    }

    /// Resolve a value expression to a serde_json Value
    fn resolve_value(&self, expr: &str) -> Value {
        let expr = expr.trim();

        // String literal
        if (expr.starts_with('"') && expr.ends_with('"'))
            || (expr.starts_with('\'') && expr.ends_with('\''))
        {
            return Value::String(expr[1..expr.len() - 1].to_string());
        }

        // Boolean literals
        if expr == "true" {
            return Value::Bool(true);
        }
        if expr == "false" {
            return Value::Bool(false);
        }

        // Null literal
        if expr == "null" || expr == "none" {
            return Value::Null;
        }

        // Number literal
        if let Ok(n) = expr.parse::<i64>() {
            return Value::Number(n.into());
        }
        if let Ok(n) = expr.parse::<f64>() {
            if let Some(num) = serde_json::Number::from_f64(n) {
                return Value::Number(num);
            }
        }

        // Variable reference (with dotted notation support)
        if let Some(val) = self.context.get(expr) {
            return val.clone();
        }

        Value::Null
    }

    /// Compare two JSON values numerically or lexicographically
    fn compare_values(a: &Value, b: &Value) -> i32 {
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => {
                let af = a.as_f64().unwrap_or(0.0);
                let bf = b.as_f64().unwrap_or(0.0);
                if af < bf { -1 } else if af > bf { 1 } else { 0 }
            }
            (Value::String(a), Value::String(b)) => a.cmp(b) as i32,
            _ => {
                // Fallback: compare string representations
                let as_str = a.to_string();
                let bs_str = b.to_string();
                as_str.cmp(&bs_str) as i32
            }
        }
    }

    /// Check if a variable name is truthy (original simple behavior)
    fn is_truthy(&self, name: &str) -> bool {
        if let Some(value) = self.context.get(name) {
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
