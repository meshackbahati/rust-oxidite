use std::fs;
use std::path::Path;
use std::io;

pub fn make_model(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    validate_rust_type_name(name)?;
    let models_dir = Path::new("src/models");
    if !models_dir.exists() {
        fs::create_dir_all(models_dir)?;
    }

    let file_stem = to_snake_case(name);
    let filename = format!("src/models/{}.rs", file_stem);
    if Path::new(&filename).exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("model file already exists: {filename}"),
        )
        .into());
    }

    let table_name = pluralize_table_name(&file_stem);

    let model_template = format!(
        r#"use serde::{{Deserialize, Serialize}};
use oxidite_db::{{Model, sqlx}};

#[derive(Debug, Clone, Serialize, Deserialize, Model, sqlx::FromRow)]
#[model(table = "{}")]
pub struct {} {{
    pub id: i64,
    // Add your fields here, e.g.:
    // pub name: String,
    // pub created_at: i64,
    // pub updated_at: i64,
    // pub deleted_at: Option<i64>,
}}
"#,
        table_name,
        name
    );

    fs::write(&filename, model_template)?;

    println!("✅ Model created: {}", filename);
    Ok(())
}

pub fn make_controller(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    validate_rust_type_name(name)?;
    let controllers_dir = Path::new("src/controllers");
    if !controllers_dir.exists() {
        fs::create_dir_all(controllers_dir)?;
    }

    let controller_template = format!(
        r#"use oxidite::prelude::*;

pub struct {};

impl {} {{
    pub async fn index(_req: Request) -> Result<Response> {{
        Ok(Response::json(serde_json::json!({{
            "message": "List endpoint"
        }})))
    }}

    pub async fn show(_req: Request) -> Result<Response> {{
        Ok(Response::json(serde_json::json!({{
            "message": "Show endpoint"
        }})))
    }}

    pub async fn create(_req: Request) -> Result<Response> {{
        Ok(Response::json(serde_json::json!({{
            "message": "Create endpoint"
        }})))
    }}

    pub async fn update(_req: Request) -> Result<Response> {{
        Ok(Response::json(serde_json::json!({{
            "message": "Update endpoint"
        }})))
    }}

    pub async fn destroy(_req: Request) -> Result<Response> {{
        Ok(Response::json(serde_json::json!({{
            "message": "Destroy endpoint"
        }})))
    }}
}}
"#,
        name, name
    );

    let filename = format!("src/controllers/{}.rs", to_snake_case(name));
    fs::write(&filename, controller_template)?;

    println!("✅ Controller created: {}", filename);
    Ok(())
}

pub fn make_middleware(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    validate_rust_type_name(name)?;
    let middleware_dir = Path::new("src/middleware");
    if !middleware_dir.exists() {
        fs::create_dir_all(middleware_dir)?;
    }

    let middleware_template = format!(
        r#"use oxidite_core::{{Error, OxiditeRequest, OxiditeResponse}};
use std::future::Future;
use std::pin::Pin;
use std::task::{{Context, Poll}};
use tower::{{Layer, Service}};

#[derive(Clone)]
pub struct {}<S> {{
    inner: S,
}}

impl<S> {}<S> {{
    pub fn new(inner: S) -> Self {{
        Self {{ inner }}
    }}
}}

impl<S> Service<OxiditeRequest> for {}<S>
where
    S: Service<OxiditeRequest, Response = OxiditeResponse, Error = Error> + Clone + Send + 'static,
    S::Future: Send + 'static,
{{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {{
        self.inner.poll_ready(cx)
    }}

    fn call(&mut self, req: OxiditeRequest) -> Self::Future {{
        let mut inner = self.inner.clone();

        Box::pin(async move {{
            // Pre-processing
            println!("Before request");

            let response = inner.call(req).await?;

            // Post-processing
            println!("After request");

            Ok(response)
        }})
    }}
}}

#[derive(Clone)]
pub struct {}Layer;

impl {}Layer {{
    pub fn new() -> Self {{
        Self
    }}
}}

impl<S> Layer<S> for {}Layer {{
    type Service = {}<S>;

    fn layer(&self, inner: S) -> Self::Service {{
        {}::<S>::new(inner)
    }}
}}
"#,
        name, name, name, name, name, name, name, name
    );

    let filename = format!("src/middleware/{}.rs", to_snake_case(name));
    fs::write(&filename, middleware_template)?;

    println!("✅ Middleware created: {}", filename);
    Ok(())
}

pub fn make_service(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    validate_rust_type_name(name)?;
    let services_dir = Path::new("src/services");
    if !services_dir.exists() {
        fs::create_dir_all(services_dir)?;
    }

    let service_template = format!(
        r#"use serde::{{Deserialize, Serialize}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {}Input {{
    // Add input fields
}}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {}Output {{
    // Add output fields
}}

#[derive(Clone)]
pub struct {}Service;

impl {}Service {{
    pub fn new() -> Self {{
        Self
    }}

    pub async fn execute(
        &self,
        _input: {}Input,
    ) -> Result<{}Output, Box<dyn std::error::Error>> {{
        Err("Not implemented".into())
    }}
}}
"#,
        name, name, name, name, name, name
    );

    let filename = format!("src/services/{}.rs", to_snake_case(name));
    fs::write(&filename, service_template)?;

    println!("✅ Service created: {}", filename);
    Ok(())
}

pub fn make_validator(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    validate_rust_type_name(name)?;
    let validators_dir = Path::new("src/validators");
    if !validators_dir.exists() {
        fs::create_dir_all(validators_dir)?;
    }

    let validator_template = format!(
        r#"use serde_json::Value;
use std::collections::HashMap;

pub struct {}Validator;

impl {}Validator {{
    pub fn validate(data: &Value) -> Result<(), ValidationError> {{
        let mut errors = HashMap::new();

        if data.is_null() {{
            errors.insert("body".to_string(), "Body must not be null".to_string());
        }}

        if !errors.is_empty() {{
            return Err(ValidationError::new(errors));
        }}

        Ok(())
    }}
}}

#[derive(Debug)]
pub struct ValidationError {{
    pub errors: HashMap<String, String>,
}}

impl ValidationError {{
    pub fn new(errors: HashMap<String, String>) -> Self {{
        Self {{ errors }}
    }}
}}

impl std::fmt::Display for ValidationError {{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{
        write!(f, "Validation failed: {{:?}}", self.errors)
    }}
}}

impl std::error::Error for ValidationError {{}}
"#,
        name, name
    );

    let filename = format!("src/validators/{}.rs", to_snake_case(name));
    fs::write(&filename, validator_template)?;

    println!("✅ Validator created: {}", filename);
    Ok(())
}

pub fn make_job(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    validate_rust_type_name(name)?;
    let jobs_dir = Path::new("src/jobs");
    if !jobs_dir.exists() {
        fs::create_dir_all(jobs_dir)?;
    }

    let filename = format!("src/jobs/{}.rs", to_snake_case(name));
    let template = format!(
        r#"use serde::{{Deserialize, Serialize}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {}Job {{
    pub id: String,
    // Add job payload fields
}}

impl {}Job {{
    pub async fn handle(&self) -> Result<(), Box<dyn std::error::Error>> {{
        // Add background processing logic
        Ok(())
    }}
}}
"#,
        name, name
    );

    fs::write(&filename, template)?;
    println!("✅ Job created: {}", filename);
    Ok(())
}

pub fn make_policy(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    validate_rust_type_name(name)?;
    let policies_dir = Path::new("src/policies");
    if !policies_dir.exists() {
        fs::create_dir_all(policies_dir)?;
    }

    let filename = format!("src/policies/{}.rs", to_snake_case(name));
    let template = format!(
        r#"pub struct {}Policy;

impl {}Policy {{
    pub fn can_view(_user_id: i64, _resource_owner_id: i64) -> bool {{
        // Add authorization logic
        true
    }}

    pub fn can_update(_user_id: i64, _resource_owner_id: i64) -> bool {{
        // Add authorization logic
        true
    }}
}}
"#,
        name, name
    );

    fs::write(&filename, template)?;
    println!("✅ Policy created: {}", filename);
    Ok(())
}

pub fn make_event(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    validate_rust_type_name(name)?;
    let events_dir = Path::new("src/events");
    if !events_dir.exists() {
        fs::create_dir_all(events_dir)?;
    }

    let filename = format!("src/events/{}.rs", to_snake_case(name));
    let template = format!(
        r#"use serde::{{Deserialize, Serialize}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {}Event {{
    pub occurred_at: i64,
    // Add event fields
}}
"#,
        name
    );

    fs::write(&filename, template)?;
    println!("✅ Event created: {}", filename);
    Ok(())
}

fn validate_rust_type_name(name: &str) -> Result<(), io::Error> {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "name cannot be empty",
        ));
    };

    if !(first.is_ascii_alphabetic() || first == '_') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "name must start with a letter or underscore",
        ));
    }

    if !chars.all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "name must contain only letters, numbers, and underscores",
        ));
    }

    Ok(())
}

fn to_snake_case(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 4);
    for (i, ch) in input.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch.to_ascii_lowercase());
        }
    }
    out
}

fn pluralize_table_name(base: &str) -> String {
    if base.ends_with('s') {
        base.to_string()
    } else {
        format!("{base}s")
    }
}

#[cfg(test)]
mod tests {
    use super::{pluralize_table_name, to_snake_case, validate_rust_type_name};

    #[test]
    fn snake_case_conversion() {
        assert_eq!(to_snake_case("UserProfile"), "user_profile");
        assert_eq!(to_snake_case("user"), "user");
    }

    #[test]
    fn pluralize_table_names() {
        assert_eq!(pluralize_table_name("users"), "users");
        assert_eq!(pluralize_table_name("post"), "posts");
    }

    #[test]
    fn validates_type_names() {
        assert!(validate_rust_type_name("User").is_ok());
        assert!(validate_rust_type_name("_User1").is_ok());
        assert!(validate_rust_type_name("1User").is_err());
        assert!(validate_rust_type_name("user-name").is_err());
    }
}
