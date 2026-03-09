use crate::entities::jobs as jobs_entity;
use anyhow::Context;
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::Set,
    ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect,
    sea_query::{OnConflict, SelectStatement},
};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Notify;
use tokio::time::sleep;

pub mod handlers;
pub mod registry;

const BATCH_SIZE: u64 = 100;
const EMPTY_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_secs(10);
const DEFAULT_BACKOFF_SECONDS: &[i64] = &[24 * 60 * 60, 7 * 24 * 60 * 60, 30 * 24 * 60 * 60];
const EXISTING_SUBJECT_LOOKUP_CHUNK_SIZE: usize = 400;

pub const TARGET_ID_COLUMN: &str = "target_id";
pub const VERSION_KEY_COLUMN: &str = "version_key";

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

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobTarget {
    File,
    Asset,
    Root,
    Item,
}

impl JobTarget {
    fn read_target_id_from_query_row(
        self,
        row: &sea_orm::QueryResult,
    ) -> anyhow::Result<JobTargetId> {
        match self {
            JobTarget::File => {
                let target_id = row
                    .try_get_by::<i64, _>(TARGET_ID_COLUMN)
                    .context("missing or invalid file target_id")?;
                Ok(JobTargetId::File(target_id))
            }
            JobTarget::Asset => {
                let target_id = row
                    .try_get_by::<i64, _>(TARGET_ID_COLUMN)
                    .context("missing or invalid asset target_id")?;
                Ok(JobTargetId::Asset(target_id))
            }
            JobTarget::Root => {
                let target_id = row
                    .try_get_by::<String, _>(TARGET_ID_COLUMN)
                    .context("missing or invalid root target_id")?;
                Ok(JobTargetId::Root(target_id))
            }
            JobTarget::Item => {
                let target_id = row
                    .try_get_by::<String, _>(TARGET_ID_COLUMN)
                    .context("missing or invalid item target_id")?;
                Ok(JobTargetId::Item(target_id))
            }
        }
    }

    fn read_target_id_from_job_row(self, job: &jobs_entity::Model) -> anyhow::Result<JobTargetId> {
        match self {
            JobTarget::File => job
                .file_id
                .map(JobTargetId::File)
                .with_context(|| format!("job {} missing file_id", job.id)),
            JobTarget::Asset => {
                let raw = job
                    .asset_id
                    .as_deref()
                    .with_context(|| format!("job {} missing asset_id", job.id))?;
                let parsed = raw
                    .parse::<i64>()
                    .with_context(|| format!("job {} has non-integer asset_id '{raw}'", job.id))?;
                Ok(JobTargetId::Asset(parsed))
            }
            JobTarget::Root => job
                .root_id
                .clone()
                .map(JobTargetId::Root)
                .with_context(|| format!("job {} missing root_id", job.id)),
            JobTarget::Item => job
                .item_id
                .clone()
                .map(JobTargetId::Item)
                .with_context(|| format!("job {} missing item_id", job.id)),
        }
    }

    fn apply_target_id(
        self,
        model: &mut jobs_entity::ActiveModel,
        target_id: &JobTargetId,
    ) -> anyhow::Result<()> {
        model.file_id = Set(None);
        model.asset_id = Set(None);
        model.root_id = Set(None);
        model.item_id = Set(None);

        match (self, target_id) {
            (JobTarget::File, JobTargetId::File(id)) => {
                model.file_id = Set(Some(*id));
            }
            (JobTarget::Asset, JobTargetId::Asset(id)) => {
                model.asset_id = Set(Some(id.to_string()));
            }
            (JobTarget::Root, JobTargetId::Root(id)) => {
                model.root_id = Set(Some(id.clone()));
            }
            (JobTarget::Item, JobTargetId::Item(id)) => {
                model.item_id = Set(Some(id.clone()));
            }
            _ => {
                anyhow::bail!(
                    "job target {:?} does not match target id {:?}",
                    self,
                    target_id
                );
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum JobTargetId {
    File(i64),
    Asset(i64),
    Root(String),
    Item(String),
}

impl JobTargetId {
    fn as_subject_prefix(&self) -> &'static str {
        match self {
            JobTargetId::File(_) => "file",
            JobTargetId::Asset(_) => "asset",
            JobTargetId::Root(_) => "root",
            JobTargetId::Item(_) => "item",
        }
    }

    fn as_subject_id(&self) -> String {
        match self {
            JobTargetId::File(id) => id.to_string(),
            JobTargetId::Asset(id) => id.to_string(),
            JobTargetId::Root(id) => id.clone(),
            JobTargetId::Item(id) => id.clone(),
        }
    }

    fn as_log_value(&self) -> String {
        self.as_subject_id()
    }
}

#[async_trait::async_trait]
pub trait JobHandler: Send + Sync {
    fn job_kind(&self) -> jobs_entity::JobKind;

    fn targets(&self) -> (JobTarget, SelectStatement);

    fn execution_policy(&self) -> JobExecutionPolicy {
        JobExecutionPolicy::default()
    }

    fn subject_key(&self, target: &JobTargetId) -> String {
        format!(
            "{}:{}:{}",
            target.as_subject_prefix(),
            self.job_kind().subject_segment(),
            target.as_subject_id()
        )
    }

    async fn cleanup(
        &self,
        _pool: &DatabaseConnection,
        _target_id: &JobTargetId,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn execute(
        &self,
        pool: &DatabaseConnection,
        target_id: &JobTargetId,
    ) -> anyhow::Result<()>;
}

struct PendingTargetRecord {
    target_id: JobTargetId,
    version_key: Option<i64>,
    subject_key: String,
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

    pub fn job_kind(&self) -> jobs_entity::JobKind {
        self.handler.job_kind()
    }

    pub async fn start_thread(&self) -> anyhow::Result<()> {
        loop {
            let now = chrono::Utc::now().timestamp();
            let (target_kind, target_query) = self.handler.targets();
            let enqueued = self.sync_targets(target_kind, target_query, now).await?;
            let due_jobs = self.find_due_jobs(now).await?;

            if due_jobs.is_empty() {
                if enqueued == 0 {
                    self.wait_for_work().await;
                }
                continue;
            }

            for job in due_jobs {
                self.run_job_for_entry(target_kind, job).await?;
            }
        }
    }

    async fn wait_for_work(&self) {
        tokio::select! {
            _ = self.wake_signal.notified() => {},
            _ = sleep(EMPTY_POLL_INTERVAL) => {},
        }
    }

    async fn sync_targets(
        &self,
        target_kind: JobTarget,
        target_query: SelectStatement,
        now: i64,
    ) -> anyhow::Result<usize> {
        let statement = self.database.get_database_backend().build(&target_query);
        let rows = self.database.query_all(statement).await.with_context(|| {
            format!(
                "failed to query targets for job kind {:?}",
                self.handler.job_kind()
            )
        })?;

        if rows.is_empty() {
            return Ok(0);
        }

        let mut seen_subject_keys = HashSet::new();
        let mut targets = Vec::with_capacity(rows.len());
        for row in rows {
            let target_id = target_kind.read_target_id_from_query_row(&row)?;
            let version_key = row
                .try_get_by::<Option<i64>, _>(VERSION_KEY_COLUMN)
                .ok()
                .flatten();
            let subject_key = self.handler.subject_key(&target_id);

            if !seen_subject_keys.insert(subject_key.clone()) {
                continue;
            }

            targets.push(PendingTargetRecord {
                target_id,
                version_key,
                subject_key,
            });
        }

        if targets.is_empty() {
            return Ok(0);
        }

        let mut existing_by_subject: HashMap<String, jobs_entity::Model> = HashMap::new();
        let subject_keys = targets
            .iter()
            .map(|row| row.subject_key.clone())
            .collect::<Vec<_>>();
        for chunk in subject_keys.chunks(EXISTING_SUBJECT_LOOKUP_CHUNK_SIZE) {
            let existing = jobs_entity::Entity::find()
                .filter(jobs_entity::Column::SubjectKey.is_in(chunk.to_vec()))
                .all(&self.database)
                .await?;
            for row in existing {
                existing_by_subject.insert(row.subject_key.clone(), row);
            }
        }

        let mut enqueued = 0usize;
        for target in targets {
            let existing = existing_by_subject.get(&target.subject_key);

            if let Some(existing) = existing {
                if existing.job_kind != self.handler.job_kind() {
                    anyhow::bail!(
                        "subject key '{}' is already used by {:?}, not {:?}",
                        target.subject_key,
                        existing.job_kind,
                        self.handler.job_kind()
                    );
                }

                if existing.version_key == target.version_key {
                    continue;
                }

                let mut updated: jobs_entity::ActiveModel = existing.clone().into();
                updated.version_key = Set(target.version_key);
                target_kind.apply_target_id(&mut updated, &target.target_id)?;
                updated.run_after = Set(Some(now));
                updated.last_error_message = Set(None);
                updated.attempt_count = Set(0);
                updated.updated_at = Set(now);
                updated.update(&self.database).await?;
                enqueued += 1;
                continue;
            }

            let mut job = jobs_entity::ActiveModel {
                job_kind: Set(self.handler.job_kind()),
                subject_key: Set(target.subject_key),
                version_key: Set(target.version_key),
                run_after: Set(Some(now)),
                last_run_at: Set(0),
                last_error_message: Set(None),
                attempt_count: Set(0),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            };
            target_kind.apply_target_id(&mut job, &target.target_id)?;

            jobs_entity::Entity::insert(job)
                .on_conflict(
                    OnConflict::column(jobs_entity::Column::SubjectKey)
                        .do_nothing()
                        .to_owned(),
                )
                .exec(&self.database)
                .await?;

            enqueued += 1;
        }

        Ok(enqueued)
    }

    async fn find_due_jobs(&self, now: i64) -> anyhow::Result<Vec<jobs_entity::Model>> {
        Ok(jobs_entity::Entity::find()
            .filter(jobs_entity::Column::JobKind.eq(self.handler.job_kind()))
            .filter(jobs_entity::Column::RunAfter.is_not_null())
            .filter(jobs_entity::Column::RunAfter.lte(now))
            .order_by_asc(jobs_entity::Column::RunAfter)
            .order_by_asc(jobs_entity::Column::Id)
            .limit(BATCH_SIZE)
            .all(&self.database)
            .await?)
    }

    async fn run_job_for_entry(
        &self,
        target_kind: JobTarget,
        job: jobs_entity::Model,
    ) -> anyhow::Result<()> {
        let job_kind = self.handler.job_kind();
        let policy = self.handler.execution_policy();
        let now = chrono::Utc::now().timestamp();
        let target_id = target_kind.read_target_id_from_job_row(&job)?;

        if job.last_run_at > 0 {
            self.handler
                .cleanup(&self.database, &target_id)
                .await
                .with_context(|| {
                    format!(
                        "cleanup failed for job kind={:?} target={}",
                        job_kind,
                        target_id.as_log_value()
                    )
                })?;
        }

        let start = Instant::now();
        tracing::info!(
            job_kind = ?job_kind,
            target = target_id.as_log_value(),
            "executing job"
        );

        match self.handler.execute(&self.database, &target_id).await {
            Ok(()) => {
                tracing::debug!(
                    job_kind = ?job_kind,
                    target = target_id.as_log_value(),
                    elapsed = ?start.elapsed(),
                    "finished job"
                );

                self.persist_job_outcome(job, now, JobOutcome::success())
                    .await?;
            }
            Err(error) => {
                let attempt_count = job.attempt_count + 1;
                let run_after = policy.next_retry_at(now, attempt_count);

                tracing::warn!(
                    job_kind = ?job_kind,
                    target = target_id.as_log_value(),
                    attempt_count,
                    run_after,
                    error = %error,
                    "job execution failed"
                );

                self.persist_job_outcome(
                    job,
                    now,
                    JobOutcome::error(attempt_count, run_after, error.to_string()),
                )
                .await?;
            }
        }

        Ok(())
    }

    async fn persist_job_outcome(
        &self,
        job: jobs_entity::Model,
        now: i64,
        outcome: JobOutcome,
    ) -> anyhow::Result<()> {
        let mut updated: jobs_entity::ActiveModel = job.into();
        updated.run_after = Set(outcome.run_after);
        updated.attempt_count = Set(outcome.attempt_count);
        updated.last_error_message = Set(outcome.last_error_message);
        updated.last_run_at = Set(now);
        updated.updated_at = Set(now);
        updated.update(&self.database).await?;

        Ok(())
    }
}

struct JobOutcome {
    run_after: Option<i64>,
    attempt_count: i64,
    last_error_message: Option<String>,
}

impl JobOutcome {
    fn success() -> Self {
        Self {
            run_after: None,
            attempt_count: 0,
            last_error_message: None,
        }
    }

    fn error(attempt_count: i64, run_after: Option<i64>, message: String) -> Self {
        Self {
            run_after,
            attempt_count,
            last_error_message: Some(message),
        }
    }
}
