use std::time::{Instant, Duration};
use colored::*;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

/// Runs a performance benchmark profile against an HTTP target
pub async fn run(url: &str, concurrency: usize, total_requests: usize) {
    println!("{}", "=========================================================".cyan());
    println!("  🚀 Oxidite Performance Profiler (v2.2.0-beta)");
    println!("  Target:      {}", url.yellow().bold());
    println!("  Concurrency: {}", concurrency.to_string().green());
    println!("  Requests:    {}", total_requests.to_string().green());
    println!("{}", "=========================================================".cyan());

    let client = reqwest::Client::new();
    let start_time = Instant::now();
    
    let (tx, mut rx) = mpsc::channel(total_requests);
    let mut tasks: Vec<JoinHandle<()>> = Vec::new();
    
    // Distribute total requests among workers
    let requests_per_worker = total_requests / concurrency;
    
    for _ in 0..concurrency {
        let tx_clone = tx.clone();
        let client_clone = client.clone();
        let url_clone = url.to_string();
        
        let handle = tokio::spawn(async move {
            for _ in 0..requests_per_worker {
                let start = Instant::now();
                let res = client_clone.get(&url_clone).send().await;
                let duration = start.elapsed();
                
                let success = match res {
                    Ok(response) => response.status().is_success(),
                    Err(_) => false,
                };
                
                let _ = tx_clone.send((duration, success)).await;
            }
        });
        tasks.push(handle);
    }
    
    drop(tx); // Close original sender so channel knows when all are done
    
    let mut total_duration = Duration::ZERO;
    let mut latencies: Vec<Duration> = Vec::new();
    let mut successes = 0;
    let mut failures = 0;
    
    while let Some((duration, success)) = rx.recv().await {
        latencies.push(duration);
        total_duration += duration;
        if success {
            successes += 1;
        } else {
            failures += 1;
        }
    }
    
    let total_elapsed = start_time.elapsed();
    
    if latencies.is_empty() {
        println!("{}", "No requests were successfully dispatched.".red());
        return;
    }
    
    // Sort to calculate percentiles
    latencies.sort();
    
    let min_latency = latencies.first().unwrap();
    let max_latency = latencies.last().unwrap();
    let avg_latency = total_duration / latencies.len() as u32;
    
    // Percentiles
    let p50 = latencies[latencies.len() * 50 / 100];
    let p90 = latencies[latencies.len() * 90 / 100];
    let p99 = latencies[latencies.len() * 99 / 100];
    
    let rps = total_requests as f64 / total_elapsed.as_secs_f64();
    
    println!("\n{}", "Benchmark Results:".bold().underline());
    println!("  Total Time:          {:.3?}", total_elapsed);
    println!("  Successful Requests: {}", successes.to_string().green());
    println!("  Failed Requests:     {}", failures.to_string().red());
    println!("  Requests/sec (RPS):  {:.2}", rps.to_string().green().bold());
    
    println!("\n{}", "Latency Metrics:".bold().underline());
    println!("  Min Latency:         {:.3?}", min_latency);
    println!("  Max Latency:         {:.3?}", max_latency);
    println!("  Avg Latency:         {:.3?}", avg_latency);
    println!("  50% Percentile (p50): {:.3?}", p50);
    println!("  90% Percentile (p90): {:.3?}", p90);
    println!("  99% Percentile (p99): {:.3?}", p99);
    println!("{}", "=========================================================".cyan());
}
