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

impl PathItem {
    pub fn with_get(mut self, operation: Operation) -> Self {
        self.get = Some(operation);
        self
    }

    pub fn with_post(mut self, operation: Operation) -> Self {
        self.post = Some(operation);
        self
    }

    pub fn with_put(mut self, operation: Operation) -> Self {
        self.put = Some(operation);
        self
    }

    pub fn with_delete(mut self, operation: Operation) -> Self {
        self.delete = Some(operation);
        self
    }
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

impl Operation {
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn add_tag(mut self, tag: impl Into<String>) -> Self {
        let tags = self.tags.get_or_insert_with(Vec::new);
        tags.push(tag.into());
        self
    }

    pub fn add_parameter(mut self, parameter: Parameter) -> Self {
        let parameters = self.parameters.get_or_insert_with(Vec::new);
        parameters.push(parameter);
        self
    }

    pub fn with_request_body(mut self, request_body: RequestBody) -> Self {
        self.request_body = Some(request_body);
        self
    }

    pub fn add_response(mut self, status_code: impl Into<String>, response: Response) -> Self {
        self.responses.insert(status_code.into(), response);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterLocation {
    Query,
    Path,
    Header,
    Cookie,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub location: String, // "query", "path", "header", "cookie"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    pub schema: Schema,
}

impl Parameter {
    pub fn new(
        name: impl Into<String>,
        location: ParameterLocation,
        schema: Schema,
    ) -> Self {
        let location = match location {
            ParameterLocation::Query => "query",
            ParameterLocation::Path => "path",
            ParameterLocation::Header => "header",
            ParameterLocation::Cookie => "cookie",
        }
        .to_string();

        Self {
            name: name.into(),
            location,
            description: None,
            required: None,
            schema,
        }
    }

    pub fn query(name: impl Into<String>, schema: Schema) -> Self {
        Self::new(name, ParameterLocation::Query, schema)
    }

    pub fn path(name: impl Into<String>, schema: Schema) -> Self {
        let mut p = Self::new(name, ParameterLocation::Path, schema);
        p.required = Some(true);
        p
    }

    pub fn header(name: impl Into<String>, schema: Schema) -> Self {
        Self::new(name, ParameterLocation::Header, schema)
    }

    pub fn cookie(name: impl Into<String>, schema: Schema) -> Self {
        Self::new(name, ParameterLocation::Cookie, schema)
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = Some(required);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub required: bool,
    pub content: HashMap<String, MediaType>,
}

impl RequestBody {
    pub fn json(schema: Schema) -> Self {
        let mut content = HashMap::new();
        content.insert("application/json".to_string(), MediaType { schema });
        Self {
            description: None,
            required: true,
            content,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<HashMap<String, MediaType>>,
}

impl Response {
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            content: None,
        }
    }

    pub fn json(description: impl Into<String>, schema: Schema) -> Self {
        let mut content = HashMap::new();
        content.insert("application/json".to_string(), MediaType { schema });
        Self {
            description: description.into(),
            content: Some(content),
        }
    }
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

impl Schema {
    pub fn string() -> Self {
        Self::Simple {
            type_name: "string".to_string(),
        }
    }

    pub fn integer() -> Self {
        Self::Simple {
            type_name: "integer".to_string(),
        }
    }

    pub fn number() -> Self {
        Self::Simple {
            type_name: "number".to_string(),
        }
    }

    pub fn boolean() -> Self {
        Self::Simple {
            type_name: "boolean".to_string(),
        }
    }

    pub fn object(properties: HashMap<String, Schema>) -> Self {
        Self::Object {
            type_name: "object".to_string(),
            properties: properties
                .into_iter()
                .map(|(k, v)| (k, Box::new(v)))
                .collect(),
        }
    }

    pub fn array(items: Schema) -> Self {
        Self::Array {
            type_name: "array".to_string(),
            items: Box::new(items),
        }
    }
}

/// Lightweight schema inference trait for common Rust types.
pub trait ToSchema {
    fn schema() -> Schema;
}

impl ToSchema for String {
    fn schema() -> Schema {
        Schema::string()
    }
}
impl ToSchema for bool {
    fn schema() -> Schema {
        Schema::boolean()
    }
}
impl ToSchema for i32 {
    fn schema() -> Schema {
        Schema::integer()
    }
}
impl ToSchema for i64 {
    fn schema() -> Schema {
        Schema::integer()
    }
}
impl ToSchema for u32 {
    fn schema() -> Schema {
        Schema::integer()
    }
}
impl ToSchema for u64 {
    fn schema() -> Schema {
        Schema::integer()
    }
}
impl ToSchema for f32 {
    fn schema() -> Schema {
        Schema::number()
    }
}
impl ToSchema for f64 {
    fn schema() -> Schema {
        Schema::number()
    }
}
impl<T: ToSchema> ToSchema for Vec<T> {
    fn schema() -> Schema {
        Schema::array(T::schema())
    }
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

impl AutoDocs for oxidite_core::Router {
    fn with_auto_docs(mut self, spec: OpenApiSpec) -> Self {
        let spec_arc = std::sync::Arc::new(spec);

        let spec_json = spec_arc.clone();
        self.get("/openapi.json", move || {
            let spec_json = spec_json.clone();
            async move { Ok(oxidite_core::OxiditeResponse::json((*spec_json).clone())) }
        });

        let spec_docs = spec_arc.clone();
        self.get("/api/docs", move || {
            let spec_docs = spec_docs.clone();
            async move {
                Ok(oxidite_core::OxiditeResponse::html(generate_docs_html(
                    &spec_docs,
                )))
            }
        });

        self
    }
}

/// Generate HTML documentation page
pub fn generate_docs_html(spec: &OpenApiSpec) -> String {
    let spec_json = serde_json::to_string_pretty(spec).unwrap_or_else(|_| "{}".to_string());
    let safe_title = html_escape(&spec.info.title);
    let safe_spec_json = spec_json.replace("</script>", "<\\/script>");
    
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
</html>"#, safe_title, safe_spec_json)
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
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

    #[test]
    fn test_operation_builder_helpers() {
        let operation = get_operation("Get users")
            .with_description("Return users")
            .add_tag("users")
            .add_parameter(Parameter::query("page", Schema::integer()).required(false))
            .add_response("200", Response::json("ok", Schema::array(Schema::string())));

        assert_eq!(operation.summary.as_deref(), Some("Get users"));
        assert_eq!(operation.tags.as_ref().map(Vec::len), Some(1));
        assert!(operation.responses.contains_key("200"));
    }

    #[test]
    fn test_generate_docs_html_escapes_title() {
        let spec = OpenApiBuilder::new("<script>x</script>", "1.0.0").build();
        let html = generate_docs_html(&spec);
        assert!(html.contains("&lt;script&gt;x&lt;/script&gt;"));
        assert!(!html.contains("<title><script>x</script>"));
    }

    #[test]
    fn to_schema_infers_basic_types() {
        let string_schema = <String as ToSchema>::schema();
        let vec_schema = <Vec<i32> as ToSchema>::schema();
        assert!(matches!(string_schema, Schema::Simple { .. }));
        assert!(matches!(vec_schema, Schema::Array { .. }));
    }
}
