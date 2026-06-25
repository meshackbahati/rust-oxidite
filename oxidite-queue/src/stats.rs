use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Job queue statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QueueStats {
    /// Total number of jobs enqueued
    pub total_enqueued: u64,
    /// Total number of jobs processed
    pub total_processed: u64,
    /// Total number of jobs failed
    pub total_failed: u64,
    /// Total number of jobs retried
    pub total_retried: u64,
    /// Current number of pending jobs
    pub pending_count: u64,
    /// Current number of running jobs
    pub running_count: u64,
    /// Current number of dead letter jobs
    pub dead_letter_count: u64,
}

/// Thread-safe statistics tracker
#[derive(Clone)]
pub struct StatsTracker {
    stats: Arc<RwLock<QueueStats>>,
}

impl StatsTracker {
    /// Create a new stats tracker
    pub fn new() -> Self {
        Self {
            stats: Arc::new(RwLock::new(QueueStats::default())),
        }
    }

    /// Increment the enqueued counter
    pub async fn increment_enqueued(&self) {
        let mut stats = self.stats.write().await;
        stats.total_enqueued += 1;
        stats.pending_count += 1;
    }

    /// Increment the processed counter
    pub async fn increment_processed(&self) {
        let mut stats = self.stats.write().await;
        stats.total_processed += 1;
        if stats.running_count > 0 {
            stats.running_count -= 1;
        }
    }

    /// Increment the failed counter
    pub async fn increment_failed(&self) {
        let mut stats = self.stats.write().await;
        stats.total_failed += 1;
        if stats.running_count > 0 {
            stats.running_count -= 1;
        }
    }

    /// Increment the retried counter
    pub async fn increment_retried(&self) {
        let mut stats = self.stats.write().await;
        stats.total_retried += 1;
    }

    /// Increment the dead letter counter
    pub async fn increment_dead_letter(&self) {
        let mut stats = self.stats.write().await;
        stats.dead_letter_count += 1;
    }

    /// Mark a job as running
    pub async fn mark_running(&self) {
        let mut stats = self.stats.write().await;
        if stats.pending_count > 0 {
            stats.pending_count -= 1;
        }
        stats.running_count += 1;
    }

    /// Get a snapshot of the current stats
    pub async fn get_stats(&self) -> QueueStats {
        self.stats.read().await.clone()
    }

    /// Reset all statistics to zero
    pub async fn reset(&self) {
        let mut stats = self.stats.write().await;
        *stats = QueueStats::default();
    }
}

impl Default for StatsTracker {
    fn default() -> Self {
        Self::new()
    }
}
