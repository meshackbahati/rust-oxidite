use std::path::PathBuf;

/// Email attachment
#[derive(Debug, Clone)]
pub struct Attachment {
    pub(crate) filename: String,
    pub(crate) content: Vec<u8>,
    pub(crate) content_type: Option<String>,
    pub(crate) inline: bool,
}

impl Attachment {
    pub fn new(filename: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
            content: Vec::new(),
            content_type: None,
            inline: false,
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
        let content_type = mime_guess::from_path(&path)
            .first()
            .map(|m| m.to_string());

        Ok(Self {
            filename,
            content,
            content_type,
            inline: false,
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
}
