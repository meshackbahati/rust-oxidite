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
        self.allowed_mime_types = Some(
            types
                .into_iter()
                .map(|t| t.trim().to_ascii_lowercase())
                .collect(),
        );
        self
    }

    pub fn allowed_extensions(mut self, extensions: Vec<String>) -> Self {
        self.allowed_extensions = Some(
            extensions
                .into_iter()
                .map(|e| e.trim().trim_start_matches('.').to_ascii_lowercase())
                .collect(),
        );
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
            let extension = extension.to_ascii_lowercase();

            if !allowed_extensions.iter().any(|allowed| allowed == &extension) {
                return Err(StorageError::Validation(
                    format!("File extension '{}' not allowed", extension)
                ));
            }
        }

        // Validate MIME type
        if let Some(allowed_mime_types) = &self.rules.allowed_mime_types {
            let mime_type = mime_guess::from_path(filename)
                .first_or_octet_stream()
                .to_string()
                .to_ascii_lowercase();

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
        format!("{}.{}", uuid, extension.to_ascii_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::{generate_filename, FileValidator, ValidationRules};
    use bytes::Bytes;

    #[test]
    fn validator_accepts_case_insensitive_extension_and_mime() {
        let rules = ValidationRules::new()
            .allowed_extensions(vec!["JPG".to_string()])
            .allowed_mime_types(vec!["IMAGE/".to_string()]);
        let validator = FileValidator::new(rules);
        let data = Bytes::from_static(b"fake");
        assert!(validator.validate("photo.JPG", &data).is_ok());
    }

    #[test]
    fn generate_filename_preserves_extension_lowercased() {
        let name = generate_filename("avatar.PNG");
        assert!(name.ends_with(".png"));
    }
}
