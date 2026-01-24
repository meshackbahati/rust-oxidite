use oxidite::prelude::*;
use oxidite_queue::Queue;
use std::sync::Arc;

pub async fn init_queue() -> Result<Arc<Queue>> {
    println!("Initializing queue...");
    
    let queue = Queue::memory();
    
    println!("Queue initialized.");
    Ok(Arc::new(queue))
}

pub fn queue_routes(router: &mut Router) {
    router.post("/queue/job", |_req: Request| async {
        Ok(Response::json(serde_json::json!({ "status": "job_enqueued" })))
    });
}
