use crate::error::{Error, Result};
use crate::types::OxiditeRequest;
use serde::de::DeserializeOwned;

/// Extract typed path parameters from the request
///
/// # Example
/// ```ignore
/// #[derive(Deserialize)]
/// struct UserPath {
///     id: u64,
/// }
///
/// async fn get_user(Path(params): Path<UserPath>) -> Result<Json<User>> {
///     let user = User::find(params.id).await?;
///     Ok(Json(user))
/// }
/// ```
pub struct Path<T>(pub T);

/// Extract typed query parameters from the request
///
/// # Example
/// ```ignore
/// #[derive(Deserialize)]
/// struct Pagination {
///     page: u32,
///     limit: u32,
/// }
///
/// async fn list_users(Query(params): Query<Pagination>) -> Result<Json<Vec<User>>> {
///     let users = User::paginate(params.page, params.limit).await?;
///     Ok(Json(users))
/// }
/// ```
pub struct Query<T>(pub T);

/// Extract and deserialize JSON request body
///
/// # Example
/// ```ignore
/// #[derive(Deserialize)]
/// struct CreateUser {
///     name: String,
///     email: String,
/// }
///
/// async fn create_user(Json(data): Json<CreateUser>) -> Result<Json<User>> {
///     let user = User::create(data).await?;
///     Ok(Json(user))
/// }
/// ```
pub struct Json<T>(pub T);

/// Extractor trait - allows types to be extracted from requests
pub trait FromRequest: Sized {
    fn from_request(req: &mut OxiditeRequest) -> impl std::future::Future<Output = Result<Self>> + Send;
}

impl<T: DeserializeOwned + Send> FromRequest for Path<T> {
    async fn from_request(req: &mut OxiditeRequest) -> Result<Self> {
        // Path params are stored in request extensions after routing
        req.extensions()
            .get::<PathParams>()
            .ok_or_else(|| Error::BadRequest("No path parameters found".to_string()))
            .and_then(|params| {
                serde_json::from_value(params.0.clone())
                    .map(Path)
                    .map_err(|e| Error::BadRequest(format!("Invalid path parameters: {}", e)))
            })
    }
}

impl<T: DeserializeOwned + Send> FromRequest for Query<T> {
    async fn from_request(req: &mut OxiditeRequest) -> Result<Self> {
        let query = req.uri().query().unwrap_or("");
        serde_urlencoded::from_str(query)
            .map(Query)
            .map_err(|e| Error::BadRequest(format!("Invalid query parameters: {}", e)))
    }
}

impl<T: DeserializeOwned + Send> FromRequest for Json<T> {
    async fn from_request(req: &mut OxiditeRequest) -> Result<Self> {
        use http_body_util::BodyExt;
        use bytes::Buf;

        let body = req.body_mut();
        let bytes = body.collect().await
            .map_err(|e| Error::InternalServerError(format!("Failed to read body: {}", e)))?
            .aggregate();

        serde_json::from_reader(bytes.reader())
            .map(Json)
            .map_err(|e| Error::BadRequest(format!("Invalid JSON: {}", e)))
    }
}

// Storage for path parameters extracted during routing
#[derive(Clone)]
pub struct PathParams(pub serde_json::Value);

// Helper to serialize responses as JSON
impl<T: serde::Serialize> Json<T> {
    pub fn into_response(self) -> Result<http_body_util::Full<bytes::Bytes>> {
        let body = serde_json::to_vec(&self.0)
            .map_err(|e| Error::InternalServerError(format!("Failed to serialize JSON: {}", e)))?;
        Ok(http_body_util::Full::new(bytes::Bytes::from(body)))
    }
}

/// Extract application state from request extensions
///
/// # Example
/// ```ignore
/// async fn handler(State(state): State<Arc<AppState>>) -> Result<Response> {
///     // use state
/// }
/// ```
pub struct State<T>(pub T);

impl<T: Clone + Send + Sync + 'static> FromRequest for State<T> {
    async fn from_request(req: &mut OxiditeRequest) -> Result<Self> {
        req.extensions()
            .get::<T>()
            .cloned()
            .map(State)
            .ok_or_else(|| Error::InternalServerError("Application state not found in request extensions".to_string()))
    }
}

/// Extract form data from the request body
///
/// # Example
/// ```ignore
/// #[derive(Deserialize)]
/// struct LoginForm {
///     username: String,
///     password: String,
/// }
///
/// async fn login(Form(data): Form<LoginForm>) -> Result<Json<Session>> {
///     // process login
/// }
/// ```
pub struct Form<T>(pub T);

impl<T: DeserializeOwned + Send> FromRequest for Form<T> {
    async fn from_request(req: &mut OxiditeRequest) -> Result<Self> {
        use http_body_util::BodyExt;
        use bytes::Buf;
        
        // Check content type
        let content_type = req.headers()
            .get("content-type")
            .and_then(|ct| ct.to_str().ok())
            .unwrap_or("");
            
        if !content_type.starts_with("application/x-www-form-urlencoded") {
            return Err(Error::BadRequest(
                "Expected application/x-www-form-urlencoded content type".to_string()
            ));
        }
        
        let body = req.body_mut();
        let bytes = body.collect().await
            .map_err(|e| Error::InternalServerError(format!("Failed to read body: {}", e)))?
            .aggregate();
        
        let body_str = std::str::from_utf8(bytes.chunk())
            .map_err(|e| Error::BadRequest(format!("Invalid UTF-8 in form data: {}", e)))?;
        
        serde_urlencoded::from_str(body_str)
            .map(Form)
            .map_err(|e| Error::BadRequest(format!("Invalid form data: {}", e)))
    }
}

/// Extract cookies from the request
///
/// # Example
/// ```ignore
/// async fn handler(cookies: Cookies) -> Result<Response> {
///     if let Some(token) = cookies.get("auth_token") {
///         // use token
///     }
///     Ok(Response::text("OK"))
/// }
/// ```
pub struct Cookies {
    cookies: std::collections::HashMap<String, String>,
}

impl Cookies {
    pub fn get(&self, name: &str) -> Option<&String> {
        self.cookies.get(name)
    }
    
    pub fn contains_key(&self, name: &str) -> bool {
        self.cookies.contains_key(name)
    }
    
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.cookies.iter()
    }
}

impl FromRequest for Cookies {
    async fn from_request(req: &mut OxiditeRequest) -> Result<Self> {
        let mut cookies_map = std::collections::HashMap::new();
        
        if let Some(cookie_header) = req.headers().get(http::header::COOKIE) {
            if let Ok(cookie_str) = cookie_header.to_str() {
                for cookie_pair in cookie_str.split(';') {
                    let trimmed = cookie_pair.trim();
                    if let Some((name, value)) = trimmed.split_once('=') {
                        cookies_map.insert(name.trim().to_string(), value.trim().to_string());
                    }
                }
            }
        }
        
        Ok(Cookies { cookies: cookies_map })
    }
}

/// Extract raw request body as string
///
/// # Example
/// ```ignore
/// async fn webhook_handler(Body(raw): Body<String>) -> Result<Response> {
///     // process raw body
///     Ok(Response::text("Received"))
/// }
/// ```
pub struct Body<T>(pub T);

impl FromRequest for Body<String> {
    async fn from_request(req: &mut OxiditeRequest) -> Result<Self> {
        use http_body_util::BodyExt;
        use bytes::Buf;
        
        let body = req.body_mut();
        let bytes = body.collect().await
            .map_err(|e| Error::InternalServerError(format!("Failed to read body: {}", e)))?
            .aggregate();
        
        let body_str = std::str::from_utf8(bytes.chunk())
            .map_err(|e| Error::InternalServerError(format!("Invalid UTF-8 in body: {}", e)))?
            .to_string();
        
        Ok(Body(body_str))
    }
}

impl FromRequest for Body<Vec<u8>> {
    async fn from_request(req: &mut OxiditeRequest) -> Result<Self> {
        use http_body_util::BodyExt;
        
        let body = req.body_mut();
        let bytes = body.collect().await
            .map_err(|e| Error::InternalServerError(format!("Failed to read body: {}", e)))?
            .to_bytes();
        
        Ok(Body(bytes.to_vec()))
    }
}
