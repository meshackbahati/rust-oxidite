use async_trait::async_trait;
use sqlx::{PgPool, Row};
use crate::{QueueBackend, job::JobWrapper, Result, QueueError};

/// PostgreSQL queue backend
pub struct PostgresBackend {
    pool: PgPool,
    table_name: String,
    dlq_table_name: String,
}

impl PostgresBackend {
    pub async fn new(pool: PgPool, table_name: &str) -> Result<Self> {
        let backend = Self {
            pool,
            table_name: table_name.to_string(),
            dlq_table_name: format!("{}_dlq", table_name),
        };
        
        // Initialize tables if they don't exist
        backend.init_tables().await?;
        
        Ok(backend)
    }

    async fn init_tables(&self) -> Result<()> {
        // Create main queue table
        sqlx::query(&format!(
            r#"CREATE TABLE IF NOT EXISTS {} (
                id TEXT PRIMARY KEY,
                payload JSONB NOT NULL,
                priority INTEGER DEFAULT 0,
                attempts INTEGER DEFAULT 0,
                max_attempts INTEGER DEFAULT 3,
                scheduled_at TIMESTAMP WITH TIME ZONE,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                status VARCHAR(20) DEFAULT 'pending'
            )"#,
            self.table_name
        ))
        .execute(&self.pool)
        .await
        .map_err(|e| QueueError::BackendError(e.to_string()))?;

        // Create dead letter queue table
        sqlx::query(&format!(
            r#"CREATE TABLE IF NOT EXISTS {} (
                id TEXT PRIMARY KEY,
                payload JSONB NOT NULL,
                priority INTEGER DEFAULT 0,
                attempts INTEGER DEFAULT 0,
                error TEXT,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                status VARCHAR(20) DEFAULT 'dead_letter'
            )"#,
            self.dlq_table_name
        ))
        .execute(&self.pool)
        .await
        .map_err(|e| QueueError::BackendError(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl QueueBackend for PostgresBackend {
    async fn enqueue(&self, mut job: JobWrapper) -> Result<()> {
        job.status = crate::job::JobStatus::Pending;
        
        sqlx::query(&format!(
            r#"INSERT INTO {} (id, payload, priority, attempts, max_attempts, scheduled_at, status)
               VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
            self.table_name
        ))
        .bind(&job.id)
        .bind(serde_json::to_value(&job)?)
        .bind(job.priority)
        .bind(job.attempts as i32)
        .bind(job.max_retries as i32)
        .bind(job.scheduled_at.map(|t| chrono::DateTime::<chrono::Utc>::from_timestamp(t, 0)))
        .bind("pending")
        .execute(&self.pool)
        .await
        .map_err(|e| QueueError::BackendError(e.to_string()))?;

        Ok(())
    }

    async fn dequeue(&self) -> Result<Option<JobWrapper>> {
        // Get the next job that is pending and ready to be processed
        let row = sqlx::query(&format!(
            r#"UPDATE {}
               SET status = 'running', attempts = attempts + 1, updated_at = NOW()
               WHERE id = (
                   SELECT id FROM {}
                   WHERE status = 'pending'
                   AND (scheduled_at IS NULL OR scheduled_at <= NOW())
                   ORDER BY priority DESC, created_at ASC
                   LIMIT 1
                   FOR UPDATE SKIP LOCKED
               )
               RETURNING payload"#,
            self.table_name, self.table_name
        ))
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| QueueError::BackendError(e.to_string()))?;

        if let Some(row) = row {
            let payload: serde_json::Value = row.try_get("payload")
                .map_err(|e| QueueError::BackendError(e.to_string()))?;
            let mut job: JobWrapper = serde_json::from_value(payload)
                .map_err(|e| QueueError::SerializationError(e))?;
            job.status = crate::job::JobStatus::Running;
            job.attempts += 1;
            Ok(Some(job))
        } else {
            Ok(None)
        }
    }

    async fn complete(&self, job_id: &str) -> Result<()> {
        sqlx::query(&format!(
            r#"DELETE FROM {} WHERE id = $1"#,
            self.table_name
        ))
        .bind(job_id)
        .execute(&self.pool)
        .await
        .map_err(|e| QueueError::BackendError(e.to_string()))?;

        Ok(())
    }

    async fn fail(&self, job_id: &str, error: String) -> Result<()> {
        // Check if max attempts reached
        let row = sqlx::query(&format!(
            r#"SELECT attempts, max_retries FROM {} WHERE id = $1"#,
            self.table_name
        ))
        .bind(job_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| QueueError::BackendError(e.to_string()))?;

        if let Some(row) = row {
            let attempts: i32 = row.try_get("attempts")
                .map_err(|e| QueueError::BackendError(e.to_string()))?;
            let max_retries: i32 = row.try_get("max_retries")
                .map_err(|e| QueueError::BackendError(e.to_string()))?;

            if attempts >= max_retries {
                // Move to dead letter queue
                self.move_to_dead_letter_with_error(job_id, error).await?;
            } else {
                // Update status back to pending for retry
                sqlx::query(&format!(
                    r#"UPDATE {} SET status = 'pending', updated_at = NOW() WHERE id = $1"#,
                    self.table_name
                ))
                .bind(job_id)
                .execute(&self.pool)
                .await
                .map_err(|e| QueueError::BackendError(e.to_string()))?;
            }
        }

        Ok(())
    }

    async fn retry(&self, job: JobWrapper) -> Result<()> {
        // First, delete the existing job
        sqlx::query(&format!(
            r#"DELETE FROM {} WHERE id = $1"#,
            self.table_name
        ))
        .bind(&job.id)
        .execute(&self.pool)
        .await
        .map_err(|e| QueueError::BackendError(e.to_string()))?;

        // Then re-enqueue with reset attempts
        let mut job = job;
        job.attempts = 0;
        job.status = crate::job::JobStatus::Pending;
        
        self.enqueue(job).await
    }

    async fn move_to_dead_letter(&self, job: JobWrapper) -> Result<()> {
        self.move_to_dead_letter_with_error(&job.id, job.error.unwrap_or_else(|| "Unknown error".to_string())).await
    }

    async fn list_dead_letter(&self) -> Result<Vec<JobWrapper>> {
        let rows = sqlx::query(&format!(
            r#"SELECT payload FROM {} ORDER BY created_at DESC"#,
            self.dlq_table_name
        ))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| QueueError::BackendError(e.to_string()))?;

        let mut jobs = Vec::new();
        for row in rows {
            let payload: serde_json::Value = row.try_get("payload")
                .map_err(|e| QueueError::BackendError(e.to_string()))?;
            if let Ok(job) = serde_json::from_value::<JobWrapper>(payload) {
                jobs.push(job);
            }
        }

        Ok(jobs)
    }

    async fn retry_from_dead_letter(&self, job_id: &str) -> Result<()> {
        // Get job from DLQ
        let row = sqlx::query(&format!(
            r#"SELECT payload FROM {} WHERE id = $1"#,
            self.dlq_table_name
        ))
        .bind(job_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| QueueError::BackendError(e.to_string()))?;

        if let Some(row) = row {
            let payload: serde_json::Value = row.try_get("payload")
                .map_err(|e| QueueError::BackendError(e.to_string()))?;
            let mut job: JobWrapper = serde_json::from_value(payload)
                .map_err(|e| QueueError::SerializationError(e))?;

            // Delete from DLQ
            sqlx::query(&format!(
                r#"DELETE FROM {} WHERE id = $1"#,
                self.dlq_table_name
            ))
            .bind(job_id)
            .execute(&self.pool)
            .await
            .map_err(|e| QueueError::BackendError(e.to_string()))?;

            // Reset and enqueue to main queue
            job.status = crate::job::JobStatus::Pending;
            job.attempts = 0;
            job.error = None;

            self.enqueue(job).await?;
        }

        Ok(())
    }
}

impl PostgresBackend {
    async fn move_to_dead_letter_with_error(&self, job_id: &str, error: String) -> Result<()> {
        // Get the job from the main queue
        let row = sqlx::query(&format!(
            r#"SELECT payload, priority, attempts FROM {} WHERE id = $1"#,
            self.table_name
        ))
        .bind(job_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| QueueError::BackendError(e.to_string()))?;

        if let Some(row) = row {
            let payload: serde_json::Value = row.try_get("payload")
                .map_err(|e| QueueError::BackendError(e.to_string()))?;
            let priority: i32 = row.try_get("priority")
                .map_err(|e| QueueError::BackendError(e.to_string()))?;
            let attempts: i32 = row.try_get("attempts")
                .map_err(|e| QueueError::BackendError(e.to_string()))?;

            // Insert into dead letter queue
            sqlx::query(&format!(
                r#"INSERT INTO {} (id, payload, priority, attempts, error)
                   VALUES ($1, $2, $3, $4, $5)"#,
                self.dlq_table_name
            ))
            .bind(job_id)
            .bind(&payload)
            .bind(priority)
            .bind(attempts)
            .bind(error)
            .execute(&self.pool)
            .await
            .map_err(|e| QueueError::BackendError(e.to_string()))?;

            // Remove from main queue
            sqlx::query(&format!(
                r#"DELETE FROM {} WHERE id = $1"#,
                self.table_name
            ))
            .bind(job_id)
            .execute(&self.pool)
            .await
            .map_err(|e| QueueError::BackendError(e.to_string()))?;
        }

        Ok(())
    }
}