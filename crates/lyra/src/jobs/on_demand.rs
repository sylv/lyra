use crate::entities::jobs as jobs_entity;
use anyhow::Context;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use std::time::Duration;
use tokio::sync::Notify;
use tokio::time::{sleep, timeout};

const JOB_WAIT_POLL_INTERVAL: Duration = Duration::from_millis(100);

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum TryRunJobFilter<'a> {
    SubjectKey(&'a str),
    FileId(&'a str),
    AssetId(&'a str),
    NodeId(&'a str),
}

impl<'a> TryRunJobFilter<'a> {
    fn apply(
        self,
        query: sea_orm::Select<jobs_entity::Entity>,
    ) -> sea_orm::Select<jobs_entity::Entity> {
        match self {
            Self::SubjectKey(subject_key) => {
                query.filter(jobs_entity::Column::SubjectKey.eq(subject_key))
            }
            Self::FileId(file_id) => query.filter(jobs_entity::Column::FileId.eq(file_id)),
            Self::AssetId(asset_id) => query.filter(jobs_entity::Column::AssetId.eq(asset_id)),
            Self::NodeId(node_id) => query.filter(jobs_entity::Column::NodeId.eq(node_id)),
        }
    }
}

pub async fn clear_locked_jobs_on_startup(database: &DatabaseConnection) -> anyhow::Result<()> {
    // jobs only run in-process, so a restart can safely release any abandoned lease.
    let locked_jobs = jobs_entity::Entity::find()
        .filter(jobs_entity::Column::LockedAt.is_not_null())
        .all(database)
        .await?;

    for job in locked_jobs {
        let mut updated: jobs_entity::ActiveModel = job.into();
        updated.locked_at = Set(None);
        updated.updated_at = Set(chrono::Utc::now().timestamp());
        updated.update(database).await?;
    }

    Ok(())
}

pub async fn try_run_job(
    database: &DatabaseConnection,
    wake_signal: &Notify,
    job_kind: jobs_entity::JobKind,
    filter: TryRunJobFilter<'_>,
    timeout_duration: Duration,
) -> anyhow::Result<jobs_entity::Model> {
    let mut job = find_job(database, job_kind, filter)
        .await?
        .with_context(|| format!("failed to find existing {job_kind:?} job"))?;

    if let Some(message) = job
        .last_error_message
        .clone()
        .filter(|_| job.locked_at.is_none())
    {
        anyhow::bail!(message);
    }

    if job.locked_at.is_none() && job.run_after.is_some() {
        let now = chrono::Utc::now().timestamp();
        let mut updated: jobs_entity::ActiveModel = job.clone().into();
        updated.priority_at = Set(Some(now));
        updated.run_after = Set(Some(now));
        updated.updated_at = Set(now);
        updated.update(database).await?;
        job = jobs_entity::Entity::find_by_id(job.id)
            .one(database)
            .await?
            .with_context(|| format!("job {} disappeared after promotion", job.id))?;
    }

    wake_signal.notify_waiters();

    timeout(timeout_duration, async {
        loop {
            let current = jobs_entity::Entity::find_by_id(job.id)
                .one(database)
                .await?
                .with_context(|| format!("job {} disappeared while waiting", job.id))?;

            if let Some(message) = current
                .last_error_message
                .clone()
                .filter(|_| current.locked_at.is_none())
            {
                anyhow::bail!(message);
            }

            if current.locked_at.is_none() && current.run_after.is_none() {
                return Ok(current);
            }

            sleep(JOB_WAIT_POLL_INTERVAL).await;
        }
    })
    .await
    .with_context(|| format!("timed out waiting for {job_kind:?} job"))?
}

async fn find_job(
    database: &DatabaseConnection,
    job_kind: jobs_entity::JobKind,
    filter: TryRunJobFilter<'_>,
) -> anyhow::Result<Option<jobs_entity::Model>> {
    let query = jobs_entity::Entity::find().filter(jobs_entity::Column::JobKind.eq(job_kind));
    Ok(filter.apply(query).one(database).await?)
}
