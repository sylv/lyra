use crate::entities::{files, jobs as jobs_entity};
use anyhow::Context;
use sea_orm::{
    ActiveValue::Set,
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
    sea_query::{Expr, OnConflict, Query},
};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Notify;
use tokio::time::sleep;

pub mod handlers;
pub mod registry;

const BATCH_SIZE: u64 = 100;
const EMPTY_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_secs(60 * 5);
const DEFAULT_BACKOFF_SECONDS: &[i64] = &[24 * 60 * 60, 7 * 24 * 60 * 60, 30 * 24 * 60 * 60];

pub struct JobExecutionPolicy {
    backoff_seconds: &'static [i64],
}

impl Default for JobExecutionPolicy {
    fn default() -> Self {
        Self {
            backoff_seconds: DEFAULT_BACKOFF_SECONDS,
        }
    }
}

impl JobExecutionPolicy {
    pub fn max_attempts(&self) -> i64 {
        self.backoff_seconds.len() as i64
    }

    fn next_retry_at(&self, now: i64, attempt_count: i64) -> Option<i64> {
        if attempt_count >= self.max_attempts() {
            None
        } else {
            self.backoff_seconds
                .get(attempt_count.saturating_sub(1) as usize)
                .map(|offset| now + offset)
        }
    }
}

#[async_trait::async_trait]
pub trait JobHandler: Send + Sync {
    fn job_type(&self) -> jobs_entity::JobType;

    fn execution_policy(&self) -> JobExecutionPolicy {
        JobExecutionPolicy::default()
    }

    /// Optional job-specific filter (for example, by file metadata).
    /// Returning `None` means "all files".
    fn filter_condition(&self) -> Option<Condition> {
        None
    }

    fn final_condition(&self, now: i64) -> Condition {
        let mut base = build_pending_condition(self.job_type(), &self.execution_policy(), now);
        if let Some(filter) = self.filter_condition() {
            base = base.add(filter);
        }
        base
    }

    /// Called before execution when a previous job row exists for this file and job type.
    async fn cleanup(&self, _pool: &DatabaseConnection, _file_id: i64) -> anyhow::Result<()> {
        Ok(())
    }

    async fn execute(&self, pool: &DatabaseConnection, file_id: i64) -> anyhow::Result<()>;
}

pub fn build_pending_condition(
    job_type: jobs_entity::JobType,
    policy: &JobExecutionPolicy,
    now: i64,
) -> Condition {
    let no_job_row_for_type = files::Column::Id.not_in_subquery(
        Query::select()
            .column(jobs_entity::Column::FileId)
            .from(jobs_entity::Entity)
            .and_where(Expr::col(jobs_entity::Column::JobType).eq(job_type))
            .to_owned(),
    );

    let retryable_job_row_for_type = files::Column::Id.in_subquery(
        Query::select()
            .column(jobs_entity::Column::FileId)
            .from(jobs_entity::Entity)
            .and_where(Expr::col(jobs_entity::Column::JobType).eq(job_type))
            .and_where(Expr::col(jobs_entity::Column::Status).eq(jobs_entity::JobStatus::Error))
            .and_where(Expr::col(jobs_entity::Column::AttemptCount).lt(policy.max_attempts()))
            .and_where(Expr::col(jobs_entity::Column::NextRetryAt).is_not_null())
            .and_where(Expr::col(jobs_entity::Column::NextRetryAt).lte(now))
            .to_owned(),
    );

    Condition::all()
        .add(files::Column::UnavailableAt.is_null())
        .add(files::Column::CorruptedAt.is_null())
        .add(
            Condition::any()
                .add(no_job_row_for_type)
                .add(retryable_job_row_for_type),
        )
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

    pub fn job_type(&self) -> jobs_entity::JobType {
        self.handler.job_type()
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
                    "failed to find batch for job type {:?}",
                    self.handler.job_type()
                )
            })?;

            if batch.is_empty() {
                self.wait_for_work().await;
                continue;
            }

            for file_id in batch {
                self.run_job_for_file(file_id).await?;
            }
        }
    }

    async fn wait_for_work(&self) {
        tokio::select! {
            _ = self.wake_signal.notified() => {},
            _ = sleep(EMPTY_POLL_INTERVAL) => {},
        }
    }

    async fn run_job_for_file(&self, file_id: i64) -> anyhow::Result<()> {
        let job_type = self.handler.job_type();
        let policy = self.handler.execution_policy();
        let now = chrono::Utc::now().timestamp();

        let existing = jobs_entity::Entity::find()
            .filter(jobs_entity::Column::JobType.eq(job_type))
            .filter(jobs_entity::Column::FileId.eq(file_id))
            .one(&self.database)
            .await
            .with_context(|| {
                format!(
                    "failed to load existing job row for type={:?} file_id={file_id}",
                    job_type
                )
            })?;

        if existing.is_some() {
            self.handler
                .cleanup(&self.database, file_id)
                .await
                .with_context(|| {
                    format!(
                        "cleanup failed for job type={:?} file_id={file_id}",
                        job_type
                    )
                })?;
        }

        let start = Instant::now();
        tracing::info!(job_type = ?job_type, file_id, "executing job");

        match self.handler.execute(&self.database, file_id).await {
            Ok(()) => {
                tracing::debug!(
                    job_type = ?job_type,
                    file_id,
                    elapsed = ?start.elapsed(),
                    "finished job"
                );

                self.persist_job_outcome(file_id, job_type, now, JobOutcome::success())
                    .await?;
            }
            Err(error) => {
                let prior_attempts = existing.as_ref().map_or(0, |row| row.attempt_count);
                let attempt_count = prior_attempts + 1;
                let next_retry_at = policy.next_retry_at(now, attempt_count);

                tracing::warn!(
                    job_type = ?job_type,
                    file_id,
                    attempt_count,
                    next_retry_at,
                    error = %error,
                    "job execution failed"
                );

                self.persist_job_outcome(
                    file_id,
                    job_type,
                    now,
                    JobOutcome::error(attempt_count, next_retry_at, error.to_string()),
                )
                .await?;
            }
        }

        Ok(())
    }

    async fn persist_job_outcome(
        &self,
        file_id: i64,
        job_type: jobs_entity::JobType,
        now: i64,
        outcome: JobOutcome,
    ) -> anyhow::Result<()> {
        jobs_entity::Entity::insert(jobs_entity::ActiveModel {
            job_type: Set(job_type),
            file_id: Set(file_id),
            status: Set(outcome.status),
            attempt_count: Set(outcome.attempt_count),
            next_retry_at: Set(outcome.next_retry_at),
            last_error_message: Set(outcome.last_error_message),
            last_run_at: Set(now),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        })
        .on_conflict(
            OnConflict::columns([jobs_entity::Column::JobType, jobs_entity::Column::FileId])
                .update_columns([
                    jobs_entity::Column::Status,
                    jobs_entity::Column::AttemptCount,
                    jobs_entity::Column::NextRetryAt,
                    jobs_entity::Column::LastErrorMessage,
                    jobs_entity::Column::LastRunAt,
                    jobs_entity::Column::UpdatedAt,
                ])
                .to_owned(),
        )
        .exec(&self.database)
        .await?;

        Ok(())
    }
}

struct JobOutcome {
    status: jobs_entity::JobStatus,
    attempt_count: i64,
    next_retry_at: Option<i64>,
    last_error_message: Option<String>,
}

impl JobOutcome {
    fn success() -> Self {
        Self {
            status: jobs_entity::JobStatus::Success,
            attempt_count: 0,
            next_retry_at: None,
            last_error_message: None,
        }
    }

    fn error(attempt_count: i64, next_retry_at: Option<i64>, message: String) -> Self {
        Self {
            status: jobs_entity::JobStatus::Error,
            attempt_count,
            next_retry_at,
            last_error_message: Some(message),
        }
    }
}
