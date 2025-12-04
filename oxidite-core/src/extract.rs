use crate::error::{Error, Result};
use crate::types::OxiditeRequest;
use serde::de::DeserializeOwned;
use std::marker::PhantomData;

/// Extract typed path parameters from the request
///
/// # Example
/// ```
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
/// ```
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
/// ```
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
    async fn from_request(req: &mut OxiditeRequest) -> Result<Self>;
}

impl<T: DeserializeOwned> FromRequest for Path<T> {
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

impl<T: DeserializeOwned> FromRequest for Query<T> {
    async fn from_request(req: &mut OxiditeRequest) -> Result<Self> {
        let query = req.uri().query().unwrap_or("");
        serde_urlencoded::from_str(query)
            .map(Query)
            .map_err(|e| Error::BadRequest(format!("Invalid query parameters: {}", e)))
    }
}

impl<T: DeserializeOwned> FromRequest for Json<T> {
    async fn from_request(req: &mut OxiditeRequest) -> Result<Self> {
        use http_body_util::BodyExt;
        use bytes::Buf;

        let body = req.body_mut();
        let bytes = body.collect().await
            .map_err(|e| Error::Server(format!("Failed to read body: {}", e)))?
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
            .map_err(|e| Error::Server(format!("Failed to serialize JSON: {}", e)))?;
        Ok(http_body_util::Full::new(bytes::Bytes::from(body)))
    }
}
