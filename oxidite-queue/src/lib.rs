pub mod job;
pub mod queue;
pub mod worker;

pub use job::{Job, JobStatus, JobResult};
pub use queue::{Queue, QueueBackend, MemoryBackend};
pub mod redis;
pub use crate::redis::RedisBackend;
pub use worker::Worker;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum QueueError {
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Job failed: {0}")]
    JobFailed(String),
    
    #[error("Queue full")]
    QueueFull,
    
    #[error("Backend error: {0}")]
    BackendError(String),
}

pub type Result<T> = std::result::Result<T, QueueError>;
