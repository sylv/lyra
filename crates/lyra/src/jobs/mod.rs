use crate::entities::{files, jobs as jobs_entity};
use anyhow::Context;
use chrono::Duration;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, prelude::Expr,
};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Notify;
use tokio::time::sleep;

pub mod handlers;
pub mod registry;

const BATCH_SIZE: u64 = 100;
const EMPTY_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_secs(60 * 5);

pub struct JobExecutionPolicy {
    backoff: Vec<Duration>,
}

impl Default for JobExecutionPolicy {
    fn default() -> Self {
        Self {
            backoff: vec![Duration::days(1), Duration::days(7), Duration::days(30)],
        }
    }
}

impl JobExecutionPolicy {
    pub fn max_attempts(&self) -> i64 {
        self.backoff.len() as i64
    }

    fn next_retry_at(&self, attempt_count: i64) -> Option<i64> {
        if attempt_count >= self.max_attempts() {
            None
        } else {
            Some((chrono::Utc::now() + self.backoff[attempt_count as usize - 1]).timestamp())
        }
    }
}

#[async_trait::async_trait]
pub trait JobHandler: Send + Sync {
    fn job_type(&self) -> &'static str;

    fn execution_policy(&self) -> JobExecutionPolicy {
        JobExecutionPolicy::default()
    }

    /// Optional job-specific filter (for example, by file metadata).
    /// Returning `None` means "all files".
    fn filter_condition(&self) -> Option<Condition> {
        None
    }

    fn final_condition(&self, now: i64) -> Condition {
        let quoted_job_type = escape_sql_literal(self.job_type());
        let max_attempts = self.execution_policy().max_attempts();
        let no_job_sql = format!(
            "NOT EXISTS (
                SELECT 1
                FROM jobs j
                WHERE j.file_id = files.id
                  AND j.job_type = '{quoted_job_type}'
            )"
        );
        let retryable_job_sql = format!(
            "EXISTS (
                SELECT 1
                FROM jobs j
                WHERE j.file_id = files.id
                  AND j.job_type = '{quoted_job_type}'
                  AND j.status = {error_status}
                  AND j.attempt_count < {max_attempts}
                  AND j.next_retry_at IS NOT NULL
                  AND j.next_retry_at <= {now}
            )",
            error_status = jobs_entity::JobStatus::Error as i64
        );

        let base_condition = Condition::all()
            .add(files::Column::UnavailableAt.is_null())
            .add(files::Column::CorruptedAt.is_null())
            .add(
                Condition::any()
                    .add(Expr::cust(no_job_sql))
                    .add(Expr::cust(retryable_job_sql)),
            );

        if let Some(filter_condition) = self.filter_condition() {
            base_condition.add(filter_condition)
        } else {
            base_condition
        }
    }

    /// Called before execution when a previous job row exists for this file and job type.
    async fn cleanup(&self, _pool: &DatabaseConnection, _file_id: i64) -> anyhow::Result<()> {
        Ok(())
    }

    async fn execute(&self, pool: &DatabaseConnection, file_id: i64) -> anyhow::Result<()>;
}

fn escape_sql_literal(value: &str) -> String {
    value.replace('\'', "''")
}

pub async fn find_pending_file_ids(
    pool: &DatabaseConnection,
    condition: Condition,
    limit: u64,
) -> anyhow::Result<Vec<i64>> {
    Ok(files::Entity::find()
        .select_only()
        .column(files::Column::Id)
        .filter(condition)
        .order_by_asc(files::Column::Id)
        .limit(limit)
        .into_tuple()
        .all(pool)
        .await?)
}

pub async fn count_pending_files(
    pool: &DatabaseConnection,
    condition: Condition,
) -> anyhow::Result<i64> {
    let count = files::Entity::find().filter(condition).count(pool).await?;
    Ok(i64::try_from(count).context("pending file count overflowed i64")?)
}

pub struct JobManager {
    handler: Arc<dyn JobHandler>,
    database: DatabaseConnection,
    wake_signal: Arc<Notify>,
}

#[async_trait::async_trait]
pub trait JobRunner: Send + Sync {
    fn job_type(&self) -> &'static str;
    async fn start_thread(&self) -> anyhow::Result<()>;
}

#[async_trait::async_trait]
impl JobRunner for JobManager {
    fn job_type(&self) -> &'static str {
        self.handler.job_type()
    }

    async fn start_thread(&self) -> anyhow::Result<()> {
        JobManager::start_thread(self).await
    }
}

impl JobManager {
    pub fn new(
        handler: Arc<dyn JobHandler>,
        database: DatabaseConnection,
        wake_signal: Arc<Notify>,
    ) -> Self {
        Self {
            handler,
            database,
            wake_signal,
        }
    }

    pub async fn start_thread(&self) -> anyhow::Result<()> {
        loop {
            let now = chrono::Utc::now().timestamp();
            let batch = find_pending_file_ids(
                &self.database,
                self.handler.final_condition(now),
                BATCH_SIZE,
            )
            .await
            .with_context(|| {
                format!(
                    "failed to find batch for job type {}",
                    self.handler.job_type()
                )
            })?;

            if batch.is_empty() {
                tokio::select! {
                    _ = self.wake_signal.notified() => {},
                    _ = sleep(EMPTY_POLL_INTERVAL) => {},
                }
                continue;
            }

            for file_id in batch {
                self.run_job_for_file(file_id).await?;
            }
        }
    }

    async fn run_job_for_file(&self, file_id: i64) -> anyhow::Result<()> {
        let policy = self.handler.execution_policy();
        let now = chrono::Utc::now().timestamp();

        let existing = jobs_entity::Entity::find()
            .filter(jobs_entity::Column::JobType.eq(self.handler.job_type()))
            .filter(jobs_entity::Column::FileId.eq(file_id))
            .one(&self.database)
            .await
            .with_context(|| {
                format!(
                    "failed to load existing job row for type={} file_id={file_id}",
                    self.handler.job_type()
                )
            })?;

        if existing.is_some() {
            self.handler
                .cleanup(&self.database, file_id)
                .await
                .with_context(|| {
                    format!(
                        "cleanup failed for job type={} file_id={file_id}",
                        self.handler.job_type()
                    )
                })?;
        }

        let start = Instant::now();
        tracing::info!(
            job_type = %self.handler.job_type(),
            file_id,
            "executing job"
        );

        match self.handler.execute(&self.database, file_id).await {
            Ok(()) => {
                tracing::debug!(
                    job_type = %self.handler.job_type(),
                    file_id,
                    elapsed = ?start.elapsed(),
                    "finished job"
                );

                self.persist_success(file_id, now, existing).await?;
            }
            Err(error) => {
                let prior_attempts = existing.as_ref().map_or(0, |row| row.attempt_count);
                let attempt_count = prior_attempts + 1;
                let next_retry_at = policy.next_retry_at(attempt_count);

                tracing::warn!(
                    job_type = %self.handler.job_type(),
                    file_id,
                    attempt_count,
                    next_retry_at,
                    error = %error,
                    "job execution failed"
                );

                self.persist_error(file_id, now, existing, attempt_count, next_retry_at, &error)
                    .await?;
            }
        }

        Ok(())
    }

    async fn persist_success(
        &self,
        file_id: i64,
        now: i64,
        existing: Option<jobs_entity::Model>,
    ) -> anyhow::Result<()> {
        if let Some(existing) = existing {
            jobs_entity::ActiveModel {
                id: Set(existing.id),
                status: Set(jobs_entity::JobStatus::Success),
                attempt_count: Set(0),
                next_retry_at: Set(None),
                last_error_message: Set(None),
                last_run_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            }
            .update(&self.database)
            .await?;
        } else {
            jobs_entity::ActiveModel {
                job_type: Set(self.handler.job_type().to_string()),
                file_id: Set(file_id),
                status: Set(jobs_entity::JobStatus::Success),
                attempt_count: Set(0),
                next_retry_at: Set(None),
                last_error_message: Set(None),
                last_run_at: Set(now),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            }
            .insert(&self.database)
            .await?;
        }

        Ok(())
    }

    async fn persist_error(
        &self,
        file_id: i64,
        now: i64,
        existing: Option<jobs_entity::Model>,
        attempt_count: i64,
        next_retry_at: Option<i64>,
        error: &anyhow::Error,
    ) -> anyhow::Result<()> {
        if let Some(existing) = existing {
            jobs_entity::ActiveModel {
                id: Set(existing.id),
                status: Set(jobs_entity::JobStatus::Error),
                attempt_count: Set(attempt_count),
                next_retry_at: Set(next_retry_at),
                last_error_message: Set(Some(error.to_string())),
                last_run_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            }
            .update(&self.database)
            .await?;
        } else {
            jobs_entity::ActiveModel {
                job_type: Set(self.handler.job_type().to_string()),
                file_id: Set(file_id),
                status: Set(jobs_entity::JobStatus::Error),
                attempt_count: Set(attempt_count),
                next_retry_at: Set(next_retry_at),
                last_error_message: Set(Some(error.to_string())),
                last_run_at: Set(now),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            }
            .insert(&self.database)
            .await?;
        }

        Ok(())
    }
}
