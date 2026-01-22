// Example: API Versioning in Oxidite
// Demonstrates URL-based, header-based, and query parameter-based versioning

use oxidite::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct UserV1 {
    id: u64,
    name: String,
    email: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct UserV2 {
    id: u64,
    name: String,
    email: String,
    created_at: String,
    updated_at: String,
    profile: UserProfile,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct UserProfile {
    bio: Option<String>,
    avatar_url: Option<String>,
    is_verified: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ApiResponse<T> {
    version: String,
    data: T,
    timestamp: String,
}

#[derive(Clone)]
struct UserStore {
    users: Arc<RwLock<HashMap<u64, UserV1>>>,
    next_id: Arc<RwLock<u64>>,
}

impl UserStore {
    fn new() -> Self {
        let mut users = HashMap::new();
        users.insert(1, UserV1 {
            id: 1,
            name: "Alice Johnson".to_string(),
            email: "alice@example.com".to_string(),
        });
        users.insert(2, UserV1 {
            id: 2,
            name: "Bob Smith".to_string(),
            email: "bob@example.com".to_string(),
        });
        
        Self {
            users: Arc::new(RwLock::new(users)),
            next_id: Arc::new(RwLock::new(3)),
        }
    }
    
    fn get_user(&self, id: u64) -> Option<UserV1> {
        let users = self.users.read().unwrap();
        users.get(&id).cloned()
    }
    
    fn get_all_users(&self) -> Vec<UserV1> {
        let users = self.users.read().unwrap();
        users.values().cloned().collect()
    }
    
    fn create_user(&self, mut user: UserV1) -> Result<UserV1> {
        let mut next_id = self.next_id.write().unwrap();
        let id = *next_id;
        user.id = id;
        *next_id += 1;
        
        drop(next_id);
        
        let mut users = self.users.write().unwrap();
        users.insert(id, user.clone());
        Ok(user)
    }
}

#[derive(Clone)]
struct AppState {
    user_store: UserStore,
}

#[derive(Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

// Version 1 API handlers
async fn v1_get_user(
    State(state): State<Arc<AppState>>,
    Path(params): Path<serde_json::Value>
) -> Result<OxiditeResponse> {
    let id = params["id"].as_u64().ok_or_else(|| 
        Error::BadRequest("Invalid user ID format".to_string())
    )?;
    
    match state.user_store.get_user(id) {
        Some(user) => {
            let response = ApiResponse {
                version: "v1".to_string(),
                data: user,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            Ok(response::json(serde_json::json!(response)))
        },
        None => Err(Error::NotFound),
    }
}

async fn v1_list_users(
    State(state): State<Arc<AppState>>
) -> Result<OxiditeResponse> {
    let users = state.user_store.get_all_users();
    let response = ApiResponse {
        version: "v1".to_string(),
        data: users,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    Ok(response::json(serde_json::json!(response)))
}

async fn v1_create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserRequest>
) -> Result<OxiditeResponse> {
    let user = UserV1 {
        id: 0, // Will be assigned by store
        name: payload.name,
        email: payload.email,
    };
    
    let created_user = state.user_store.create_user(user)?;
    let response = ApiResponse {
        version: "v1".to_string(),
        data: created_user,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    let mut response = response::json(serde_json::json!(response));
    *response.status_mut() = hyper::StatusCode::CREATED;
    Ok(response)
}

// Version 2 API handlers
async fn v2_get_user(
    State(state): State<Arc<AppState>>,
    Path(params): Path<serde_json::Value>
) -> Result<OxiditeResponse> {
    let id = params["id"].as_u64().ok_or_else(|| 
        Error::BadRequest("Invalid user ID format".to_string())
    )?;
    
    match state.user_store.get_user(id) {
        Some(v1_user) => {
            // Transform V1 user to V2 user
            let v2_user = UserV2 {
                id: v1_user.id,
                name: v1_user.name,
                email: v1_user.email,
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
                profile: UserProfile {
                    bio: Some("Default bio".to_string()),
                    avatar_url: None,
                    is_verified: false,
                },
            };
            
            let response = ApiResponse {
                version: "v2".to_string(),
                data: v2_user,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            Ok(response::json(serde_json::json!(response)))
        },
        None => Err(Error::NotFound),
    }
}

async fn v2_list_users(
    State(state): State<Arc<AppState>>
) -> Result<OxiditeResponse> {
    let v1_users = state.user_store.get_all_users();
    
    // Transform V1 users to V2 users
    let v2_users: Vec<UserV2> = v1_users.into_iter().map(|v1_user| UserV2 {
        id: v1_user.id,
        name: v1_user.name,
        email: v1_user.email,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        profile: UserProfile {
            bio: Some("Default bio".to_string()),
            avatar_url: None,
            is_verified: false,
        },
    }).collect();
    
    let response = ApiResponse {
        version: "v2".to_string(),
        data: v2_users,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    Ok(response::json(serde_json::json!(response)))
}

async fn v2_create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserRequest>
) -> Result<OxiditeResponse> {
    let v1_user = UserV1 {
        id: 0, // Will be assigned by store
        name: payload.name,
        email: payload.email,
    };
    
    let created_v1_user = state.user_store.create_user(v1_user)?;
    
    // Transform to V2 user
    let v2_user = UserV2 {
        id: created_v1_user.id,
        name: created_v1_user.name,
        email: created_v1_user.email,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        profile: UserProfile {
            bio: Some("Default bio".to_string()),
            avatar_url: None,
            is_verified: false,
        },
    };
    
    let response = ApiResponse {
        version: "v2".to_string(),
        data: v2_user,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    let mut response = response::json(serde_json::json!(response));
    *response.status_mut() = hyper::StatusCode::CREATED;
    Ok(response)
}

// Version-agnostic handler that determines version from headers
async fn versioned_api_handler(
    State(state): State<Arc<AppState>>,
    req: OxiditeRequest
) -> Result<OxiditeResponse> {
    // Determine version from Accept header (e.g., application/vnd.api+json;version=2)
    let version = req.headers()
        .get("accept")
        .and_then(|hv| hv.to_str().ok())
        .and_then(|s| {
            if s.contains("version=2") {
                Some(2)
            } else {
                Some(1) // Default to version 1
            }
        })
        .unwrap_or(1);
    
    match version {
        1 => {
            let users = state.user_store.get_all_users();
            let response = ApiResponse {
                version: "v1".to_string(),
                data: users,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            Ok(response::json(serde_json::json!(response)))
        },
        2 => {
            let v1_users = state.user_store.get_all_users();
            let v2_users: Vec<UserV2> = v1_users.into_iter().map(|v1_user| UserV2 {
                id: v1_user.id,
                name: v1_user.name,
                email: v1_user.email,
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
                profile: UserProfile {
                    bio: Some("Default bio".to_string()),
                    avatar_url: None,
                    is_verified: false,
                },
            }).collect();
            
            let response = ApiResponse {
                version: "v2".to_string(),
                data: v2_users,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            Ok(response::json(serde_json::json!(response)))
        },
        _ => Err(Error::BadRequest("Unsupported API version".to_string())),
    }
}

// Version-agnostic handler that determines version from query parameter
async fn query_versioned_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<serde_json::Value>,
    req: OxiditeRequest
) -> Result<OxiditeResponse> {
    // Determine version from query parameter
    let version = params["version"]
        .as_u64()
        .or_else(|| {
            // Also check from headers as fallback
            req.headers()
                .get("x-api-version")
                .and_then(|hv| hv.to_str().ok())
                .and_then(|s| s.parse().ok())
        })
        .unwrap_or(1);
    
    match version {
        1 => {
            let users = state.user_store.get_all_users();
            let response = ApiResponse {
                version: "v1".to_string(),
                data: users,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            Ok(response::json(serde_json::json!(response)))
        },
        2 => {
            let v1_users = state.user_store.get_all_users();
            let v2_users: Vec<UserV2> = v1_users.into_iter().map(|v1_user| UserV2 {
                id: v1_user.id,
                name: v1_user.name,
                email: v1_user.email,
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
                profile: UserProfile {
                    bio: Some("Default bio".to_string()),
                    avatar_url: None,
                    is_verified: false,
                },
            }).collect();
            
            let response = ApiResponse {
                version: "v2".to_string(),
                data: v2_users,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            Ok(response::json(serde_json::json!(response)))
        },
        _ => Err(Error::BadRequest(format!("Unsupported API version: {}", version))),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting Oxidite API Versioning Demo");
    println!("📍 Listening on http://127.0.0.1:3000");
    println!("📋 API Versioning Approaches Demonstrated:");
    println!("   • URL-based versioning (e.g., /api/v1/users, /api/v2/users)");
    println!("   • Header-based versioning (Accept: application/vnd.api+json;version=2)");
    println!("   • Query parameter versioning (e.g., ?version=2)");
    println!();
    println!("🧪 Test endpoints:");
    println!("   GET  http://localhost:3000/api/v1/users");
    println!("   GET  http://localhost:3000/api/v2/users");
    println!("   GET  http://localhost:3000/versioned (try with header: Accept: application/vnd.api+json;version=2)");
    println!("   GET  http://localhost:3000/query-versioned?version=2");
    println!();

    let user_store = UserStore::new();
    let app_state = Arc::new(AppState {
        user_store,
    });

    let mut router = Router::new();
    
    // URL-based versioning
    router.get("/api/v1/users", v1_list_users);
    router.get("/api/v1/users/:id", v1_get_user);
    router.post("/api/v1/users", v1_create_user);
    
    router.get("/api/v2/users", v2_list_users);
    router.get("/api/v2/users/:id", v2_get_user);
    router.post("/api/v2/users", v2_create_user);
    
    // Header-based versioning
    router.get("/versioned", versioned_api_handler);
    
    // Query parameter versioning
    router.get("/query-versioned", query_versioned_handler);

    let service = oxidite_middleware::tower::ServiceBuilder::new()
        .layer(oxidite_middleware::tower_http::add_extension::AddExtensionLayer::new(app_state))
        .service(router);

    Server::new(service)
        .listen("127.0.0.1:3000".parse().unwrap())
        .await?;

    Ok(())
}