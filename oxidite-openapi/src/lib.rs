use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OpenAPI 3.0 Specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiSpec {
    pub openapi: String,
    pub info: Info,
    pub paths: HashMap<String, PathItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Components>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<Vec<Server>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
    pub title: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PathItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub put: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<Operation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Operation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<Parameter>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_body: Option<RequestBody>,
    pub responses: HashMap<String, Response>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub location: String, // "query", "path", "header"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    pub schema: Schema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub required: bool,
    pub content: HashMap<String, MediaType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<HashMap<String, MediaType>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaType {
    pub schema: Schema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Schema {
    Simple {
        #[serde(rename = "type")]
        type_name: String,
    },
    Object {
        #[serde(rename = "type")]
        type_name: String,
        properties: HashMap<String, Box<Schema>>,
    },
    Array {
        #[serde(rename = "type")]
        type_name: String,
        items: Box<Schema>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Components {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schemas: Option<HashMap<String, Schema>>,
}

/// OpenAPI Documentation Builder
pub struct OpenApiBuilder {
    spec: OpenApiSpec,
}

impl OpenApiBuilder {
    pub fn new(title: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            spec: OpenApiSpec {
                openapi: "3.0.0".to_string(),
                info: Info {
                    title: title.into(),
                    version: version.into(),
                    description: None,
                },
                paths: HashMap::new(),
                components: None,
                servers: None,
            },
        }
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.spec.info.description = Some(desc.into());
        self
    }

    pub fn server(mut self, url: impl Into<String>, description: Option<String>) -> Self {
        let servers = self.spec.servers.get_or_insert_with(Vec::new);
        servers.push(Server {
            url: url.into(),
            description,
        });
        self
    }

    pub fn path(mut self, path: impl Into<String>, item: PathItem) -> Self {
        self.spec.paths.insert(path.into(), item);
        self
    }

    pub fn build(self) -> OpenApiSpec {
        self.spec
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.spec)
    }
}

/// Helper to create a GET operation
pub fn get_operation(summary: impl Into<String>) -> Operation {
    Operation {
        summary: Some(summary.into()),
        description: None,
        tags: None,
        parameters: None,
        request_body: None,
        responses: HashMap::new(),
    }
}

/// Helper to create a POST operation
pub fn post_operation(summary: impl Into<String>) -> Operation {
    Operation {
        summary: Some(summary.into()),
        description: None,
        tags: None,
        parameters: None,
        request_body: None,
        responses: HashMap::new(),
    }
}

/// Auto-documentation trait for routers
pub trait AutoDocs {
    /// Register the /api/docs endpoint with OpenAPI documentation
    fn with_auto_docs(self, spec: OpenApiSpec) -> Self;
}

/// Generate HTML documentation page
pub fn generate_docs_html(spec: &OpenApiSpec) -> String {
    let spec_json = serde_json::to_string_pretty(spec).unwrap_or_else(|_| "{}".to_string());
    
    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} - API Documentation</title>
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui.css">
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
    <script>
        const spec = {};
        SwaggerUIBundle({{
            spec: spec,
            dom_id: '#swagger-ui',
            deepLinking: true,
            presets: [
                SwaggerUIBundle.presets.apis,
                SwaggerUIBundle.SwaggerUIStandalonePreset
            ],
        }});
    </script>
</body>
</html>"#, spec.info.title, spec_json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openapi_builder() {
        let spec = OpenApiBuilder::new("Test API", "1.0.0")
            .description("A test API")
            .server("http://localhost:8080", Some("Local server".to_string()))
            .build();

        assert_eq!(spec.info.title, "Test API");
        assert_eq!(spec.info.version, "1.0.0");
    }
}
