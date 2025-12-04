use std::fs;
use std::path::Path;

pub fn make_model(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create models directory if it doesn't exist
    let models_dir = Path::new("src/models");
    if !models_dir.exists() {
        fs::create_dir_all(models_dir)?;
    }
    
    let model_template = format!(r#"use serde::{{Deserialize, Serialize}};
use oxidite_db::{{Model, Connection, Result}};

#[derive(Debug, Serialize, Deserialize)]
pub struct {} {{
    pub id: i64,
    // Add your fields here
}}

#[async_trait::async_trait]
impl Model for {} {{
    fn table_name() -> &'static str {{
        "{}"
    }}
    
    async fn find(id: i64, conn: &dyn Connection) -> Result<Option<Self>> {{
        // TODO: Implement
        todo!()
    }}
    
    async fn create(self, conn: &dyn Connection) -> Result<Self> {{
        // TODO: Implement
        todo!()
    }}
    
    async fn update(&self, conn: &dyn Connection) -> Result<()> {{
        // TODO: Implement
        todo!()
    }}
    
    async fn delete(&self, conn: &dyn Connection) -> Result<()> {{
        // TODO: Implement
        todo!()
    }}
}}
"#, name, name, name.to_lowercase());
    
    let filename = format!("src/models/{}.rs", name.to_lowercase());
    fs::write(&filename, model_template)?;
    
    println!("✅ Model created: {}", filename);
    Ok(())
}

pub fn make_controller(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create controllers directory if it doesn't exist
    let controllers_dir = Path::new("src/controllers");
    if !controllers_dir.exists() {
        fs::create_dir_all(controllers_dir)?;
    }
    
    let controller_template = format!(r#"use oxidite_core::{{OxiditeRequest, OxiditeResponse, Result, Json}};
use http_body_util::Full;
use bytes::Bytes;
use serde::{{Deserialize, Serialize}};

pub struct {} {{}}

impl {} {{
    pub async fn index(_req: OxiditeRequest) -> Result<OxiditeResponse> {{
        let response = serde_json::json!({{
            "message": "List endpoint"
        }});
        
        let json = serde_json::to_vec(&response)
            .map_err(|e| oxidite_core::Error::Server(e.to_string()))?;
        
        Ok(hyper::Response::builder()
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(json)))
            .unwrap())
    }}
    
    pub async fn show(_req: OxiditeRequest) -> Result<OxiditeResponse> {{
        // TODO: Implement show
        todo!()
    }}
    
    pub async fn create(_req: OxiditeRequest) -> Result<OxiditeResponse> {{
        // TODO: Implement create
        todo!()
    }}
    
    pub async fn update(_req: OxiditeRequest) -> Result<OxiditeResponse> {{
        // TODO: Implement update
        todo!()
    }}
    
    pub async fn destroy(_req: OxiditeRequest) -> Result<OxiditeResponse> {{
        // TODO: Implement destroy
        todo!()
    }}
}}
"#, name, name);
    
    let filename = format!("src/controllers/{}.rs", name.to_lowercase());
    fs::write(&filename, controller_template)?;
    
    println!("✅ Controller created: {}", filename);
    Ok(())
}

pub fn make_middleware(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create middleware directory if it doesn't exist
    let middleware_dir = Path::new("src/middleware");
    if !middleware_dir.exists() {
        fs::create_dir_all(middleware_dir)?;
    }
    
    let middleware_template = format!(r#"use oxidite_core::{{OxiditeRequest, OxiditeResponse, Error as CoreError}};
use tower::{{Service, Layer}};
use std::task::{{Context, Poll}};
use std::future::Future;
use std::pin::Pin;

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
    S: Service<OxiditeRequest, Response = OxiditeResponse, Error = CoreError> + Clone + Send + 'static,
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
            
            // Call inner service
            let response = inner.call(req).await?;
            
            // Post-processing
            println!("After request");
            
            Ok(response)
        }})
    }}
}}

pub struct {}Layer {{}}

impl {}Layer {{
    pub fn new() -> Self {{
        Self {{}}
    }}
}}

impl<S> Layer<S> for {}Layer {{
    type Service = {}<S>;

    fn layer(&self, inner: S) -> Self::Service {{
        {}<S>::new(inner)
    }}
}}
"#, name, name, name, name, name, name, name, name);
    
    let filename = format!("src/middleware/{}.rs", name.to_lowercase());
    fs::write(&filename, middleware_template)?;
    
    println!("✅ Middleware created: {}", filename);
    Ok(())
}
