use crate::Result;
use regex::Regex;

/// Template AST nodes
#[derive(Debug, Clone, PartialEq)]
pub enum TemplateNode {
    Text(String),
    Variable { name: String, filters: Vec<String> },
    If { condition: String, then_branch: Vec<TemplateNode>, else_branch: Option<Vec<TemplateNode>> },
    For { item: String, iterable: String, body: Vec<TemplateNode> },
    Block { name: String, body: Vec<TemplateNode> },
    Extends(String),
    Include(String),
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

        // Control structures: {% if %}, {% for %}, {% block %}, {% extends %}, {% include %}
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

        // {% block name %}
        if source.starts_with("{% block ") {
            return self.parse_block(source);
        }

        // {% extends "template" %}
        if source.starts_with("{% extends ") {
            return self.parse_extends(source);
        }

        // {% include "template" %}
        if source.starts_with("{% include ") {
            return self.parse_include(source);
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

    fn parse_block(&self, source: &str) -> Result<Option<(TemplateNode, usize)>> {
        let re_block = Regex::new(r"\{%\s*block\s+([a-zA-Z0-9_]+)\s*%\}").unwrap();
        
        if let Some(cap) = re_block.captures(source) {
            let name = cap.get(1).unwrap().as_str().to_string();
            let start_pos = cap.get(0).unwrap().end();

            // Find matching {% endblock %}
            let mut nesting = 1;
            let mut current_pos = start_pos;
            
            while nesting > 0 {
                 let next_open = source[current_pos..].find("{% block ");
                 let next_close = source[current_pos..].find("{% endblock %}");
                 
                 match (next_open, next_close) {
                     (Some(open), Some(close)) => {
                         if open < close {
                             nesting += 1;
                             current_pos += open + 9; // length of "{% block "
                         } else {
                             nesting -= 1;
                             if nesting == 0 {
                                 // Found matching endblock
                                 let endblock_pos = current_pos + close;
                                 let body_source = &source[start_pos..endblock_pos];
                                 let parser = Parser::new(body_source);
                                 let body = parser.parse()?;
                                 
                                 let total_len = endblock_pos + 14; // length of "{% endblock %}"
                                 return Ok(Some((TemplateNode::Block { name, body }, total_len)));
                             }
                             current_pos += close + 14;
                         }
                     },
                     (None, Some(close)) => {
                         nesting -= 1;
                         if nesting == 0 {
                             let endblock_pos = current_pos + close;
                             let body_source = &source[start_pos..endblock_pos];
                             let parser = Parser::new(body_source);
                             let body = parser.parse()?;
                             let total_len = endblock_pos + 14;
                             return Ok(Some((TemplateNode::Block { name, body }, total_len)));
                         }
                         current_pos += close + 14;
                     },
                     (Some(open), None) => {
                         nesting += 1;
                         current_pos += open + 9;
                     },
                     (None, None) => break,
                 }
            }
        }

        Ok(None)
    }

    fn parse_extends(&self, source: &str) -> Result<Option<(TemplateNode, usize)>> {
        let re_extends = Regex::new(r#"\{%\s*extends\s+"([^"]+)"\s*%\}"#).unwrap();
        
        if let Some(cap) = re_extends.captures(source) {
            let template = cap.get(1).unwrap().as_str().to_string();
            let len = cap.get(0).unwrap().len();
            
            return Ok(Some((TemplateNode::Extends(template), len)));
        }

        Ok(None)
    }

    fn parse_include(&self, source: &str) -> Result<Option<(TemplateNode, usize)>> {
        let re_include = Regex::new(r#"\{%\s*include\s+"([^"]+)"\s*%\}"#).unwrap();
        
        if let Some(cap) = re_include.captures(source) {
            let template = cap.get(1).unwrap().as_str().to_string();
            let len = cap.get(0).unwrap().len();
            
            return Ok(Some((TemplateNode::Include(template), len)));
        }

        Ok(None)
    }

    fn parse_text(&self, source: &str) -> Option<(String, usize)> {
        // Find next template tag
        let pos_var = source.find("{{");
        let pos_tag = source.find("{%");

        let next_tag = match (pos_var, pos_tag) {
            (Some(v), Some(t)) => Some(std::cmp::min(v, t)),
            (Some(v), None) => Some(v),
            (None, Some(t)) => Some(t),
            (None, None) => None,
        };
        
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
