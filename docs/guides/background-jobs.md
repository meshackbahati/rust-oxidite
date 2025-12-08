# Background Jobs Guide

Learn how to process background jobs, schedule cron tasks, and handle async operations with Oxidite.

## Installation

```toml
[dependencies]
oxidite = { version = "1.0", features = ["queue"] }
```

## Quick Start

### Define a Job

```rust
use oxidite_queue::{Job, JobResult};
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
    async fn handle(&self) -> JobResult {
        // Your job logic
        println!("Sending email to: {}", self.to);
        
        // Simulate email sending
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        Ok(())
    }
}
```

### Enqueue Jobs

```rust
use oxidite::prelude::*;
use oxidite_queue::{Queue, Job};

#[tokio::main]
async fn main() -> Result<()> {
    // Create queue (Memory or Redis)
    let queue = Queue::new_redis("redis://127.0.0.1").await?;
    
    // Enqueue a job
    let job = SendEmailJob {
        to: "user@example.com".to_string(),
        subject: "Welcome!".to_string(),
        body: "Thanks for signing up!".to_string(),
    };
    
    queue.dispatch(job).await?;
    
    Ok(())
}
```

### Start Workers

You can start workers using the CLI:
```bash
oxidite queue work
```

Or programmatically:
```rust
use std::sync::Arc;
use oxidite_queue::Worker;

// Start worker
let worker = Worker::new(Arc::new(queue))
    .concurrency(4);  // 4 concurrent workers

worker.run().await;
```

## Cron Jobs

Scheduling cron jobs is not yet implemented in the `oxidite-queue` crate, but it is on the roadmap.

## Retry Logic

Jobs automatically retry on failure with exponential backoff. You can configure this when dispatching a job.

```rust
use std::time::Duration;

queue.dispatch(job)
    .with_max_retries(5)
    .with_backoff(Duration::from_secs(60))
    .await?;
```

## Dead Letter Queue

Jobs that fail after all retries go to the Dead Letter Queue. You can manage these using the CLI.

```bash
# List failed jobs
oxidite queue dlq

# Retry a specific job from DLQ
oxidite queue retry <job_id>
```

## CLI Commands

Manage jobs with the CLI:

```bash
# Start worker
oxidite queue work --workers 4

# View statistics
oxidite queue list

# View dead letter queue
oxidite queue dlq

# Clear pending jobs
oxidite queue clear
```
