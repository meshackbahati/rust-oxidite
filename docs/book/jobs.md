# Background Jobs

Background jobs allow you to process tasks asynchronously outside of the main request-response cycle. This chapter covers how to create, queue, and process background jobs in Oxidite.

## Overview

Background jobs are essential for:
- Processing long-running tasks
- Sending emails
- Processing files
- Integrating with external services
- Periodic maintenance tasks

## Job Definition

Define jobs by implementing the Job trait:

```rust
use oxidite::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SendEmailJob {
    pub recipient: String,
    pub subject: String,
    pub body: String,
}

#[async_trait::async_trait]
impl Job for SendEmailJob {
    type Output = Result<(), String>;
    
    async fn execute(self) -> Self::Output {
        // Simulate sending an email
        println!("Sending email to: {}", self.recipient);
        println!("Subject: {}", self.subject);
        println!("Body: {}", self.body);
        
        // In a real app, this would connect to an email service
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        Ok(())
    }
}

// Another example: Image processing job
#[derive(Serialize, Deserialize)]
pub struct ProcessImageJob {
    pub image_path: String,
    pub width: u32,
    pub height: u32,
}

#[async_trait::async_trait]
impl Job for ProcessImageJob {
    type Output = Result<String, String>;
    
    async fn execute(self) -> Self::Output {
        println!("Processing image: {}", self.image_path);
        println!("Resizing to {}x{}", self.width, self.height);
        
        // Simulate image processing
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        
        Ok(format!("processed_{}", self.image_path))
    }
}
```

## Queue Configuration

Configure queues for job processing:

```rust
use oxidite::prelude::*;
use oxidite_queue::{Queue, QueueBackend, RedisBackend};

async fn configure_queues() -> Result<()> {
    // Configure Redis backend
    let redis_backend = RedisBackend::new("redis://127.0.0.1:6379").await?;
    
    // Create queues
    let email_queue = Queue::new(redis_backend.clone());
    let image_queue = Queue::new(redis_backend.clone());
    let default_queue = Queue::new(redis_backend);
    
    // Store queues in application state
    // This would typically be done during app initialization
    
    Ok(())
}
```

## Enqueuing Jobs

Add jobs to the queue for processing:

```rust
use oxidite::prelude::*;

async fn enqueue_examples() -> Result<()> {
    // Get the queue (in a real app, this would come from state)
    let queue = get_queue("emails").await?;
    
    // Create and enqueue an email job
    let email_job = SendEmailJob {
        recipient: "user@example.com".to_string(),
        subject: "Welcome!".to_string(),
        body: "Thank you for joining our platform.".to_string(),
    };
    
    // Enqueue immediately
    let job_id = queue.enqueue(email_job).await?;
    println!("Enqueued email job with ID: {}", job_id);
    
    // Enqueue with delay (for scheduled tasks)
    let delayed_job = SendEmailJob {
        recipient: "user@example.com".to_string(),
        subject: "Reminder".to_string(),
        body: "This is a reminder about your account.".to_string(),
    };
    
    let delayed_job_id = queue.enqueue_delayed(delayed_job, std::time::Duration::from_secs(3600)).await?;
    println!("Enqueued delayed job with ID: {}", delayed_job_id);
    
    // Batch enqueue multiple jobs
    let jobs = vec![
        SendEmailJob {
            recipient: "user1@example.com".to_string(),
            subject: "Newsletter".to_string(),
            body: "Here's your weekly newsletter.".to_string(),
        },
        SendEmailJob {
            recipient: "user2@example.com".to_string(),
            subject: "Newsletter".to_string(),
            body: "Here's your weekly newsletter.".to_string(),
        },
    ];
    
    let batch_ids = queue.enqueue_batch(jobs).await?;
    println!("Enqueued {} jobs in batch", batch_ids.len());
    
    Ok(())
}

async fn get_queue(_name: &str) -> Result<Queue> {
    // In a real app, this would return the configured queue
    Ok(Queue::memory())
}

pub struct Queue {
    name: String,
}

impl Queue {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string() }
    }
    
    pub async fn enqueue<T: Job>(&self, _job: T) -> Result<String> {
        Ok("job_id".to_string())
    }
    
    pub async fn enqueue_delayed<T: Job>(&self, _job: T, _delay: std::time::Duration) -> Result<String> {
        Ok("delayed_job_id".to_string())
    }
    
    pub async fn enqueue_batch<T: Job>(&self, _jobs: Vec<T>) -> Result<Vec<String>> {
        Ok(vec!["job1".to_string(), "job2".to_string()])
    }
}

#[async_trait::async_trait]
pub trait Job: Send + Sync + serde::Serialize + serde::de::DeserializeOwned {
    type Output;
    async fn execute(self) -> Self::Output;
}
```

## Worker Configuration

Set up workers to process jobs:

```rust
use oxidite::prelude::*;
use oxidite_queue::{Worker, Queue};

async fn start_workers() -> Result<()> {
    let queue = get_queue("emails").await?;
    
    // Create a worker
    let mut worker = Worker::new(queue);
    
    // Configure worker settings
    worker
        .set_concurrency(5)  // Process up to 5 jobs concurrently
        .set_poll_interval(std::time::Duration::from_millis(100))  // Poll every 100ms
        .set_max_retries(3)  // Retry failed jobs up to 3 times
        .set_timeout(std::time::Duration::from_secs(30));  // Timeout after 30 seconds
    
    // Add error handling
    worker.on_error(|job_id, error| {
        eprintln!("Job {} failed: {}", job_id, error);
        // In a real app, log to monitoring system
    });
    
    // Start processing jobs
    worker.start().await?;
    
    Ok(())
}

// Graceful shutdown example
async fn graceful_shutdown_worker() -> Result<()> {
    let queue = get_queue("emails").await?;
    let mut worker = Worker::new(queue);
    
    worker.set_concurrency(3);
    
    // Handle shutdown signal
    let shutdown_signal = tokio::signal::ctrl_c();
    
    tokio::select! {
        result = worker.start() => {
            result?;
        }
        _ = shutdown_signal => {
            println!("Shutdown signal received, stopping worker...");
            worker.stop().await?;
            println!("Worker stopped gracefully");
        }
    }
    
    Ok(())
}
```

## Job Monitoring

Monitor job queues and their status:

```rust
use oxidite::prelude::*;

async fn monitor_jobs() -> Result<()> {
    let queue = get_queue("emails").await?;
    
    // Get queue statistics
    let stats = queue.stats().await?;
    println!("Queue Stats:");
    println!("  Pending: {}", stats.pending);
    println!("  Running: {}", stats.running);
    println!("  Completed: {}", stats.completed);
    println!("  Failed: {}", stats.failed);
    
    // Get specific job status
    let job_status = queue.get_job_status("some-job-id").await?;
    println!("Job Status: {:?}", job_status);
    
    // List recent jobs
    let recent_jobs = queue.list_recent_jobs(10).await?;
    for job in recent_jobs {
        println!("Recent Job: {} - {}", job.id, job.status);
    }
    
    Ok(())
}

pub struct QueueStats {
    pub pending: u64,
    pub running: u64,
    pub completed: u64,
    pub failed: u64,
}

impl Queue {
    pub async fn stats(&self) -> Result<QueueStats> {
        Ok(QueueStats {
            pending: 5,
            running: 2,
            completed: 50,
            failed: 1,
        })
    }
    
    pub async fn get_job_status(&self, _job_id: &str) -> Result<JobStatus> {
        Ok(JobStatus::Completed)
    }
    
    pub async fn list_recent_jobs(&self, _limit: usize) -> Result<Vec<ListedJob>> {
        Ok(vec![
            ListedJob { id: "job1".to_string(), status: JobStatus::Completed },
            ListedJob { id: "job2".to_string(), status: JobStatus::Pending },
        ])
    }
}

pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

pub struct ListedJob {
    pub id: String,
    pub status: JobStatus,
}
```

## Retry Logic and Error Handling

Implement robust error handling and retry mechanisms:

```rust
use oxidite::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct RobustJob {
    pub attempt_number: u32,
    pub data: String,
}

#[async_trait::async_trait]
impl Job for RobustJob {
    type Output = Result<(), JobError>;
    
    async fn execute(self) -> Self::Output {
        // Simulate a job that might fail occasionally
        if self.attempt_number < 2 && rand::random::<bool>() {
            return Err(JobError::TemporaryFailure(
                "Random failure for demonstration".to_string()
            ));
        }
        
        // Job succeeded
        println!("Job executed successfully on attempt {}", self.attempt_number);
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum JobError {
    TemporaryFailure(String),
    PermanentFailure(String),
    ValidationError(String),
}

impl std::fmt::Display for JobError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobError::TemporaryFailure(msg) => write!(f, "Temporary failure: {}", msg),
            JobError::PermanentFailure(msg) => write!(f, "Permanent failure: {}", msg),
            JobError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for JobError {}

// Retry strategy
pub struct RetryStrategy {
    pub max_attempts: u32,
    pub base_delay: std::time::Duration,
    pub backoff_multiplier: f64,
}

impl RetryStrategy {
    pub fn calculate_delay(&self, attempt: u32) -> std::time::Duration {
        let multiplier = self.backoff_multiplier.powf(attempt as f64 - 1.0);
        let delay_ms = (self.base_delay.as_millis() as f64 * multiplier) as u64;
        std::time::Duration::from_millis(delay_ms.min(300_000)) // Cap at 5 minutes
    }
}

// Example usage with retry strategy
async fn execute_with_retry(job: RobustJob, strategy: &RetryStrategy) -> Result<()> {
    let mut attempt = 1;
    
    loop {
        match job.clone().execute().await {
            Ok(_) => return Ok(()),
            Err(JobError::PermanentFailure(_)) => {
                eprintln!("Permanent failure, not retrying");
                return Err(Error::InternalServerError("Permanent job failure".to_string()));
            }
            Err(JobError::TemporaryFailure(_)) | Err(JobError::ValidationError(_)) => {
                if attempt >= strategy.max_attempts {
                    eprintln!("Max attempts reached, failing permanently");
                    return Err(Error::InternalServerError("Job failed after max retries".to_string()));
                }
                
                let delay = strategy.calculate_delay(attempt);
                println!("Attempt {} failed, retrying in {:?}", attempt, delay);
                
                tokio::time::sleep(delay).await;
                attempt += 1;
            }
        }
    }
}
```

## Scheduled Jobs

Schedule jobs to run at specific times:

```rust
use oxidite::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct ScheduledReportJob {
    pub report_type: String,
    pub recipient: String,
    pub schedule_time: String, // ISO 8601 formatted
}

#[async_trait::async_trait]
impl Job for ScheduledReportJob {
    type Output = Result<(), String>;
    
    async fn execute(self) -> Self::Output {
        println!("Generating {} report for {}", self.report_type, self.recipient);
        
        // Generate and send report
        // In a real app, this would connect to reporting systems
        
        Ok(())
    }
}

// Schedule recurring jobs
pub struct Scheduler {
    queue: Queue,
}

impl Scheduler {
    pub fn new(queue: Queue) -> Self {
        Self { queue }
    }
    
    pub async fn schedule_daily_report(&self, recipient: String) -> Result<()> {
        // Calculate next occurrence (tomorrow at 9 AM)
        let tomorrow = chrono::Local::now()
            .date_naive()
            .succ_opt()
            .unwrap()
            .and_hms_opt(9, 0, 0)
            .unwrap();
        
        let job = ScheduledReportJob {
            report_type: "daily_summary".to_string(),
            recipient,
            schedule_time: tomorrow.and_utc().to_rfc3339(),
        };
        
        // Enqueue for tomorrow morning
        self.queue.enqueue_delayed(
            job,
            std::time::Duration::from_secs(24 * 3600) // 24 hours
        ).await?;
        
        Ok(())
    }
    
    pub async fn schedule_weekly_report(&self, recipient: String) -> Result<()> {
        // Schedule for next Monday at 10 AM
        let now = chrono::Local::now();
        let days_until_monday = (7 - now.weekday().num_days_from_monday()) % 7;
        let next_monday = now.date_naive()
            .with_days_added(days_until_monday as u32)
            .and_hms_opt(10, 0, 0)
            .unwrap();
        
        let job = ScheduledReportJob {
            report_type: "weekly_summary".to_string(),
            recipient,
            schedule_time: next_monday.and_utc().to_rfc3339(),
        };
        
        let delay_seconds = (next_monday.and_utc() - chrono::Utc::now()).num_seconds() as u64;
        self.queue.enqueue_delayed(job, std::time::Duration::from_secs(delay_seconds)).await?;
        
        Ok(())
    }
}
```

## Job Dependencies

Chain jobs that depend on each other:

```rust
use oxidite::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct ProcessUserDataJob {
    pub user_id: String,
}

#[async_trait::async_trait]
impl Job for ProcessUserDataJob {
    type Output = Result<String, String>; // Returns processed data ID
    
    async fn execute(self) -> Self::Output {
        println!("Processing user data for: {}", self.user_id);
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        Ok(format!("processed_data_{}", self.user_id))
    }
}

#[derive(Serialize, Deserialize)]
pub struct SendNotificationJob {
    pub user_id: String,
    pub processed_data_id: String,
}

#[async_trait::async_trait]
impl Job for SendNotificationJob {
    type Output = Result<(), String>;
    
    async fn execute(self) -> Self::Output {
        println!("Sending notification to {} about {}", 
                 self.user_id, self.processed_data_id);
        Ok(())
    }
}

// Chain jobs with dependencies
pub struct JobChainer {
    queue: Queue,
}

impl JobChainer {
    pub fn new(queue: Queue) -> Self {
        Self { queue }
    }
    
    pub async fn process_user_with_notification(&self, user_id: String) -> Result<()> {
        // First job processes user data and returns an ID
        let process_job = ProcessUserDataJob {
            user_id: user_id.clone(),
        };
        
        let process_job_id = self.queue.enqueue(process_job).await?;
        
        // Second job waits for the first to complete
        // In a real implementation, this would use job callbacks or a workflow system
        tokio::spawn({
            let queue = self.queue.clone();
            let user_id_clone = user_id;
            async move {
                // Poll for job completion (simplified)
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                
                let notification_job = SendNotificationJob {
                    user_id: user_id_clone,
                    processed_data_id: format!("processed_data_{}", user_id_clone),
                };
                
                queue.enqueue(notification_job).await.ok();
            }
        });
        
        Ok(())
    }
}
```

## Performance Considerations

Optimize job processing for performance:

```rust
use oxidite::prelude::*;

pub struct JobProcessorConfig {
    pub concurrency: usize,
    pub batch_size: usize,
    pub memory_limit_mb: usize,
    pub timeout_seconds: u64,
}

impl JobProcessorConfig {
    pub fn production_defaults() -> Self {
        Self {
            concurrency: num_cpus::get(), // Use all CPU cores
            batch_size: 10,               // Process jobs in batches
            memory_limit_mb: 512,         // Limit memory usage
            timeout_seconds: 300,         // 5 minute timeout
        }
    }
    
    pub fn development_defaults() -> Self {
        Self {
            concurrency: 2,
            batch_size: 5,
            memory_limit_mb: 128,
            timeout_seconds: 60,
        }
    }
}

// Memory-efficient job processor
pub struct MemoryEfficientProcessor<J: Job> {
    queue: Queue,
    config: JobProcessorConfig,
    phantom: std::marker::PhantomData<J>,
}

impl<J: Job> MemoryEfficientProcessor<J> {
    pub fn new(queue: Queue, config: JobProcessorConfig) -> Self {
        Self {
            queue,
            config,
            phantom: std::marker::PhantomData,
        }
    }
    
    pub async fn process_batch(&self) -> Result<()> {
        // Fetch and process jobs in memory-conscious way
        for _ in 0..self.config.batch_size {
            // Process individual job with memory limits
            // Implementation would handle memory monitoring
        }
        
        Ok(())
    }
}
```

## Error Recovery and Monitoring

Implement robust error recovery:

```rust
use oxidite::prelude::*;

pub struct JobRecoverySystem {
    dead_letter_queue: Queue,
    monitoring_client: MonitoringClient,
}

impl JobRecoverySystem {
    pub fn new(dead_letter_queue: Queue, monitoring_client: MonitoringClient) -> Self {
        Self {
            dead_letter_queue,
            monitoring_client,
        }
    }
    
    pub async fn handle_failed_job<T: Job>(&self, job: T, error: JobError) -> Result<()> {
        // Log the error
        self.monitoring_client.log_error(&error.to_string()).await;
        
        // Move to dead letter queue for manual inspection
        self.dead_letter_queue.enqueue(DeadLetterJob {
            original_job: serde_json::to_value(&job)?,
            error: error.to_string(),
            failed_at: chrono::Utc::now().to_rfc3339(),
        }).await?;
        
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct DeadLetterJob {
    pub original_job: serde_json::Value,
    pub error: String,
    pub failed_at: String,
}

pub struct MonitoringClient;

impl MonitoringClient {
    pub async fn log_error(&self, _error: &str) {
        // In a real app, send to monitoring system like Sentry, Datadog, etc.
        println!("Error logged: {}", _error);
    }
}
```

## Integration with HTTP Handlers

Trigger jobs from HTTP requests:

```rust
use oxidite::prelude::*;

#[derive(Deserialize)]
pub struct EmailRequest {
    pub to: String,
    pub subject: String,
    pub body: String,
}

// HTTP handler that triggers a background job
async fn send_email_handler(
    Json(request): Json<EmailRequest>,
    State(queue): State<Queue>
) -> Result<Response> {
    let job = SendEmailJob {
        recipient: request.to,
        subject: request.subject,
        body: request.body,
    };
    
    let job_id = queue.enqueue(job).await
        .map_err(|e| Error::InternalServerError(format!("Failed to queue email: {}", e)))?;
    
    Ok(Response::json(serde_json::json!({
        "status": "queued",
        "job_id": job_id,
        "message": "Email queued for sending"
    })))
}

// Check job status endpoint
async fn check_job_status(
    Path(job_id): Path<String>,
    State(queue): State<Queue>
) -> Result<Response> {
    let status = queue.get_job_status(&job_id).await
        .map_err(|e| Error::InternalServerError(format!("Failed to get job status: {}", e)))?;
    
    Ok(Response::json(serde_json::json!({
        "job_id": job_id,
        "status": match status {
            JobStatus::Pending => "pending",
            JobStatus::Running => "running", 
            JobStatus::Completed => "completed",
            JobStatus::Failed => "failed",
            JobStatus::Cancelled => "cancelled",
        }
    })))
}
```

## Summary

Background jobs in Oxidite provide:

- **Asynchronous Processing**: Handle long-running tasks without blocking requests
- **Reliability**: Built-in retry logic and error handling
- **Scalability**: Concurrency controls and resource management
- **Monitoring**: Job status tracking and statistics
- **Scheduling**: Delayed execution and recurring tasks
- **Integration**: Easy to trigger from HTTP handlers

Jobs are essential for building responsive applications that need to handle time-consuming operations while keeping the user experience smooth.