/// Job types and traits
pub mod job;
/// Queue backends and the Queue type
pub mod queue;
/// Worker for processing jobs
pub mod worker;
/// Queue statistics
pub mod stats;

/// Re-export of [`Job`], [`JobStatus`], [`JobResult`]
pub use job::{Job, JobStatus, JobResult};
/// Re-export of [`Queue`], [`QueueBackend`], [`MemoryBackend`]
pub use queue::{Queue, QueueBackend, MemoryBackend};
/// Redis queue backend
pub mod redis;
/// Re-export of [`RedisBackend`]
pub use crate::redis::RedisBackend;
/// PostgreSQL queue backend
pub mod postgres;
/// Re-export of [`PostgresBackend`]
pub use crate::postgres::PostgresBackend;
/// Re-export of [`Worker`]
pub use worker::Worker;
/// Re-export of [`QueueStats`], [`StatsTracker`]
pub use stats::{QueueStats, StatsTracker};

use thiserror::Error;

/// Queue errors
#[derive(Error, Debug)]
pub enum QueueError {
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    /// Job execution failure
    #[error("Job failed: {0}")]
    JobFailed(String),
    
    /// Queue is at capacity
    #[error("Queue full")]
    QueueFull,
    
    /// Backend-specific error
    #[error("Backend error: {0}")]
    BackendError(String),
}

/// Queue result type alias
pub type Result<T> = std::result::Result<T, QueueError>;
