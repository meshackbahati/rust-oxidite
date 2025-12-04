use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use crate::Result;

/// Job status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Retrying,
}

/// Job result
pub type JobResult = Result<()>;

/// Trait for background jobs
#[async_trait]
pub trait Job: Send + Sync + Serialize + for<'de> Deserialize<'de> {
    /// Perform the job
    async fn perform(&self) -> JobResult;
    
    /// Maximum number of retries
    fn max_retries(&self) -> u32 {
        3
    }
    
    /// Backoff duration for retries
    fn backoff(&self, attempt: u32) -> Duration {
        Duration::from_secs(60 * 2_u64.pow(attempt))
    }
    
    /// Job priority (higher = more important)
    fn priority(&self) -> i32 {
        0
    }
    
    /// Job name for identification
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// Job wrapper for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobWrapper {
    pub id: String,
    pub name: String,
    pub payload: serde_json::Value,
    pub status: JobStatus,
    pub attempts: u32,
    pub max_retries: u32,
    pub created_at: i64,
    pub scheduled_at: Option<i64>,
    pub priority: i32,
}

impl JobWrapper {
    pub fn new<J: Job>(job: &J) -> Result<Self> {
        let payload = serde_json::to_value(job)?;
        let now = chrono::Utc::now().timestamp();
        
        Ok(Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: job.name().to_string(),
            payload,
            status: JobStatus::Pending,
            attempts: 0,
            max_retries: job.max_retries(),
            created_at: now,
            scheduled_at: None,
            priority: job.priority(),
        })
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        let scheduled_time = chrono::Utc::now().timestamp() + delay.as_secs() as i64;
        self.scheduled_at = Some(scheduled_time);
        self
    }
}
