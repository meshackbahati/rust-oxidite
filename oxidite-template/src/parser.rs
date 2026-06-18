use crate::Result;
use regex::Regex;
use std::sync::OnceLock;

fn variable_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"\{\{\s*([a-zA-Z0-9_.]+)((?:\s*\|\s*[a-zA-Z0-9_]+)*)\s*\}\}")
            .expect("variable regex must compile")
    })
}

fn if_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\{%\s*if\s+(.+?)\s*%\}").expect("if regex"))
}

fn elif_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\{%\s*elif\s+(.+?)\s*%\}").expect("elif regex"))
}

fn for_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"\{%\s*for\s+([a-zA-Z0-9_]+)\s+in\s+([a-zA-Z0-9_.]+)\s*%\}")
            .expect("for regex")
    })
}

fn block_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\{%\s*block\s+([a-zA-Z0-9_]+)\s*%\}").expect("block regex"))
}

fn extends_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"\{%\s*extends\s+"([^"]+)"\s*%\}"#).expect("extends regex"))
}

fn include_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"\{%\s*include\s+(?:"([^"]+)"|([a-zA-Z0-9_.]+))\s*%\}"#).expect("include regex"))
}

/// Template AST nodes
#[derive(Debug, Clone, PartialEq)]
pub enum TemplateNode {
    Text(String),
    Variable { name: String, filters: Vec<String> },
    If {
        condition: String,
        then_branch: Vec<TemplateNode>,
        elif_branches: Vec<(String, Vec<TemplateNode>)>,
        else_branch: Option<Vec<TemplateNode>>,
    },
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
        if let Some(cap) = variable_regex().captures(source) {
            let full_match = cap.get(0).unwrap();
            let var_name = cap.get(1).unwrap().as_str().to_string();
            let filter = cap
                .get(2)
                .map(|m| {
                    m.as_str()
                        .split('|')
                        .map(str::trim)
                        .filter(|part| !part.is_empty())
                        .map(ToOwned::to_owned)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

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
        if let Some(cap) = if_regex().captures(source) {
            let condition = cap.get(1).unwrap().as_str().to_string();
            let start_pos = cap.get(0).unwrap().end();

            // Find the matching {% endif %}
            let endif_pattern = "{% endif %}";
            
            if let Some(endif_pos) = source[start_pos..].find(endif_pattern) {
                let body_source = &source[start_pos..start_pos + endif_pos];

                // Split body by {% elif %} and {% else %}
                // We need to find all elif/else blocks in order
                let mut segments: Vec<(&str, &str)> = Vec::new(); // (tag_type, body)
                let mut current_pos = 0;
                let mut current_type = "if";

                loop {
                    // Find next elif or else
                    let elif_match = elif_regex().find(&body_source[current_pos..]);
                    let else_match = body_source[current_pos..].find("{% else %}");

                    let next_split = match (elif_match.map(|m| m.start()), else_match) {
                        (Some(e1), Some(e2)) => Some(std::cmp::min(e1, e2)),
                        (Some(e1), None) => Some(e1),
                        (None, Some(e2)) => Some(e2),
                        (None, None) => None,
                    };

                    if let Some(split_pos) = next_split {
                        // Save current segment
                        let segment_body = &body_source[current_pos..current_pos + split_pos];
                        segments.push((current_type, segment_body));

                        // Determine what tag we hit
                        let remaining = &body_source[current_pos + split_pos..];
                        if remaining.starts_with("{% elif ") {
                            if let Some(elif_cap) = elif_regex().captures(remaining) {
                                let elif_condition = elif_cap.get(1).unwrap().as_str().to_string();
                                let tag_len = elif_cap.get(0).unwrap().end();
                                current_pos = current_pos + split_pos + tag_len;
                                // Store the elif condition as the "type" marker
                                // We'll extract it when processing
                                segments.push(("elif_cond", ""));  // placeholder
                                current_type = "elif";
                                // Actually, let's rethink this approach
                                // We'll store elif conditions separately
                                let _ = elif_condition;
                                continue;
                            }
                        } else if remaining.starts_with("{% else %}") {
                            current_pos = current_pos + split_pos + "{% else %}".len();
                            current_type = "else";
                            continue;
                        }
                        break;
                    } else {
                        // No more splits, rest is the final segment
                        let segment_body = &body_source[current_pos..];
                        segments.push((current_type, segment_body));
                        break;
                    }
                }

                // Simpler approach: manually parse if/elif/else/endif
                // Let me restart with a cleaner algorithm
                let (then_branch, elif_branches, else_branch) = self.parse_if_body(body_source)?;

                let node = TemplateNode::If {
                    condition,
                    then_branch,
                    elif_branches,
                    else_branch,
                };

                let total_len = start_pos + endif_pos + endif_pattern.len();
                return Ok(Some((node, total_len)));
            }
        }

        Ok(None)
    }

    /// Parse the body of an if block, splitting into then/elif/else branches
    fn parse_if_body(&self, body: &str) -> Result<(Vec<TemplateNode>, Vec<(String, Vec<TemplateNode>)>, Option<Vec<TemplateNode>>)> {
        // Find all {% elif %} and {% else %} positions
        let mut split_points: Vec<(usize, usize, &str)> = Vec::new(); // (start, end, type)
        
        // Find all elif matches
        for cap in elif_regex().captures_iter(body) {
            let m = cap.get(0).unwrap();
            split_points.push((m.start(), m.end(), "elif"));
        }
        
        // Find else match
        if let Some(else_pos) = body.find("{% else %}") {
            split_points.push((else_pos, else_pos + "{% else %}".len(), "else"));
        }
        
        // Sort by position
        split_points.sort_by_key(|(start, _, _)| *start);

        if split_points.is_empty() {
            // Simple if with no elif or else
            let parser = Parser::new(body);
            return Ok((parser.parse()?, Vec::new(), None));
        }

        // Then branch is everything before the first split point
        let then_source = &body[..split_points[0].0];
        let parser = Parser::new(then_source);
        let then_branch = parser.parse()?;

        let mut elif_branches = Vec::new();
        let mut else_branch = None;

        for (i, (start, end, tag_type)) in split_points.iter().enumerate() {
            let next_start = if i + 1 < split_points.len() {
                split_points[i + 1].0
            } else {
                body.len()
            };

            let branch_body = &body[*end..next_start];
            let parser = Parser::new(branch_body);
            let nodes = parser.parse()?;

            if *tag_type == "elif" {
                // Extract the elif condition from the original source
                if let Some(cap) = elif_regex().captures(&body[*start..]) {
                    let condition = cap.get(1).unwrap().as_str().to_string();
                    elif_branches.push((condition, nodes));
                }
            } else if *tag_type == "else" {
                else_branch = Some(nodes);
            }
        }

        Ok((then_branch, elif_branches, else_branch))
    }

    fn parse_for(&self, source: &str) -> Result<Option<(TemplateNode, usize)>> {
        if let Some(cap) = for_regex().captures(source) {
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
        if let Some(cap) = block_regex().captures(source) {
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
        if let Some(cap) = extends_regex().captures(source) {
            let template = cap.get(1).unwrap().as_str().to_string();
            let len = cap.get(0).unwrap().len();
            
            return Ok(Some((TemplateNode::Extends(template), len)));
        }

        Ok(None)
    }

    fn parse_include(&self, source: &str) -> Result<Option<(TemplateNode, usize)>> {
        if let Some(cap) = include_regex().captures(source) {
            // Group 1 = quoted string literal, Group 2 = variable name
            let template = cap.get(1)
                .or_else(|| cap.get(2))
                .unwrap()
                .as_str()
                .to_string();
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

#[cfg(test)]
mod tests {
    use super::{Parser, TemplateNode};

    #[test]
    fn parse_variable_with_multiple_filters() {
        let nodes = Parser::new("{{ user.name | trim | uppercase }}")
            .parse()
            .expect("parser should succeed");
        assert_eq!(
            nodes,
            vec![TemplateNode::Variable {
                name: "user.name".to_string(),
                filters: vec!["trim".to_string(), "uppercase".to_string()]
            }]
        );
    }
}
