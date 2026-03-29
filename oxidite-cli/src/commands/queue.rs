use std::io::{self, Write};
use oxidite_queue::{Queue, QueueStats};

pub async fn queue_work(workers: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting queue worker with {} workers...", workers);
    
    // Initialize queue (in real app, this would come from config)
    let queue = Queue::memory();
    
    let worker = oxidite_queue::Worker::new(std::sync::Arc::new(queue))
        .worker_count(workers);
    
    println!("✅ Queue worker started. Press Ctrl+C to stop.");
    worker.start().await;
    
    Ok(())
}

pub async fn queue_list() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Queue Statistics\n");
    
    // Initialize queue
    let queue = Queue::memory();
    let stats = queue.get_stats().await;
    
    print_stats(&stats);
    
    Ok(())
}

pub async fn queue_dlq() -> Result<(), Box<dyn std::error::Error>> {
    println!("💀 Dead Letter Queue\n");
    
    // Initialize queue
    let queue = Queue::memory();
    let dlq_jobs = queue.list_dead_letter().await?;
    
    if dlq_jobs.is_empty() {
        println!("✨ No jobs in dead letter queue");
    } else {
        println!("Found {} jobs in DLQ:\n", dlq_jobs.len());
        for (i, job) in dlq_jobs.iter().enumerate() {
            println!("{}. Job ID: {}", i + 1, job.id);
            println!("   Name: {}", job.name);
            println!("   Attempts: {}", job.attempts);
            println!("   Error: {}", job.error.as_ref().unwrap_or(&"Unknown".to_string()));
            println!();
        }
    }
    
    Ok(())
}

pub async fn queue_clear() -> Result<(), Box<dyn std::error::Error>> {
    print!("⚠️  Are you sure you want to clear all pending jobs? (y/N): ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    if input.trim().to_lowercase() == "y" {
        println!("🗑️  Clearing queue...");
        let queue = Queue::memory();
        queue.clear().await?;
        println!("✅ Queue cleared");
    } else {
        println!("❌ Cancelled");
    }
    
    Ok(())
}

fn print_stats(stats: &QueueStats) {
    println!("┌─────────────────────────────────────┐");
    println!("│ Queue Statistics                    │");
    println!("├─────────────────────────────────────┤");
    println!("│ Total Enqueued:      {:>14} │", stats.total_enqueued);
    println!("│ Total Processed:     {:>14} │", stats.total_processed);
    println!("│ Total Failed:        {:>14} │", stats.total_failed);
    println!("│ Total Retried:       {:>14} │", stats.total_retried);
    println!("├─────────────────────────────────────┤");
    println!("│ Pending:             {:>14} │", stats.pending_count);
    println!("│ Running:             {:>14} │", stats.running_count);
    println!("│ Dead Letter:         {:>14} │", stats.dead_letter_count);
    println!("└─────────────────────────────────────┘");
}
