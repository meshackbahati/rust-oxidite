use async_trait::async_trait;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::job::{JobWrapper, JobStatus};
use crate::stats::StatsTracker;
use crate::Result;

/// Queue backend trait
#[async_trait]
pub trait QueueBackend: Send + Sync {
    async fn enqueue(&self, job: JobWrapper) -> Result<()>;
    async fn dequeue(&self) -> Result<Option<JobWrapper>>;
    async fn complete(&self, job_id: &str) -> Result<()>;
    async fn fail(&self, job_id: &str, error: String) -> Result<()>;
    async fn retry(&self, job: JobWrapper) -> Result<()>;
    async fn move_to_dead_letter(&self, job: JobWrapper) -> Result<()>;
    async fn list_dead_letter(&self) -> Result<Vec<JobWrapper>>;
    async fn retry_from_dead_letter(&self, job_id: &str) -> Result<()>;
    async fn clear(&self) -> Result<()>;
}

/// In-memory queue backend
pub struct MemoryBackend {
    queue: Arc<Mutex<VecDeque<JobWrapper>>>,
    dead_letter: Arc<Mutex<Vec<JobWrapper>>>,
}

impl MemoryBackend {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            dead_letter: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Default for MemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl QueueBackend for MemoryBackend {
    async fn enqueue(&self, mut job: JobWrapper) -> Result<()> {
        job.status = JobStatus::Pending;
        let mut queue = self.queue.lock().await;
        
        // Insert based on priority (higher priority first)
        let pos = queue.iter().position(|j| j.priority < job.priority)
            .unwrap_or(queue.len());
        queue.insert(pos, job);
        
        Ok(())
    }

    async fn dequeue(&self) -> Result<Option<JobWrapper>> {
        let mut queue = self.queue.lock().await;
        
        // Find first job that can be run now
        let now = chrono::Utc::now().timestamp();
        let pos = queue.iter().position(|j| {
            j.status == JobStatus::Pending &&
            j.scheduled_at.map(|t| t <= now).unwrap_or(true)
        });

        if let Some(pos) = pos {
            let mut job = queue.remove(pos).unwrap();
            job.status = JobStatus::Running;
            job.attempts += 1;
            Ok(Some(job))
        } else {
            Ok(None)
        }
    }

    async fn complete(&self, _job_id: &str) -> Result<()> {
        // In memory backend doesn't need to track completed jobs
        Ok(())
    }

    async fn fail(&self, _job_id: &str, _error: String) -> Result<()> {
        // In memory backend doesn't need to track failed jobs
        Ok(())
    }

    async fn retry(&self, job: JobWrapper) -> Result<()> {
        self.enqueue(job).await
    }

    async fn move_to_dead_letter(&self, mut job: JobWrapper) -> Result<()> {
        job.status = JobStatus::DeadLetter;
        let mut dlq = self.dead_letter.lock().await;
        dlq.push(job);
        Ok(())
    }

    async fn list_dead_letter(&self) -> Result<Vec<JobWrapper>> {
        let dlq = self.dead_letter.lock().await;
        Ok(dlq.clone())
    }

    async fn retry_from_dead_letter(&self, job_id: &str) -> Result<()> {
        let mut dlq = self.dead_letter.lock().await;
        if let Some(pos) = dlq.iter().position(|j| j.id == job_id) {
            let mut job = dlq.remove(pos);
            job.status = JobStatus::Pending;
            job.attempts = 0;
            job.error = None;
            drop(dlq); // Release lock before enqueue
            self.enqueue(job).await?;
        }
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        let mut queue = self.queue.lock().await;
        queue.clear();
        Ok(())
    }
}

/// Queue for managing jobs
pub struct Queue {
    backend: Arc<dyn QueueBackend>,
    stats: StatsTracker,
}

impl Queue {
    pub fn new(backend: Arc<dyn QueueBackend>) -> Self {
        Self {
            backend,
            stats: StatsTracker::new(),
        }
    }

    pub fn memory() -> Self {
        Self::new(Arc::new(MemoryBackend::new()))
    }

    pub async fn enqueue(&self, job: JobWrapper) -> Result<String> {
        let job_id = job.id.clone();
        self.backend.enqueue(job).await?;
        self.stats.increment_enqueued().await;
        Ok(job_id)
    }

    pub async fn dequeue(&self) -> Result<Option<JobWrapper>> {
        let job = self.backend.dequeue().await?;
        if job.is_some() {
            self.stats.mark_running().await;
        }
        Ok(job)
    }

    pub async fn complete(&self, job_id: &str) -> Result<()> {
        self.backend.complete(job_id).await?;
        self.stats.increment_processed().await;
        Ok(())
    }

    pub async fn fail(&self, job_id: &str, error: String) -> Result<()> {
        self.backend.fail(job_id, error).await?;
        self.stats.increment_failed().await;
        Ok(())
    }

    pub async fn retry(&self, job: JobWrapper) -> Result<()> {
        self.backend.retry(job).await?;
        self.stats.increment_retried().await;
        Ok(())
    }

    pub async fn move_to_dead_letter(&self, job: JobWrapper) -> Result<()> {
        self.backend.move_to_dead_letter(job).await?;
        self.stats.increment_dead_letter().await;
        Ok(())
    }

    pub async fn list_dead_letter(&self) -> Result<Vec<JobWrapper>> {
        self.backend.list_dead_letter().await
    }

    pub async fn retry_from_dead_letter(&self, job_id: &str) -> Result<()> {
        self.backend.retry_from_dead_letter(job_id).await
    }

    pub async fn clear(&self) -> Result<()> {
        self.backend.clear().await?;
        self.stats.reset().await;
        Ok(())
    }

    pub async fn get_stats(&self) -> crate::stats::QueueStats {
        self.stats.get_stats().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::job::Job;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct TestJob {
        value: i32,
    }

    #[async_trait::async_trait]
    impl Job for TestJob {
        async fn perform(&self) -> crate::Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_enqueue_dequeue() {
        let queue = Queue::memory();
        let job = JobWrapper::new(&TestJob { value: 42 }).unwrap();
        
        queue.enqueue(job).await.unwrap();
        let dequeued = queue.dequeue().await.unwrap();
        
        assert!(dequeued.is_some());
    }
}
