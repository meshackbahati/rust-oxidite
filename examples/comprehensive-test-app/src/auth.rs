use oxidite::prelude::*;
use oxidite_auth::JwtManager;
use std::sync::Arc;

pub async fn init_auth() -> Result<Arc<JwtManager>> {
    println!("Initializing auth...");
    
    // In a real app, this would come from config
    let secret = "my_secret_key".to_string();
    
    let auth = JwtManager::new(secret);
    
    println!("Auth initialized.");
    Ok(Arc::new(auth))
}

pub fn auth_routes(router: &mut Router) {
    router.post("/auth/login", |_req: Request| async {
        Ok(Response::json(serde_json::json!({ "token": "dummy_token" })))
    });
    
    router.post("/auth/register", |_req: Request| async {
        Ok(Response::json(serde_json::json!({ "status": "registered" })))
    });
}
