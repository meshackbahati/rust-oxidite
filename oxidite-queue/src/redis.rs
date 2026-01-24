use async_trait::async_trait;
use redis::{Client, AsyncCommands};
use crate::{QueueBackend, job::JobWrapper, Result, QueueError};

/// Redis queue backend
pub struct RedisBackend {
    client: Client,
    queue_key: String,
    dlq_key: String,
}

impl RedisBackend {
    pub fn new(url: &str, queue_key: &str) -> Result<Self> {
        let client = Client::open(url)
            .map_err(|e| QueueError::BackendError(e.to_string()))?;
        
        let dlq_key = format!("{}_dlq", queue_key);
        
        Ok(Self {
            client,
            queue_key: queue_key.to_string(),
            dlq_key,
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

    async fn fail(&self, job_id: &str, error: String) -> Result<()> {
        // Move failed jobs to a separate failed queue for inspection
        let failed_key = format!("{}_failed", self.queue_key);
        
        let mut conn = self.client.get_multiplexed_async_connection()
            .await
            .map_err(|e| QueueError::BackendError(e.to_string()))?;
        
        // Store the failed job with error information
        let failed_job = serde_json::json!({
            "job_id": job_id,
            "error": error,
            "failed_at": chrono::Utc::now().to_rfc3339()
        });
        
        let payload = serde_json::to_string(&failed_job)
            .map_err(|e| QueueError::SerializationError(e))?;
        
        let _: () = conn.lpush(&failed_key, payload)
            .await
            .map_err(|e| QueueError::BackendError(e.to_string()))?;
        
        Ok(())
    }

    async fn retry(&self, job: JobWrapper) -> Result<()> {
        self.enqueue(job).await
    }

    async fn move_to_dead_letter(&self, job: JobWrapper) -> Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection()
            .await
            .map_err(|e| QueueError::BackendError(e.to_string()))?;
            
        let payload = serde_json::to_string(&job)?;
        let _: () = conn.lpush(&self.dlq_key, payload)
            .await
            .map_err(|e| QueueError::BackendError(e.to_string()))?;
            
        Ok(())
    }

    async fn list_dead_letter(&self) -> Result<Vec<JobWrapper>> {
        let mut conn = self.client.get_multiplexed_async_connection()
            .await
            .map_err(|e| QueueError::BackendError(e.to_string()))?;
            
        let results: Vec<String> = conn.lrange(&self.dlq_key, 0, -1)
            .await
            .map_err(|e| QueueError::BackendError(e.to_string()))?;
            
        let mut jobs = Vec::new();
        for payload in results {
            if let Ok(job) = serde_json::from_str::<JobWrapper>(&payload) {
                jobs.push(job);
            }
        }
        
        Ok(jobs)
    }

    async fn retry_from_dead_letter(&self, job_id: &str) -> Result<()> {
        let jobs = self.list_dead_letter().await?;
        
        // Find the job by ID
        if let Some(job) = jobs.iter().find(|j| j.id == job_id) {
            // Remove from DLQ
            let mut conn = self.client.get_multiplexed_async_connection()
                .await
                .map_err(|e| QueueError::BackendError(e.to_string()))?;
                
            let payload = serde_json::to_string(&job)?;
            let _: () = conn.lrem(&self.dlq_key, 1, payload)
                .await
                .map_err(|e| QueueError::BackendError(e.to_string()))?;
            
            // Clone and reset before re-enqueue
            let mut job_clone = job.clone();
            job_clone.status = crate::job::JobStatus::Pending;
            job_clone.attempts = 0;
            job_clone.error = None;
            self.enqueue(job_clone).await?;
        }
        
        Ok(())
    }
}
