//! Server-Sent Events (SSE) support

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// SSE event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseEvent {
    /// Event ID (optional)
    pub id: Option<String>,
    /// Event type (optional, defaults to "message")
    pub event: Option<String>,
    /// Event data
    pub data: String,
    /// Retry interval in milliseconds (optional)
    pub retry: Option<u64>,
}

impl SseEvent {
    /// Create a new SSE event with data
    pub fn new(data: impl Into<String>) -> Self {
        Self {
            id: None,
            event: None,
            data: data.into(),
            retry: None,
        }
    }

    /// Create an SSE event with JSON data
    pub fn json<T: Serialize>(data: &T) -> Result<Self, serde_json::Error> {
        Ok(Self::new(serde_json::to_string(data)?))
    }

    /// Set the event ID
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the event type
    pub fn event(mut self, event: impl Into<String>) -> Self {
        self.event = Some(event.into());
        self
    }

    /// Set the retry interval
    pub fn retry(mut self, millis: u64) -> Self {
        self.retry = Some(millis);
        self
    }

    /// Format as SSE text
    pub fn to_sse_string(&self) -> String {
        let mut result = String::new();

        if let Some(id) = &self.id {
            result.push_str(&format!("id: {}\n", id));
        }

        if let Some(event) = &self.event {
            result.push_str(&format!("event: {}\n", event));
        }

        if let Some(retry) = self.retry {
            result.push_str(&format!("retry: {}\n", retry));
        }

        // Handle multi-line data
        for line in self.data.lines() {
            result.push_str(&format!("data: {}\n", line));
        }

        result.push('\n'); // End of event
        result
    }
}

/// SSE stream configuration
#[derive(Debug, Clone)]
pub struct SseConfig {
    /// Keep-alive interval
    pub keep_alive: Duration,
    /// Default retry interval for clients
    pub retry: Option<u64>,
}

impl Default for SseConfig {
    fn default() -> Self {
        Self {
            keep_alive: Duration::from_secs(30),
            retry: Some(3000),
        }
    }
}

/// SSE stream wrapper (placeholder for actual stream implementation)
pub struct SseStream {
    config: SseConfig,
}

impl SseStream {
    /// Create a new SSE stream
    pub fn new(config: SseConfig) -> Self {
        Self { config }
    }

    /// Get the config
    pub fn config(&self) -> &SseConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sse_event_format() {
        let event = SseEvent::new("Hello, World!")
            .id("123")
            .event("message")
            .retry(5000);

        let output = event.to_sse_string();
        assert!(output.contains("id: 123\n"));
        assert!(output.contains("event: message\n"));
        assert!(output.contains("retry: 5000\n"));
        assert!(output.contains("data: Hello, World!\n"));
    }

    #[test]
    fn test_sse_multiline_data() {
        let event = SseEvent::new("Line 1\nLine 2\nLine 3");
        let output = event.to_sse_string();
        
        assert!(output.contains("data: Line 1\n"));
        assert!(output.contains("data: Line 2\n"));
        assert!(output.contains("data: Line 3\n"));
    }
}
