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

pub const SUBJECT_KEY_COLUMN: &str = "subject_key";
pub const TARGET_ID_COLUMN: &str = "target_id";
pub const VERSION_KEY_COLUMN: &str = "version_key";
pub const FILE_ID_COLUMN: &str = "file_id";
pub const ASSET_ID_COLUMN: &str = "asset_id";
pub const ROOT_ID_COLUMN: &str = "root_id";
pub const SEASON_ID_COLUMN: &str = "season_id";
pub const ITEM_ID_COLUMN: &str = "item_id";

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
    pub const fn with_backoff_seconds(backoff_seconds: &'static [i64]) -> Self {
        Self { backoff_seconds }
    }

    fn next_retry_at(&self, now: i64, attempt_count: i64) -> Option<i64> {
        self.backoff_seconds
            .get(attempt_count.saturating_sub(1) as usize)
            .map(|offset| now + offset)
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
    fn populate_required_target_fields(
        self,
        row: &sea_orm::QueryResult,
        target: &mut PendingTargetRecord,
    ) -> anyhow::Result<()> {
        match self {
            JobTarget::File => {
                if target.file_id.is_none() {
                    target.file_id = row
                        .try_get_by::<Option<i64>, _>(TARGET_ID_COLUMN)
                        .ok()
                        .flatten();
                }

                if target.file_id.is_none() {
                    anyhow::bail!(
                        "missing file target id (expected `{FILE_ID_COLUMN}` or `{TARGET_ID_COLUMN}`)"
                    );
                }
            }
            JobTarget::Asset => {
                if target.asset_id.is_none() {
                    target.asset_id = read_optional_string_or_i64(row, TARGET_ID_COLUMN)?;
                }

                if target.asset_id.is_none() {
                    anyhow::bail!(
                        "missing asset target id (expected `{ASSET_ID_COLUMN}` or `{TARGET_ID_COLUMN}`)"
                    );
                }
            }
            JobTarget::Root => {
                if target.root_id.is_none() {
                    target.root_id = row
                        .try_get_by::<Option<String>, _>(TARGET_ID_COLUMN)
                        .ok()
                        .flatten();
                }

                if target.root_id.is_none() {
                    anyhow::bail!(
                        "missing root target id (expected `{ROOT_ID_COLUMN}` or `{TARGET_ID_COLUMN}`)"
                    );
                }
            }
            JobTarget::Item => {
                if target.item_id.is_none() {
                    target.item_id = row
                        .try_get_by::<Option<String>, _>(TARGET_ID_COLUMN)
                        .ok()
                        .flatten();
                }

                if target.item_id.is_none() {
                    anyhow::bail!(
                        "missing item target id (expected `{ITEM_ID_COLUMN}` or `{TARGET_ID_COLUMN}`)"
                    );
                }
            }
        }

        Ok(())
    }

    fn build_default_subject_key(
        self,
        job_kind: jobs_entity::JobKind,
        target: &PendingTargetRecord,
    ) -> anyhow::Result<String> {
        let segment = job_kind.subject_segment();

        match self {
            JobTarget::File => Ok(format!(
                "file:{segment}:{}",
                target
                    .file_id
                    .with_context(|| "missing file_id while building subject key")?
            )),
            JobTarget::Asset => Ok(format!(
                "asset:{segment}:{}",
                target
                    .asset_id
                    .as_deref()
                    .with_context(|| "missing asset_id while building subject key")?
            )),
            JobTarget::Item => Ok(format!(
                "item:{segment}:{}",
                target
                    .item_id
                    .as_deref()
                    .with_context(|| "missing item_id while building subject key")?
            )),
            JobTarget::Root => {
                let root_id = target
                    .root_id
                    .as_deref()
                    .with_context(|| "missing root_id while building subject key")?;
                if let Some(season_id) = target.season_id.as_deref() {
                    Ok(format!("root_season:{segment}:{root_id}:{season_id}"))
                } else {
                    Ok(format!("root:{segment}:{root_id}"))
                }
            }
        }
    }
}

#[async_trait::async_trait]
pub trait JobHandler: Send + Sync {
    fn job_kind(&self) -> jobs_entity::JobKind;

    fn targets(&self) -> (JobTarget, SelectStatement);

    fn execution_policy(&self) -> JobExecutionPolicy {
        JobExecutionPolicy::default()
    }

    async fn execute(
        &self,
        pool: &DatabaseConnection,
        job: &jobs_entity::Model,
    ) -> anyhow::Result<()>;
}

struct PendingTargetRecord {
    subject_key: String,
    version_key: Option<i64>,
    file_id: Option<i64>,
    asset_id: Option<String>,
    root_id: Option<String>,
    season_id: Option<String>,
    item_id: Option<String>,
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
                self.run_job_for_entry(job).await?;
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
            let version_key = row
                .try_get_by::<Option<i64>, _>(VERSION_KEY_COLUMN)
                .ok()
                .flatten();
            let file_id = row
                .try_get_by::<Option<i64>, _>(FILE_ID_COLUMN)
                .ok()
                .flatten();
            let asset_id = read_optional_string_or_i64(&row, ASSET_ID_COLUMN)?;
            let root_id = row
                .try_get_by::<Option<String>, _>(ROOT_ID_COLUMN)
                .ok()
                .flatten();
            let season_id = row
                .try_get_by::<Option<String>, _>(SEASON_ID_COLUMN)
                .ok()
                .flatten();
            let item_id = row
                .try_get_by::<Option<String>, _>(ITEM_ID_COLUMN)
                .ok()
                .flatten();

            let mut target = PendingTargetRecord {
                subject_key: String::new(),
                version_key,
                file_id,
                asset_id,
                root_id,
                season_id,
                item_id,
            };

            target_kind.populate_required_target_fields(&row, &mut target)?;

            let subject_key = row
                .try_get_by::<Option<String>, _>(SUBJECT_KEY_COLUMN)
                .ok()
                .flatten()
                .map(Ok)
                .unwrap_or_else(|| {
                    target_kind.build_default_subject_key(self.handler.job_kind(), &target)
                })?;

            if !seen_subject_keys.insert(subject_key.clone()) {
                continue;
            }

            target.subject_key = subject_key;
            targets.push(target);
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
                updated.file_id = Set(target.file_id);
                updated.asset_id = Set(target.asset_id.clone());
                updated.root_id = Set(target.root_id.clone());
                updated.season_id = Set(target.season_id.clone());
                updated.item_id = Set(target.item_id.clone());
                updated.run_after = Set(Some(now));
                updated.last_error_message = Set(None);
                updated.attempt_count = Set(0);
                updated.updated_at = Set(now);
                updated.update(&self.database).await?;
                enqueued += 1;
                continue;
            }

            let job = jobs_entity::ActiveModel {
                job_kind: Set(self.handler.job_kind()),
                subject_key: Set(target.subject_key),
                version_key: Set(target.version_key),
                file_id: Set(target.file_id),
                asset_id: Set(target.asset_id),
                root_id: Set(target.root_id),
                season_id: Set(target.season_id),
                item_id: Set(target.item_id),
                run_after: Set(Some(now)),
                last_run_at: Set(0),
                last_error_message: Set(None),
                attempt_count: Set(0),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            };

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

    async fn run_job_for_entry(&self, job: jobs_entity::Model) -> anyhow::Result<()> {
        let job_kind = self.handler.job_kind();
        let policy = self.handler.execution_policy();
        let now = chrono::Utc::now().timestamp();
        let subject_key = job.subject_key.clone();

        let start = Instant::now();
        tracing::info!(
            job_kind = ?job_kind,
            subject_key = %subject_key,
            "executing job"
        );

        match self.handler.execute(&self.database, &job).await {
            Ok(()) => {
                tracing::debug!(
                    job_kind = ?job_kind,
                    subject_key = %subject_key,
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
                    subject_key = %subject_key,
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

fn read_optional_string_or_i64(
    row: &sea_orm::QueryResult,
    column_name: &str,
) -> anyhow::Result<Option<String>> {
    if let Ok(value) = row.try_get_by::<Option<String>, _>(column_name) {
        return Ok(value);
    }

    if let Ok(value) = row.try_get_by::<Option<i64>, _>(column_name) {
        return Ok(value.map(|value| value.to_string()));
    }

    Ok(None)
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
