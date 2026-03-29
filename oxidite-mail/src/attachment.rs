use std::path::PathBuf;

/// Email attachment
#[derive(Debug, Clone)]
pub struct Attachment {
    pub(crate) filename: String,
    pub(crate) content: Vec<u8>,
    pub(crate) content_type: Option<String>,
    pub(crate) inline: bool,
    pub(crate) content_id: Option<String>,
}

impl Attachment {
    pub fn new(filename: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
            content: Vec::new(),
            content_type: None,
            inline: false,
            content_id: None,
        }
    }

    /// Create attachment from file
    pub fn from_file(path: impl Into<PathBuf>) -> std::io::Result<Self> {
        let path = path.into();
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("attachment")
            .to_string();

        let content = std::fs::read(&path)?;
        let content_type = mime_guess::from_path(&path).first().map(|m| m.to_string());

        Ok(Self {
            filename,
            content,
            content_type,
            inline: false,
            content_id: None,
        })
    }

    /// Set attachment content
    pub fn content(mut self, content: Vec<u8>) -> Self {
        self.content = content;
        self
    }

    /// Set content type
    pub fn content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }

    /// Mark as inline attachment
    pub fn inline(mut self) -> Self {
        self.inline = true;
        self
    }

    /// Mark as inline and set content ID used by HTML emails (`cid:<id>`).
    pub fn inline_with_cid(mut self, content_id: impl Into<String>) -> Self {
        self.inline = true;
        self.content_id = Some(content_id.into());
        self
    }

    /// Access attachment filename.
    pub fn filename(&self) -> &str {
        &self.filename
    }

    /// Access attachment content type.
    pub fn content_type_ref(&self) -> Option<&str> {
        self.content_type.as_deref()
    }

    /// Access content ID if present.
    pub fn content_id(&self) -> Option<&str> {
        self.content_id.as_deref()
    }

    /// Check whether attachment is inline.
    pub fn is_inline(&self) -> bool {
        self.inline
    }
}

#[cfg(test)]
mod tests {
    use super::Attachment;

    #[test]
    fn inline_with_cid_sets_expected_fields() {
        let attachment = Attachment::new("logo.png").inline_with_cid("logo-1");
        assert!(attachment.is_inline());
        assert_eq!(attachment.content_id(), Some("logo-1"));
        assert_eq!(attachment.filename(), "logo.png");
    }
}
