use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use crate::queue::Queue;
use crate::job::JobStatus;

/// Worker for processing jobs
pub struct Worker {
    queue: Arc<Queue>,
    worker_count: usize,
    poll_interval: Duration,
}

impl Worker {
    pub fn new(queue: Arc<Queue>) -> Self {
        Self {
            queue,
            worker_count: 4,
            poll_interval: Duration::from_secs(1),
        }
    }

    pub fn worker_count(mut self, count: usize) -> Self {
        self.worker_count = count;
        self
    }

    pub fn poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    pub async fn start(self) {
        println!("Starting {} workers...", self.worker_count);
        
        let mut handles = vec![];
        
        for i in 0..self.worker_count {
            let queue = self.queue.clone();
            let poll_interval = self.poll_interval;
            
            let handle = tokio::spawn(async move {
                loop {
                    match queue.dequeue().await {
                        Ok(Some(mut job)) => {
                            println!("Worker {}: Processing job {}", i, job.id);
                            
                            // In a real implementation, deserialize and execute the job
                            // For now, just mark as complete
                            sleep(Duration::from_millis(100)).await;
                            
                            if let Err(e) = queue.complete(&job.id).await {
                                eprintln!("Worker {}: Failed to mark job as complete: {}", i, e);
                            }
                        }
                        Ok(None) => {
                            // No jobs available, sleep
                            sleep(poll_interval).await;
                        }
                        Err(e) => {
                            eprintln!("Worker {}: Error dequeuing job: {}", i, e);
                            sleep(poll_interval).await;
                        }
                    }
                }
            });
            
            handles.push(handle);
        }

        // Wait for all workers (they run forever)
        for handle in handles {
            let _ = handle.await;
        }
    }
}
