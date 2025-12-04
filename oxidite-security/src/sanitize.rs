//! HTML sanitization utilities

/// Escape HTML special characters (prevents XSS)
pub fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Sanitize HTML by removing dangerous tags and attributes
/// This is a basic implementation - for production use consider
/// a dedicated library like ammonia
pub fn sanitize_html(s: &str) -> String {
    // Remove script tags
    let mut result = remove_tag(s, "script");
    result = remove_tag(&result, "style");
    result = remove_tag(&result, "iframe");
    result = remove_tag(&result, "object");
    result = remove_tag(&result, "embed");
    result = remove_tag(&result, "form");
    
    // Remove event handlers (onclick, onload, etc.)
    result = remove_event_handlers(&result);
    
    // Remove javascript: URLs
    result = remove_javascript_urls(&result);
    
    result
}

fn remove_tag(s: &str, tag: &str) -> String {
    let lower = s.to_lowercase();
    let mut result = String::new();
    let mut i = 0;
    let bytes = s.as_bytes();
    
    while i < bytes.len() {
        let remaining = &lower[i..];
        
        // Look for opening tag
        if remaining.starts_with(&format!("<{}", tag)) {
            // Find closing tag
            if let Some(end) = remaining.find(&format!("</{}>", tag)) {
                i += end + tag.len() + 3;
                continue;
            } else if let Some(end) = remaining.find('>') {
                // Self-closing or unclosed
                i += end + 1;
                continue;
            }
        }
        
        result.push(bytes[i] as char);
        i += 1;
    }
    
    result
}

fn remove_event_handlers(s: &str) -> String {
    let event_handlers = [
        "onclick", "onload", "onerror", "onmouseover", "onmouseout",
        "onfocus", "onblur", "onsubmit", "onchange", "onkeyup",
        "onkeydown", "onkeypress",
    ];
    
    let mut result = s.to_string();
    for handler in event_handlers {
        // Remove handler="..."
        while let Some(start) = result.to_lowercase().find(handler) {
            if let Some(eq_pos) = result[start..].find('=') {
                let quote_start = start + eq_pos + 1;
                if quote_start < result.len() {
                    let quote = result.chars().nth(quote_start);
                    if quote == Some('"') || quote == Some('\'') {
                        if let Some(quote_end) = result[quote_start + 1..].find(quote.unwrap()) {
                            result = format!(
                                "{}{}",
                                &result[..start],
                                &result[quote_start + quote_end + 2..]
                            );
                            continue;
                        }
                    }
                }
            }
            break;
        }
    }
    
    result
}

fn remove_javascript_urls(s: &str) -> String {
    s.replace("javascript:", "")
        .replace("data:", "")
        .replace("vbscript:", "")
}

/// Strip all HTML tags
pub fn strip_tags(s: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_html("\"hello\""), "&quot;hello&quot;");
        assert_eq!(escape_html("a & b"), "a &amp; b");
    }

    #[test]
    fn test_strip_tags() {
        assert_eq!(strip_tags("<p>Hello</p>"), "Hello");
        assert_eq!(strip_tags("<b>Bold</b> text"), "Bold text");
    }

    #[test]
    fn test_sanitize_html() {
        let input = "<p>Hello</p><script>alert('xss')</script>";
        let sanitized = sanitize_html(input);
        assert!(!sanitized.contains("script"));
    }
}
