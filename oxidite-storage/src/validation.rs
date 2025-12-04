use crate::{StorageError, Result};
use bytes::Bytes;

/// File validation rules
#[derive(Debug, Clone)]
pub struct ValidationRules {
    pub max_size: Option<u64>,
    pub allowed_mime_types: Option<Vec<String>>,
    pub allowed_extensions: Option<Vec<String>>,
}

impl ValidationRules {
    pub fn new() -> Self {
        Self {
            max_size: None,
            allowed_mime_types: None,
            allowed_extensions: None,
        }
    }

    pub fn max_size(mut self, max_size: u64) -> Self {
        self.max_size = Some(max_size);
        self
    }

    pub fn allowed_mime_types(mut self, types: Vec<String>) -> Self {
        self.allowed_mime_types = Some(types);
        self
    }

    pub fn allowed_extensions(mut self, extensions: Vec<String>) -> Self {
        self.allowed_extensions = Some(extensions);
        self
    }
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self::new()
    }
}

/// File validator
pub struct FileValidator {
    rules: ValidationRules,
}

impl FileValidator {
    pub fn new(rules: ValidationRules) -> Self {
        Self { rules }
    }

    pub fn validate(&self, filename: &str, data: &Bytes) -> Result<()> {
        // Validate file size
        if let Some(max_size) = self.rules.max_size {
            if data.len() as u64 > max_size {
                return Err(StorageError::Validation(
                    format!("File size {} exceeds maximum {}", data.len(), max_size)
                ));
            }
        }

        // Validate extension
        if let Some(allowed_extensions) = &self.rules.allowed_extensions {
            let extension = std::path::Path::new(filename)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");

            if !allowed_extensions.contains(&extension.to_lowercase()) {
                return Err(StorageError::Validation(
                    format!("File extension '{}' not allowed", extension)
                ));
            }
        }

        // Validate MIME type
        if let Some(allowed_mime_types) = &self.rules.allowed_mime_types {
            let mime_type = mime_guess::from_path(filename)
                .first_or_octet_stream()
                .to_string();

            if !allowed_mime_types.iter().any(|m| mime_type.starts_with(m)) {
                return Err(StorageError::Validation(
                    format!("MIME type '{}' not allowed", mime_type)
                ));
            }
        }

        Ok(())
    }
}

/// Generate secure random filename
pub fn generate_filename(original: &str) -> String {
    let extension = std::path::Path::new(original)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let uuid = uuid::Uuid::new_v4();
    
    if extension.is_empty() {
        uuid.to_string()
    } else {
        format!("{}.{}", uuid, extension)
    }
}
