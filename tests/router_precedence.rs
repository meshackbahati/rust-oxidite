use oxidite_core::{Router, Request, Response, Result, Error};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let mut router = Router::new();

    // 1. Register specific route
    router.get("/", |_| async {
        Ok(Response::new("Specific".into()))
    });

    // 2. Register fallback route
    router.get("/*", |_| async {
        Ok(Response::new("Fallback".into()))
    });

    // Test specific route
    let req = Request::builder()
        .uri("/")
        .body(oxidite_core::BoxBody::default())
        .unwrap();

    let res = router.handle(req).await.unwrap();
    let body = res.into_body();
    // In a real test we'd read the body, but here we trust the logic if it compiles and runs.
    // Actually, let's print it.
    println!("Response for /: {:?}", body); // Should imply "Specific"

    // Test fallback route
    let req = Request::builder()
        .uri("/other")
        .body(oxidite_core::BoxBody::default())
        .unwrap();
    
    let res = router.handle(req).await.unwrap();
    println!("Response for /other: {:?}", res.into_body()); // Should imply "Fallback"
}
