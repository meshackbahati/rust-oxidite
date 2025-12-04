use crate::{TemplateError, Result};
use regex::Regex;

/// Template AST nodes
#[derive(Debug, Clone, PartialEq)]
pub enum TemplateNode {
    Text(String),
    Variable { name: String, filters: Vec<String> },
    If { condition: String, then_branch: Vec<TemplateNode>, else_branch: Option<Vec<TemplateNode>> },
    For { item: String, iterable: String, body: Vec<TemplateNode> },
}

/// Template parser
pub struct Parser {
    source: String,
}

impl Parser {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
        }
    }

    pub fn parse(&self) -> Result<Vec<TemplateNode>> {
        let mut nodes = Vec::new();
        let mut pos = 0;
        let source = self.source.as_str();

        while pos < source.len() {
            // Try to parse template tags
            if let Some((node, new_pos)) = self.parse_tag(&source[pos..])? {
                nodes.push(node);
                pos += new_pos;
            } else if let Some((text, new_pos)) = self.parse_text(&source[pos..]) {
                nodes.push(TemplateNode::Text(text));
                pos += new_pos;
            } else {
                break;
            }
        }

        Ok(nodes)
    }

    fn parse_tag(&self, source: &str) -> Result<Option<(TemplateNode, usize)>> {
        // Variable: {{ variable | filter }}
        if source.starts_with("{{") {
            return self.parse_variable(source);
        }

        // Control structures: {% if %}, {% for %}
        if source.starts_with("{%") {
            return self.parse_control(source);
        }

        Ok(None)
    }

    fn parse_variable(&self, source: &str) -> Result<Option<(TemplateNode, usize)>> {
        let re = Regex::new(r"\{\{\s*([a-zA-Z0-9_.]+)(\s*\|\s*([a-zA-Z0-9_]+))?\s*\}\}").unwrap();
        
        if let Some(cap) = re.captures(source) {
            let full_match = cap.get(0).unwrap();
            let var_name = cap.get(1).unwrap().as_str().to_string();
            let filter = cap.get(3).map(|m| vec![m.as_str().to_string()]).unwrap_or_default();

            let node = TemplateNode::Variable {
                name: var_name,
                filters: filter,
            };

            return Ok(Some((node, full_match.end())));
        }

        Ok(None)
    }

    fn parse_control(&self, source: &str) -> Result<Option<(TemplateNode, usize)>> {
        // {% if condition %}
        if source.starts_with("{% if ") {
            return self.parse_if(source);
        }

        // {% for item in iterable %}
        if source.starts_with("{% for ") {
            return self.parse_for(source);
        }

        Ok(None)
    }

    fn parse_if(&self, source: &str) -> Result<Option<(TemplateNode, usize)>> {
        let re_if = Regex::new(r"\{%\s*if\s+([a-zA-Z0-9_.]+)\s*%\}").unwrap();
        
        if let Some(cap) = re_if.captures(source) {
            let condition = cap.get(1).unwrap().as_str().to_string();
            let start_pos = cap.get(0).unwrap().end();

            // Find {% endif %}
            let endif_pattern = "{% endif %}";
            let else_pattern = "{% else %}";

            if let Some(endif_pos) = source[start_pos..].find(endif_pattern) {
                let body_source = &source[start_pos..start_pos + endif_pos];
                
                // Check for {% else %}
                let (then_branch, else_branch) = if let Some(else_pos) = body_source.find(else_pattern) {
                    let then_source = &body_source[..else_pos];
                    let else_source = &body_source[else_pos + else_pattern.len()..];
                    
                    let parser_then = Parser::new(then_source);
                    let parser_else = Parser::new(else_source);
                    
                    (parser_then.parse()?, Some(parser_else.parse()?))
                } else {
                    let parser = Parser::new(body_source);
                    (parser.parse()?, None)
                };

                let node = TemplateNode::If {
                    condition,
                    then_branch,
                    else_branch,
                };

                let total_len = start_pos + endif_pos + endif_pattern.len();
                return Ok(Some((node, total_len)));
            }
        }

        Ok(None)
    }

    fn parse_for(&self, source: &str) -> Result<Option<(TemplateNode, usize)>> {
        let re_for = Regex::new(r"\{%\s*for\s+([a-zA-Z0-9_]+)\s+in\s+([a-zA-Z0-9_.]+)\s*%\}").unwrap();
        
        if let Some(cap) = re_for.captures(source) {
            let item = cap.get(1).unwrap().as_str().to_string();
            let iterable = cap.get(2).unwrap().as_str().to_string();
            let start_pos = cap.get(0).unwrap().end();

            // Find {% endfor %}
            let endfor_pattern = "{% endfor %}";
            if let Some(endfor_pos) = source[start_pos..].find(endfor_pattern) {
                let body_source = &source[start_pos..start_pos + endfor_pos];
                let parser = Parser::new(body_source);
                let body = parser.parse()?;

                let node = TemplateNode::For {
                    item,
                    iterable,
                    body,
                };

                let total_len = start_pos + endfor_pos + endfor_pattern.len();
                return Ok(Some((node, total_len)));
            }
        }

        Ok(None)
    }

    fn parse_text(&self, source: &str) -> Option<(String, usize)> {
        // Find next template tag
        let next_tag = source.find("{{").or_else(|| source.find("{%"));
        
        if let Some(pos) = next_tag {
            if pos > 0 {
                Some((source[..pos].to_string(), pos))
            } else {
                None
            }
        } else {
            // No more tags, rest is text
            Some((source.to_string(), source.len()))
        }
    }
}
