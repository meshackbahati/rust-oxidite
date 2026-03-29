//! HTML sanitization utilities

/// Escape HTML special characters (prevents XSS)
#[must_use]
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
#[must_use]
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
    let mut result = String::new();
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);

    let mut cursor = 0usize;
    while let Some(start) = find_ascii_case_insensitive(s, &open, cursor) {
        result.push_str(&s[cursor..start]);

        if let Some(end) = find_ascii_case_insensitive(s, &close, start) {
            cursor = end + close.len();
        } else if let Some(gt) = s[start..].find('>') {
            cursor = start + gt + 1;
        } else {
            cursor = s.len();
        }
    }
    result.push_str(&s[cursor..]);
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
        while let Some(start) = find_ascii_case_insensitive(&result, handler, 0) {
            let Some(eq_pos) = result[start..].find('=') else {
                break;
            };
            let quote_start = start + eq_pos + 1;
            let quote = result.as_bytes().get(quote_start).copied();
            let Some(quote) = quote else { break };
            if quote != b'"' && quote != b'\'' {
                break;
            }
            let next = result[quote_start + 1..]
                .find(quote as char)
                .map(|idx| quote_start + 2 + idx);
            let Some(end) = next else { break };

            result.replace_range(start..end, "");
        }
    }
    
    result
}

fn remove_javascript_urls(s: &str) -> String {
    let without_js = replace_ascii_case_insensitive(s, "javascript:", "");
    replace_ascii_case_insensitive(&without_js, "vbscript:", "")
}

fn find_ascii_case_insensitive(haystack: &str, needle: &str, from: usize) -> Option<usize> {
    if needle.is_empty() || from >= haystack.len() {
        return None;
    }
    let h = haystack.as_bytes();
    let n = needle.as_bytes();
    if n.len() > h.len().saturating_sub(from) {
        return None;
    }

    for i in from..=h.len() - n.len() {
        if h[i..i + n.len()]
            .iter()
            .zip(n.iter())
            .all(|(a, b)| a.eq_ignore_ascii_case(b))
        {
            return Some(i);
        }
    }
    None
}

fn replace_ascii_case_insensitive(input: &str, needle: &str, replacement: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut cursor = 0usize;
    while let Some(idx) = find_ascii_case_insensitive(input, needle, cursor) {
        out.push_str(&input[cursor..idx]);
        out.push_str(replacement);
        cursor = idx + needle.len();
    }
    out.push_str(&input[cursor..]);
    out
}

/// Strip all HTML tags
#[must_use]
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

    #[test]
    fn sanitize_handles_unicode_without_corruption() {
        let input = "Привет<script>alert(1)</script>世界";
        let sanitized = sanitize_html(input);
        assert_eq!(sanitized, "Привет世界");
    }

    #[test]
    fn sanitize_removes_case_insensitive_js_scheme() {
        let input = r#"<a href="JaVaScRiPt:alert(1)">x</a>"#;
        let sanitized = sanitize_html(input);
        assert!(!sanitized.to_lowercase().contains("javascript:"));
    }
}
