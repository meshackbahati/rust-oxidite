use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use crate::Result;

/// Job status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    /// Job is waiting to be processed
    Pending,
    /// Job is currently running
    Running,
    /// Job completed successfully
    Completed,
    /// Job failed
    Failed,
    /// Job is being retried
    Retrying,
    /// Permanently failed after max retries
    DeadLetter,
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
    /// Unique job identifier
    pub id: String,
    /// Job type name
    pub name: String,
    /// Serialized job payload
    pub payload: serde_json::Value,
    /// Current job status
    pub status: JobStatus,
    /// Number of execution attempts
    pub attempts: u32,
    /// Maximum number of retries
    pub max_retries: u32,
    /// Unix timestamp of creation
    pub created_at: i64,
    /// Optional scheduled run time
    pub scheduled_at: Option<i64>,
    /// Job priority (higher = more important)
    pub priority: i32,
    /// Cron expression for recurring jobs
    pub cron_schedule: Option<String>,
    /// Last execution timestamp
    pub last_run_at: Option<i64>,
    /// Last error message
    pub error: Option<String>,
}

impl JobWrapper {
    /// Create a new job wrapper from a job implementation
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
            cron_schedule: None,
            last_run_at: None,
            error: None,
        })
    }

    /// Schedule the job with a delay from now
    pub fn with_delay(mut self, delay: Duration) -> Self {
        let scheduled_time = chrono::Utc::now().timestamp() + delay.as_secs() as i64;
        self.scheduled_at = Some(scheduled_time);
        self
    }

    /// Schedule job with cron expression (e.g., "0 0 * * * *" for hourly)
    pub fn with_cron(mut self, cron_expr: String) -> Self {
        self.scheduled_at = Self::next_cron_run(&cron_expr);
        self.cron_schedule = Some(cron_expr);
        self
    }

    /// Calculate next run time from cron expression
    fn next_cron_run(cron_expr: &str) -> Option<i64> {
        use cron::Schedule;
        use std::str::FromStr;
        
        if let Ok(schedule) = Schedule::from_str(cron_expr) {
            if let Some(next) = schedule.upcoming(chrono::Utc).next() {
                return Some(next.timestamp());
            }
        }
        None
    }

    /// Reschedule recurring job for next run
    pub fn reschedule(&mut self) {
        if let Some(cron_expr) = &self.cron_schedule {
            self.scheduled_at = Self::next_cron_run(cron_expr);
            self.status = JobStatus::Pending;
            self.attempts = 0;
            self.last_run_at = Some(chrono::Utc::now().timestamp());
        }
    }

    /// Check if job should be retried
    pub fn should_retry(&self) -> bool {
        self.status == JobStatus::Failed && self.attempts < self.max_retries
    }

    /// Calculate backoff duration for retry
    pub fn calculate_backoff(&self) -> Duration {
        // Exponential backoff: 60s * 2^attempts
        Duration::from_secs(60 * 2_u64.pow(self.attempts))
    }
}
