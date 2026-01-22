// Example: Advanced error handling in Oxidite
// Demonstrates the enhanced error types and status codes

use oxidite::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct User {
    id: u64,
    name: String,
    email: String,
    active: bool,
}

// In-memory user store for demonstration
#[derive(Clone)]
struct UserStore {
    users: Arc<RwLock<HashMap<u64, User>>>,
    next_id: Arc<RwLock<u64>>,
}

impl UserStore {
    fn new() -> Self {
        let mut users = HashMap::new();
        users.insert(1, User {
            id: 1,
            name: "Alice Johnson".to_string(),
            email: "alice@example.com".to_string(),
            active: true,
        });
        users.insert(2, User {
            id: 2,
            name: "Bob Smith".to_string(),
            email: "bob@example.com".to_string(),
            active: true,
        });
        
        Self {
            users: Arc::new(RwLock::new(users)),
            next_id: Arc::new(RwLock::new(3)),
        }
    }
    
    fn get_user(&self, id: u64) -> Option<User> {
        let users = self.users.read().unwrap();
        users.get(&id).cloned()
    }
    
    fn get_all_users(&self) -> Vec<User> {
        let users = self.users.read().unwrap();
        users.values().cloned().collect()
    }
    
    fn create_user(&self, mut user: User) -> Result<User> {
        let mut next_id = self.next_id.write().unwrap();
        let id = *next_id;
        user.id = id;
        *next_id += 1;
        
        drop(next_id); // Release the lock before inserting
        
        let mut users = self.users.write().unwrap();
        if users.contains_key(&id) {
            return Err(Error::Conflict("User ID already exists".to_string()));
        }
        
        users.insert(id, user.clone());
        Ok(user)
    }
    
    fn update_user(&self, id: u64, updated_user: User) -> Result<User> {
        let mut users = self.users.write().unwrap();
        
        if !users.contains_key(&id) {
            return Err(Error::NotFound);
        }
        
        if updated_user.email != users[&id].email {
            // Check for duplicate email
            for (_, existing_user) in users.iter() {
                if existing_user.email == updated_user.email && existing_user.id != id {
                    return Err(Error::Conflict("Email already exists".to_string()));
                }
            }
        }
        
        let mut user = updated_user;
        user.id = id; // Ensure ID is preserved
        
        users.insert(id, user.clone());
        Ok(user)
    }
    
    fn delete_user(&self, id: u64) -> Result<()> {
        let mut users = self.users.write().unwrap();
        if users.contains_key(&id) {
            users.remove(&id);
            Ok(())
        } else {
            Err(Error::NotFound)
        }
    }
    
    fn deactivate_user(&self, id: u64) -> Result<()> {
        let mut users = self.users.write().unwrap();
        if let Some(mut user) = users.get_mut(&id) {
            if !user.active {
                return Err(Error::Conflict("User already deactivated".to_string()));
            }
            user.active = false;
            Ok(())
        } else {
            Err(Error::NotFound)
        }
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

#[derive(Deserialize)]
struct UpdateUserRequest {
    name: Option<String>,
    email: Option<String>,
    active: Option<bool>,
}

#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    limit: Option<u32>,
    active_only: Option<bool>,
}

// Utility function to validate email format
fn validate_email(email: &str) -> Result<()> {
    if !email.contains('@') || !email.contains('.') {
        return Err(Error::Validation("Invalid email format".to_string()));
    }
    Ok(())
}

// GET / - API info
async fn api_info(_req: Request) -> Result<Response> {
    Ok(response::json(serde_json::json!({
        "message": "Oxidite Advanced Error Handling Demo API",
        "version": "1.0",
        "endpoints": {
            "GET /users": "List all users",
            "GET /users/:id": "Get user by ID",
            "POST /users": "Create a new user",
            "PUT /users/:id": "Update a user",
            "DELETE /users/:id": "Delete a user",
            "POST /users/:id/deactivate": "Deactivate a user",
            "GET /validation-test": "Test validation errors"
        }
    })))
}

// GET /users - List users with pagination and filtering
async fn list_users(
    State(state): State<Arc<AppState>>,
    Query(params): Query<Pagination>,
    _req: Request
) -> Result<Response> {
    let users = state.user_store.get_all_users();
    
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(10).min(100); // Max 100 per page
    let active_only = params.active_only.unwrap_or(false);
    
    let filtered_users: Vec<User> = if active_only {
        users.into_iter().filter(|u| u.active).collect()
    } else {
        users
    };
    
    let start = ((page - 1) * limit) as usize;
    let end = std::cmp::min(start + limit as usize, filtered_users.len());
    let paginated_users = filtered_users[start..end].to_vec();
    
    Ok(response::json(serde_json::json!({
        "users": paginated_users,
        "pagination": {
            "page": page,
            "limit": limit,
            "total": filtered_users.len(),
            "pages": (filtered_users.len() as f64 / limit as f64).ceil() as u32
        },
        "filters": {
            "active_only": active_only
        }
    })))
}

// GET /users/:id - Get user by ID
async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(params): Path<serde_json::Value>,
    _req: Request
) -> Result<Response> {
    let id = params["id"].as_u64().ok_or_else(|| 
        Error::BadRequest("Invalid user ID format".to_string())
    )?;
    
    match state.user_store.get_user(id) {
        Some(user) => Ok(response::json(serde_json::json!(user))),
        None => Err(Error::NotFound),
    }
}

// POST /users - Create a new user
async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserRequest>,
    _req: Request
) -> Result<Response> {
    // Validate input
    if payload.name.trim().is_empty() {
        return Err(Error::Validation("Name cannot be empty".to_string()));
    }
    
    validate_email(&payload.email)?;
    
    // Check for duplicate email
    let all_users = state.user_store.get_all_users();
    for user in all_users {
        if user.email == payload.email {
            return Err(Error::Conflict("Email already exists".to_string()));
        }
    }
    
    let user = User {
        id: 0, // Will be assigned by store
        name: payload.name,
        email: payload.email,
        active: true,
    };
    
    let created_user = state.user_store.create_user(user)?;
    
    let mut response = response::json(serde_json::json!(created_user));
    *response.status_mut() = hyper::StatusCode::CREATED;
    Ok(response)
}

// PUT /users/:id - Update a user
async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(params): Path<serde_json::Value>,
    Json(payload): Json<UpdateUserRequest>,
    _req: Request
) -> Result<Response> {
    let id = params["id"].as_u64().ok_or_else(|| 
        Error::BadRequest("Invalid user ID format".to_string())
    )?;
    
    let existing_user = state.user_store.get_user(id)
        .ok_or(Error::NotFound)?;
    
    let updated_user = User {
        id,
        name: payload.name.unwrap_or(existing_user.name),
        email: payload.email.unwrap_or(existing_user.email),
        active: payload.active.unwrap_or(existing_user.active),
    };
    
    // Validate email if provided
    if let Some(email) = &payload.email {
        validate_email(email)?;
        
        // Check for duplicate email
        let all_users = state.user_store.get_all_users();
        for user in all_users {
            if user.email == *email && user.id != id {
                return Err(Error::Conflict("Email already exists".to_string()));
            }
        }
    }
    
    let updated_user = state.user_store.update_user(id, updated_user)?;
    
    Ok(response::json(serde_json::json!(updated_user)))
}

// DELETE /users/:id - Delete a user
async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(params): Path<serde_json::Value>,
    _req: Request
) -> Result<Response> {
    let id = params["id"].as_u64().ok_or_else(|| 
        Error::BadRequest("Invalid user ID format".to_string())
    )?;
    
    state.user_store.delete_user(id)?;
    
    Ok(response::json(serde_json::json!({
        "message": "User deleted successfully"
    })))
}

// POST /users/:id/deactivate - Deactivate a user
async fn deactivate_user(
    State(state): State<Arc<AppState>>,
    Path(params): Path<serde_json::Value>,
    _req: Request
) -> Result<Response> {
    let id = params["id"].as_u64().ok_or_else(|| 
        Error::BadRequest("Invalid user ID format".to_string())
    )?;
    
    state.user_store.deactivate_user(id)?;
    
    Ok(response::json(serde_json::json!({
        "message": "User deactivated successfully"
    })))
}

// GET /validation-test - Test validation errors
async fn validation_test(_req: Request) -> Result<Response> {
    // Simulate various validation errors
    let query = _req.uri().query().unwrap_or("");
    let params: std::collections::HashMap<String, String> = 
        serde_urlencoded::from_str(query).unwrap_or_default();
    
    if let Some(email) = params.get("email") {
        validate_email(email)?;
    }
    
    if let Some(name) = params.get("name") {
        if name.trim().is_empty() {
            return Err(Error::Validation("Name cannot be empty".to_string()));
        }
    }
    
    Ok(response::json(serde_json::json!({
        "message": "Validation passed"
    })))
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting Oxidite Advanced Error Handling Demo");
    println!("📍 Listening on http://127.0.0.1:3000");
    println!("📋 Features demonstrated:");
    println!("   • Enhanced error types with HTTP status codes");
    println!("   • Input validation");
    println!("   • Conflict detection");
    println!("   • Proper HTTP status codes");
    println!();
    println!("🧪 Test endpoints:");
    println!("   GET  http://localhost:3000/");
    println!("   GET  http://localhost:3000/users");
    println!("   GET  http://localhost:3000/users/1");
    println!("   POST http://localhost:3000/users (JSON body: {\"name\":\"John\",\"email\":\"john@example.com\"})");
    println!("   PUT  http://localhost:3000/users/1 (JSON body: {\"name\":\"Updated Name\"})");
    println!("   DELETE http://localhost:3000/users/1");
    println!("   POST http://localhost:3000/users/1/deactivate");
    println!("   GET  http://localhost:3000/validation-test?email=invalid-email");
    println!();

    let user_store = UserStore::new();
    let app_state = Arc::new(AppState {
        user_store,
    });

    let mut router = Router::new();
    
    // Register routes
    router.get("/", api_info);
    router.get("/users", list_users);
    router.get("/users/:id", get_user);
    router.post("/users", create_user);
    router.put("/users/:id", update_user);
    router.delete("/users/:id", delete_user);
    router.post("/users/:id/deactivate", deactivate_user);
    router.get("/validation-test", validation_test);

    // Add state to service using extension layer
    let service = oxidite_middleware::tower::ServiceBuilder::new()
        .layer(oxidite_middleware::tower_http::add_extension::AddExtensionLayer::new(app_state))
        .service(router);

    Server::new(service)
        .listen("127.0.0.1:3000".parse().unwrap())
        .await?;

    Ok(())
}