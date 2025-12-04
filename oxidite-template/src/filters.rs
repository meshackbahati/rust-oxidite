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
        filters.insert("capitalize".to_string(), capitalize as fn(&str) -> String);
        filters.insert("trim".to_string(), trim as fn(&str) -> String);
        filters.insert("length".to_string(), length as fn(&str) -> String);
        filters.insert("reverse".to_string(), reverse as fn(&str) -> String);

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
