use crate::content_update::CONTENT_UPDATE;
use crate::entities::jobs as jobs_entity;
use crate::jobs::job::JobOutcome;
use crate::jobs::semaphore::{JobLease, JobSemaphore};
use crate::{activity::ActivityHandle, jobs::job::Job};
use anyhow::Context;
use sea_orm::{
    ActiveEnum, ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, ConnectionTrait,
    DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Select, sea_query::Query,
};
use sqlx::query;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Notify;
use tokio::time::sleep;

pub struct JobManager<J: Job> {
    job: Arc<J>,
    pool: DatabaseConnection,
    wake_signal: Arc<Notify>,
    job_semaphore: Arc<JobSemaphore>,
}

impl<J: Job> JobManager<J> {
    pub fn new(
        job: Arc<J>,
        database: DatabaseConnection,
        wake_signal: Arc<Notify>,
        job_semaphore: Arc<JobSemaphore>,
    ) -> Self {
        Self {
            job,
            pool: database,
            wake_signal,
            job_semaphore,
        }
    }

    pub fn job_kind(&self) -> jobs_entity::JobKind {
        J::JOB_KIND
    }

    pub async fn start_thread(&self) -> anyhow::Result<()> {
        loop {
            let job_lease = self.job_semaphore.acquire_lease(J::IS_HEAVY).await;
            let Some((target_id, target)) = self.try_claim_next_target().await? else {
                drop(job_lease);

                tokio::select! {
                    _ = self.wake_signal.notified() => {},
                    _ = sleep(Duration::from_secs(30)) => {},
                }

                continue;
            };

            self.run_job_for_target(target_id, target, job_lease)
                .await?;
        }
    }

    // Query for jobs that are ready to be run (not errored or waiting for a retry)
    // Does not claim jobs
    fn query_runnable_targets(&self, now: i64) -> Select<J::Entity> {
        self.job.query().filter(
            self.job
                .target_id_column()
                .not_in_subquery(blocked_job_targets_query(J::JOB_KIND, now)),
        )
    }

    // Keep selection and claim in one transaction so each loop only races on a single target.
    async fn try_claim_next_target(&self) -> anyhow::Result<Option<(String, J::Model)>> {
        let now = chrono::Utc::now().timestamp();
        let target = self
            .query_runnable_targets(now)
            .order_by_asc(self.job.target_id_column())
            .one(&self.pool)
            .await
            .with_context(|| format!("failed to query next candidate for {:?}", J::JOB_KIND))?;

        let Some(target) = target else {
            return Ok(None);
        };

        let target_id = self.job.target_id(&target);
        let job_kind_code = J::JOB_KIND.code();
        let lock_result = query!(
            r#"
            INSERT INTO jobs (
                locked_at,
                job_kind,
                target_id,
                state,
                retry_after,
                last_error_message
            )
            VALUES (?1, ?2, ?3, ?4, NULL, NULL)
            ON CONFLICT (job_kind, target_id) DO UPDATE
            SET
                state = excluded.state,
                locked_at = excluded.locked_at,
                retry_after = NULL,
                last_error_message = NULL
            WHERE
                jobs.locked_at IS NULL
                AND (jobs.retry_after IS NULL OR jobs.retry_after <= ?1);
            "#,
            now,
            job_kind_code,
            target_id,
            jobs_entity::JobState::Running as i32,
        )
        .execute(self.pool.get_sqlite_connection_pool())
        .await
        .with_context(|| {
            format!(
                "failed to claim job for target {} of {:?}",
                target_id,
                J::JOB_KIND
            )
        })?;

        if lock_result.rows_affected() == 0 {
            return Ok(None);
        }

        Ok(Some((target_id, target)))
    }

    async fn run_job_for_target(
        &self,
        target_id: String,
        target: J::Model,
        lease: JobLease,
    ) -> anyhow::Result<()> {
        let _activity = ActivityHandle::new(J::JOB_KIND);
        let policy = self.job.execution_policy();
        let start = Instant::now();

        tracing::info!(
            job_kind = ?J::JOB_KIND,
            target_id = %target_id,
            is_heavy = J::IS_HEAVY,
            "executing job"
        );

        let result = self.job.run(&self.pool, target, &lease).await;
        match result {
            Ok(JobOutcome::Complete) => {
                tracing::debug!(
                    job_kind = ?J::JOB_KIND,
                    target_id = %target_id,
                    elapsed = ?start.elapsed(),
                    "finished job"
                );

                delete_job_row(&self.pool, J::JOB_KIND, &target_id).await?;
                CONTENT_UPDATE.emit();
            }
            Ok(JobOutcome::Cancelled) => {
                tracing::info!(
                    job_kind = ?J::JOB_KIND,
                    target_id = %target_id,
                    elapsed = ?start.elapsed(),
                    "job cancelled"
                );

                delete_job_row(&self.pool, J::JOB_KIND, &target_id).await?;
            }
            Err(error) => {
                let job_row = jobs_entity::Entity::find()
                    .filter(jobs_entity::Column::JobKind.eq(J::JOB_KIND.code()))
                    .filter(jobs_entity::Column::TargetId.eq(target_id.clone()))
                    .one(&self.pool)
                    .await?
                    .with_context(|| {
                        format!(
                            "missing job row while recording failed execution for {:?} {}",
                            J::JOB_KIND,
                            target_id
                        )
                    })?;
                let attempt_count = job_row.attempt_count + 1;
                let retry_after =
                    policy.next_retry_at(chrono::Utc::now().timestamp(), attempt_count);

                tracing::warn!(
                    job_kind = ?J::JOB_KIND,
                    target_id = %target_id,
                    attempt_count,
                    retry_after,
                    error = %error,
                    "job execution failed"
                );

                persist_job_error(
                    &self.pool,
                    job_row,
                    attempt_count,
                    retry_after,
                    error.to_string(),
                )
                .await?;
            }
        }

        Ok(())
    }
}

pub(crate) async fn delete_job_row<C>(
    database: &C,
    job_kind: jobs_entity::JobKind,
    target_id: &str,
) -> anyhow::Result<()>
where
    C: ConnectionTrait,
{
    jobs_entity::Entity::delete_many()
        .filter(jobs_entity::Column::JobKind.eq(job_kind.code()))
        .filter(jobs_entity::Column::TargetId.eq(target_id))
        .exec(database)
        .await?;
    Ok(())
}

pub(crate) async fn persist_job_error<C>(
    database: &C,
    job_row: jobs_entity::Model,
    attempt_count: i64,
    retry_after: Option<i64>,
    message: String,
) -> anyhow::Result<()>
where
    C: ConnectionTrait,
{
    let now = chrono::Utc::now().timestamp();
    let mut updated: jobs_entity::ActiveModel = job_row.into();
    updated.state = Set(jobs_entity::JobState::Errored.into_value());
    updated.locked_at = Set(None);
    updated.retry_after = Set(retry_after);
    updated.last_run_at = Set(now);
    updated.last_error_message = Set(Some(message));
    updated.attempt_count = Set(attempt_count);
    updated.updated_at = Set(now);
    updated.update(database).await?;
    Ok(())
}

fn blocked_job_targets_query(
    job_kind: jobs_entity::JobKind,
    now: i64,
) -> sea_orm::sea_query::SelectStatement {
    let claimable_retry = Condition::all()
        .add(jobs_entity::Column::State.eq(jobs_entity::JobState::Errored))
        .add(jobs_entity::Column::LockedAt.is_null())
        .add(jobs_entity::Column::RetryAfter.is_not_null())
        .add(jobs_entity::Column::RetryAfter.lte(now));

    Query::select()
        .column(jobs_entity::Column::TargetId)
        .from(jobs_entity::Entity)
        .and_where(jobs_entity::Column::JobKind.eq(job_kind.code()))
        .cond_where(Condition::all().not().add(claimable_retry))
        .to_owned()
}
