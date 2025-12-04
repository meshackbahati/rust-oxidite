use crate::{TemplateError, Result};
use std::collections::HashMap;

/// Built-in template filters
pub struct Filters {
    filters: HashMap<String, fn(&str) -> String>,
}

impl Filters {
    pub fn new() -> Self {
        let mut filters = HashMap::new();
        
        // Register built-in filters
        filters.insert("uppercase".to_string(), uppercase as fn(&str) -> String);
        filters.insert("lowercase".to_string(), lowercase as fn(&str) -> String);
        filters.insert("upper".to_string(), uppercase as fn(&str) -> String); // Alias
        filters.insert("lower".to_string(), lowercase as fn(&str) -> String); // Alias
        filters.insert("capitalize".to_string(), capitalize as fn(&str) -> String);
        filters.insert("trim".to_string(), trim as fn(&str) -> String);
        filters.insert("length".to_string(), length as fn(&str) -> String);
        filters.insert("reverse".to_string(), reverse as fn(&str) -> String);
        filters.insert("truncate".to_string(), truncate_default as fn(&str) -> String);
        filters.insert("slugify".to_string(), slugify as fn(&str) -> String);
        filters.insert("title".to_string(), title_case as fn(&str) -> String);
        filters.insert("default".to_string(), default_value as fn(&str) -> String);

        Self { filters }
    }

    pub fn apply(&self, name: &str, input: &str) -> Result<String> {
        if let Some(filter_fn) = self.filters.get(name) {
            Ok(filter_fn(input))
        } else {
            Err(TemplateError::FilterNotFound(name.to_string()))
        }
    }

    pub fn register(&mut self, name: String, filter: fn(&str) -> String) {
        self.filters.insert(name, filter);
    }
}

impl Default for Filters {
    fn default() -> Self {
        Self::new()
    }
}

// Built-in filter functions

fn uppercase(s: &str) -> String {
    s.to_uppercase()
}

fn lowercase(s: &str) -> String {
    s.to_lowercase()
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

fn trim(s: &str) -> String {
    s.trim().to_string()
}

fn length(s: &str) -> String {
    s.len().to_string()
}

fn reverse(s: &str) -> String {
    s.chars().rev().collect()
}

fn truncate_default(s: &str) -> String {
    // Default truncate to 100 chars
    if s.len() > 100 {
        format!("{}...", &s[..97])
    } else {
        s.to_string()
    }
}

fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                '\0' // Will be filtered out
            }
        })
        .filter(|&c| c != '\0')
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-")
}

fn title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

fn default_value(s: &str) -> String {
    if s.trim().is_empty() {
        "N/A".to_string()
    } else {
        s.to_string()
    }
}
