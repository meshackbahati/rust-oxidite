use oxidite_core::{Request, Response, Error, json};

/// List users V2 (New format)
pub async fn list_users_v2(_req: Request) -> Result<Response, Error> {
    let users = vec![
        serde_json::json!({
            "uuid": "1",
            "contact": "alice@example.com",
            "display_name": "Alice (V2)",
            "role": "admin"
        }),
        serde_json::json!({
            "uuid": "2",
            "contact": "bob@example.com",
            "display_name": "Bob (V2)",
            "role": "user"
        }),
    ];
    
    Ok(json(serde_json::json!({
        "data": users,
        "meta": {
            "count": users.len(),
            "version": "v2"
        }
    })))
}
