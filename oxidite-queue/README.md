# oxidite-queue

Background job queue with cron scheduling, DLQ, and retry logic.

## Installation

```toml
[dependencies]
oxidite-queue = "2.3.4"
```

## Usage

### Define a Job

```rust
use oxidite_queue::*;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct SendEmailJob {
    to: String,
    subject: String,
    body: String,
}

#[async_trait]
impl Job for SendEmailJob {
    async fn perform(&self) -> JobResult {
        // Send email logic
        send_email(&self.to, &self.subject, &self.body).await?;
        Ok(())
    }
    
    fn max_retries(&self) -> u32 {
        3
    }
}
```

### Enqueue Jobs

```rust
// Create queue
let queue = Queue::memory();

// Enqueue job
let job = JobWrapper::new(&SendEmailJob {
    to: "user@example.com".into(),
    subject: "Welcome!".into(),
    body: "Thanks for signing up!".into(),
})?;

queue.enqueue(job).await?;
```

### Cron Jobs

```rust
// Recurring job (runs daily at 9 AM)
let job = JobWrapper::new(&DailyReportJob {})?
    .with_cron("0 0 9 * * *".to_string());

queue.enqueue(job).await?;
```

### Worker

```rust
// Start worker
let worker = Worker::new(Arc::new(queue))
    .worker_count(4);

worker.start().await;
```

### Dead Letter Queue

```rust
// List failed jobs
let failed = queue.list_dead_letter().await?;

// Retry from DLQ
queue.retry_from_dead_letter(&job_id).await?;
```

### Statistics

```rust
let stats = queue.get_stats().await;
println!("Processed: {}", stats.total_processed);
println!("Failed: {}", stats.total_failed);
```

## Features

- Memory and Redis backends
- Cron job scheduling
- Retry logic with exponential backoff
- Dead letter queue
- Job statistics tracking
- Worker pool management
- Job priorities
- Delayed jobs

## License

MIT
