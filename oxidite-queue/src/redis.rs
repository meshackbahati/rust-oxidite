use async_trait::async_trait;
use redis::{Client, AsyncCommands};
use crate::{QueueBackend, job::JobWrapper, Result, QueueError};

/// Redis queue backend
pub struct RedisBackend {
    client: Client,
    queue_key: String,
}

impl RedisBackend {
    pub fn new(url: &str, queue_key: &str) -> Result<Self> {
        let client = Client::open(url)
            .map_err(|e| QueueError::BackendError(e.to_string()))?;
        
        Ok(Self {
            client,
            queue_key: queue_key.to_string(),
        })
    }
}

#[async_trait]
impl QueueBackend for RedisBackend {
    async fn enqueue(&self, job: JobWrapper) -> Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection()
            .await
            .map_err(|e| QueueError::BackendError(e.to_string()))?;
            
        let payload = serde_json::to_string(&job)?;
        
        // Use LPUSH to add to the head of the list (or tail, depending on how we want to process)
        // Standard queue is usually LPUSH (enqueue) and RPOP (dequeue)
        let _: () = conn.lpush(&self.queue_key, payload)
            .await
            .map_err(|e| QueueError::BackendError(e.to_string()))?;
            
        Ok(())
    }

    async fn dequeue(&self) -> Result<Option<JobWrapper>> {
        let mut conn = self.client.get_multiplexed_async_connection()
            .await
            .map_err(|e| QueueError::BackendError(e.to_string()))?;
            
        // RPOP removes and returns the last element of the list
        let result: Option<String> = conn.rpop(&self.queue_key, None)
            .await
            .map_err(|e| QueueError::BackendError(e.to_string()))?;
            
        if let Some(payload) = result {
            let job: JobWrapper = serde_json::from_str(&payload)?;
            Ok(Some(job))
        } else {
            Ok(None)
        }
    }

    async fn complete(&self, _job_id: &str) -> Result<()> {
        // In a simple RPOP implementation, the job is already removed from the queue.
        // For more reliability, we'd use RPOPLPUSH to a processing queue and then remove from there.
        // For this v1 implementation, we'll keep it simple.
        Ok(())
    }

    async fn fail(&self, _job_id: &str, _error: String) -> Result<()> {
        // Similarly, we might want to move to a failed queue.
        // TODO: Implement failed queue logic
        Ok(())
    }

    async fn retry(&self, job: JobWrapper) -> Result<()> {
        self.enqueue(job).await
    }
}
