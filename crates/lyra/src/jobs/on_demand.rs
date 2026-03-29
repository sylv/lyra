use crate::entities::jobs as jobs_entity;
use crate::jobs::manager::{delete_job_row, persist_job_error};
use crate::jobs::{JobLease, JobOutcome};
use crate::{activity::ActivityHandle, jobs::Job};
use anyhow::Context;
use sea_orm::{ActiveEnum, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use sqlx::query;
use std::time::Duration;
use tokio::time::{sleep, timeout};

const JOB_WAIT_POLL_INTERVAL: Duration = Duration::from_millis(500);

pub async fn try_run_job<J: Job>(
    database: &DatabaseConnection,
    job: &J,
    target: J::Model,
    timeout_duration: Duration,
) -> anyhow::Result<()> {
    let target_id = job.target_id(&target);
    let now = chrono::Utc::now().timestamp();
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
    .execute(database.get_sqlite_connection_pool())
    .await
    .with_context(|| {
        format!(
            "failed to claim job for target {} of {:?}",
            target_id,
            J::JOB_KIND
        )
    })?;

    if lock_result.rows_affected() == 0 {
        return poll_completion::<J>(database, target_id, timeout_duration).await;
    } else {
        return run_inline(database, job, target_id, target).await;
    }
}

async fn poll_completion<J: Job>(
    database: &DatabaseConnection,
    target_id: String,
    timeout_duration: Duration,
) -> anyhow::Result<()> {
    timeout(timeout_duration, async {
        loop {
            let current = jobs_entity::Entity::find()
                .filter(jobs_entity::Column::JobKind.eq(J::JOB_KIND.code()))
                .filter(jobs_entity::Column::TargetId.eq(target_id.clone()))
                .one(database)
                .await?;

            let Some(current) = current else {
                return Ok(());
            };

            if current.state == jobs_entity::JobState::Errored.to_value() {
                let message = current
                    .last_error_message
                    .clone()
                    .unwrap_or_else(|| format!("{:?} job failed", J::JOB_KIND));
                anyhow::bail!(message);
            }

            sleep(JOB_WAIT_POLL_INTERVAL).await;
        }
    })
    .await
    .with_context(|| format!("timed out waiting for {:?} job", J::JOB_KIND))?
}

async fn run_inline<J: Job>(
    database: &DatabaseConnection,
    job: &J,
    target_id: String,
    target: J::Model,
) -> anyhow::Result<()> {
    let _activity = ActivityHandle::new(J::JOB_KIND);
    let run_ctx = JobLease::new_blank();
    tracing::info!(
        "starting on-demand {:?} job for target {}",
        J::JOB_KIND,
        target_id
    );
    match job.run(database, target, &run_ctx).await {
        Ok(JobOutcome::Complete) | Ok(JobOutcome::Cancelled) => {
            tracing::info!(
                "completed on-demand {:?} job for target {} with outcome {:?}",
                J::JOB_KIND,
                target_id,
                JobOutcome::Complete
            );
            delete_job_row(database, J::JOB_KIND, &target_id).await?;
            Ok(())
        }
        Err(error) => {
            tracing::error!(
                "on-demand {:?} job for target {} failed with error: {:?}",
                J::JOB_KIND,
                target_id,
                error
            );
            let job_row = jobs_entity::Entity::find()
                .filter(jobs_entity::Column::JobKind.eq(J::JOB_KIND.code()))
                .filter(jobs_entity::Column::TargetId.eq(target_id.clone()))
                .one(database)
                .await?
                .with_context(|| {
                    format!(
                        "missing job row while recording on-demand failure for {:?} {}",
                        J::JOB_KIND,
                        target_id
                    )
                })?;
            let attempt_count = job_row.attempt_count + 1;
            let retry_after = job
                .execution_policy()
                .next_retry_at(chrono::Utc::now().timestamp(), attempt_count);
            let message = error.to_string();
            persist_job_error(
                database,
                job_row,
                attempt_count,
                retry_after,
                message.clone(),
            )
            .await?;
            anyhow::bail!(message);
        }
    }
}
