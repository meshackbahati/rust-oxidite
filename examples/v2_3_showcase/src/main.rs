use oxidite::prelude::*;
use oxidite::middleware::CorsLayer;
use std::sync::Arc;
use chrono::{DateTime, Utc};

#[derive(Clone)]
struct AppState {
    app_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: i64,
    name: String,
    email: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

async fn handler_12_extractors(
    State(state): State<Arc<AppState>>,
    _e2: State<Arc<AppState>>, _e3: State<Arc<AppState>>, _e4: State<Arc<AppState>>,
    _e5: State<Arc<AppState>>, _e6: State<Arc<AppState>>, _e7: State<Arc<AppState>>,
    _e8: State<Arc<AppState>>, _e9: State<Arc<AppState>>, _e10: State<Arc<AppState>>,
    _e11: State<Arc<AppState>>, _e12: State<Arc<AppState>>,
) -> Result<Response> {
    println!("App Name: {}", state.app_name);
    Ok(Response::text("Extracted 12!").header("X-Custom", "Value"))
}

async fn authenticated_ws(ws: WebSocketUpgrade) -> Result<Response> {
    Ok(ws.on_upgrade(|_socket, extensions| async move {
        if let Some(user) = extensions.get::<User>() {
            println!("Authenticated WebSocket user: {}", user.name);
        }
    }))
}

#[tokio::main]
async fn main() -> Result<()> {
    let state = Arc::new(AppState {
        app_name: "Oxidite 2.3 Showcase".to_string(),
    });

    let app = Router::new()
        .layer(CorsLayer::permissive())
        .with_state(state);

    app.get("/heavy", handler_12_extractors);
    app.get("/ws", authenticated_ws);

    println!("Oxidite 2.3 showcase ready.");

    Server::new(app)
        .bind("127.0.0.1:3000".parse().unwrap())
        .run()
        .await
}
