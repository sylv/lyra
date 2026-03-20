use crate::content_update::CONTENT_UPDATE;
use crate::entities::jobs as jobs_entity;
use crate::job_block::JobLock;
use anyhow::Context;
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::Set,
    ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    sea_query::{Expr, OnConflict, SelectStatement},
};
use std::collections::{HashMap, HashSet};
use std::sync::{
    Arc,
    atomic::{AtomicI64, Ordering},
};
use std::time::Instant;
use tokio::sync::{Notify, OwnedSemaphorePermit, Semaphore};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

pub mod handlers;
pub mod on_demand;
pub mod registry;
pub use on_demand::{TryRunJobFilter, clear_locked_jobs_on_startup, try_run_job};

const EMPTY_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_secs(30);
const DEFAULT_BACKOFF_SECONDS: &[i64] = &[24 * 60 * 60, 7 * 24 * 60 * 60, 30 * 24 * 60 * 60];
const EXISTING_SUBJECT_LOOKUP_CHUNK_SIZE: usize = 400;
pub const IDLE_RESET_AFTER_SECONDS: i64 = 5 * 60;
pub const ACTIVITY_STALE_AFTER_SECONDS: i64 = 60;

pub const SUBJECT_KEY_COLUMN: &str = "subject_key";
pub const TARGET_ID_COLUMN: &str = "target_id";
pub const VERSION_KEY_COLUMN: &str = "version_key";
pub const FILE_ID_COLUMN: &str = "file_id";
pub const ASSET_ID_COLUMN: &str = "asset_id";
pub const NODE_ID_COLUMN: &str = "node_id";

pub struct JobExecutionPolicy {
    backoff_seconds: &'static [i64],
}

#[derive(Clone)]
pub struct JobRunContext {
    cancellation_token: CancellationToken,
}

impl JobRunContext {
    pub fn new(cancellation_token: CancellationToken) -> Self {
        Self { cancellation_token }
    }

    pub fn cancellation_token(&self) -> &CancellationToken {
        &self.cancellation_token
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancellation_token.is_cancelled()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobRunResult {
    Complete,
    Cancelled,
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
    Node,
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
                        .try_get_by::<Option<String>, _>(TARGET_ID_COLUMN)
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
                    target.asset_id = row
                        .try_get_by::<Option<String>, _>(TARGET_ID_COLUMN)
                        .ok()
                        .flatten();
                }

                if target.asset_id.is_none() {
                    anyhow::bail!(
                        "missing asset target id (expected `{ASSET_ID_COLUMN}` or `{TARGET_ID_COLUMN}`)"
                    );
                }
            }
            JobTarget::Node => {
                if target.node_id.is_none() {
                    target.node_id = row
                        .try_get_by::<Option<String>, _>(TARGET_ID_COLUMN)
                        .ok()
                        .flatten();
                }

                if target.node_id.is_none() {
                    anyhow::bail!(
                        "missing node target id (expected `{NODE_ID_COLUMN}` or `{TARGET_ID_COLUMN}`)"
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
                    .clone()
                    .with_context(|| "missing file_id while building subject key")?
            )),
            JobTarget::Asset => Ok(format!(
                "asset:{segment}:{}",
                target
                    .asset_id
                    .clone()
                    .as_deref()
                    .with_context(|| "missing asset_id while building subject key")?
            )),
            JobTarget::Node => Ok(format!(
                "node:{segment}:{}",
                target
                    .node_id
                    .as_deref()
                    .with_context(|| "missing node_id while building subject key")?
            )),
        }
    }
}

#[async_trait::async_trait]
pub trait JobHandler: Send + Sync {
    fn job_kind(&self) -> jobs_entity::JobKind;

    fn is_heavy(&self) -> bool;

    fn targets(&self) -> (JobTarget, SelectStatement);

    fn execution_policy(&self) -> JobExecutionPolicy {
        JobExecutionPolicy::default()
    }

    async fn execute(
        &self,
        pool: &DatabaseConnection,
        job: &jobs_entity::Model,
        ctx: &JobRunContext,
    ) -> anyhow::Result<JobRunResult>;
}

struct PendingTargetRecord {
    subject_key: String,
    version_key: Option<i64>,
    file_id: Option<String>,
    asset_id: Option<String>,
    node_id: Option<String>,
}

#[derive(Debug)]
pub struct JobActivityState {
    idle_at: AtomicI64,
    last_activity_at: AtomicI64,
    idle_started_at: AtomicI64,
}

impl JobActivityState {
    fn new(now: i64) -> Self {
        Self {
            idle_at: AtomicI64::new(now),
            last_activity_at: AtomicI64::new(now),
            idle_started_at: AtomicI64::new(0),
        }
    }

    fn mark_active(&self, now: i64) {
        self.last_activity_at.store(now, Ordering::Relaxed);
        self.idle_started_at.store(0, Ordering::Relaxed);
    }

    fn mark_idle(&self, now: i64) {
        let idle_started_at = self.idle_started_at.load(Ordering::Relaxed);
        if idle_started_at == 0 {
            self.idle_started_at.store(now, Ordering::Relaxed);
            return;
        }

        if now - idle_started_at >= IDLE_RESET_AFTER_SECONDS {
            self.idle_at.store(now, Ordering::Relaxed);
            self.idle_started_at.store(now, Ordering::Relaxed);
        }
    }

    fn snapshot(&self) -> JobActivitySnapshot {
        JobActivitySnapshot {
            idle_at: self.idle_at.load(Ordering::Relaxed),
            last_activity_at: self.last_activity_at.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct JobActivitySnapshot {
    pub idle_at: i64,
    pub last_activity_at: i64,
}

#[derive(Clone)]
pub struct JobActivityRegistry {
    states: Arc<HashMap<jobs_entity::JobKind, Arc<JobActivityState>>>,
}

impl JobActivityRegistry {
    pub fn new(job_kinds: impl IntoIterator<Item = jobs_entity::JobKind>, now: i64) -> Self {
        let mut states = HashMap::new();
        for job_kind in job_kinds {
            states.insert(job_kind, Arc::new(JobActivityState::new(now)));
        }

        Self {
            states: Arc::new(states),
        }
    }

    pub fn state(&self, job_kind: jobs_entity::JobKind) -> Option<Arc<JobActivityState>> {
        self.states.get(&job_kind).cloned()
    }

    pub fn snapshot(&self, job_kind: jobs_entity::JobKind) -> Option<JobActivitySnapshot> {
        self.states
            .get(&job_kind)
            .map(|state| state.as_ref().snapshot())
    }

    pub fn job_kinds(&self) -> Vec<jobs_entity::JobKind> {
        self.states.keys().copied().collect()
    }
}

pub struct JobManager {
    handler: Arc<dyn JobHandler>,
    database: DatabaseConnection,
    wake_signal: Arc<Notify>,
    activity_state: Arc<JobActivityState>,
    job_lock: JobLock,
    heavy_semaphore: Arc<Semaphore>,
}

impl JobManager {
    pub fn new(
        handler: Arc<dyn JobHandler>,
        database: DatabaseConnection,
        wake_signal: Arc<Notify>,
        activity_state: Arc<JobActivityState>,
        job_lock: JobLock,
        heavy_semaphore: Arc<Semaphore>,
    ) -> Self {
        Self {
            handler,
            database,
            wake_signal,
            activity_state,
            job_lock,
            heavy_semaphore,
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

            let mut next_job = jobs_entity::Entity::find()
                .filter(jobs_entity::Column::JobKind.eq(self.handler.job_kind()))
                .filter(jobs_entity::Column::LockedAt.is_null())
                .filter(jobs_entity::Column::RunAfter.is_not_null())
                .filter(jobs_entity::Column::RunAfter.lte(now))
                .order_by_asc(Expr::col(jobs_entity::Column::PriorityAt).is_null())
                .order_by_asc(jobs_entity::Column::PriorityAt)
                .order_by_asc(jobs_entity::Column::RunAfter)
                .order_by_asc(jobs_entity::Column::Id)
                .one(&self.database)
                .await?;
            let had_work = enqueued > 0 || next_job.is_some();

            if had_work {
                self.activity_state.mark_active(now);
            } else {
                self.activity_state.mark_idle(now);
            }

            if let Some(mut job) = next_job.take() {
                // heavy background work yields to the shared job lock unless it was explicitly promoted.
                let heavy_permit = if self.handler.is_heavy() && job.priority_at.is_none() {
                    if self.job_lock.is_blocked() {
                        self.wait_for_work_or_job_unlock().await;
                        continue;
                    }

                    let Some(permit) = self.try_acquire_heavy_permit().await else {
                        continue;
                    };

                    if self.job_lock.is_blocked() {
                        self.wait_for_work_or_job_unlock().await;
                        continue;
                    }

                    Some(permit)
                } else {
                    None
                };

                let lock_now = chrono::Utc::now().timestamp();
                let lock_result = jobs_entity::Entity::update_many()
                    .set(jobs_entity::ActiveModel {
                        locked_at: Set(Some(lock_now)),
                        updated_at: Set(lock_now),
                        ..Default::default()
                    })
                    .filter(jobs_entity::Column::Id.eq(job.id))
                    .filter(jobs_entity::Column::LockedAt.is_null())
                    .exec(&self.database)
                    .await?;

                if lock_result.rows_affected != 1 {
                    continue;
                }

                job.locked_at = Some(lock_now);
                job.updated_at = lock_now;
                self.run_job_for_entry(job, heavy_permit).await?;
                self.activity_state
                    .mark_active(chrono::Utc::now().timestamp());
            } else if self.handler.is_heavy() && self.job_lock.is_blocked() {
                self.wait_for_work_or_job_unlock().await;
            } else if enqueued == 0 {
                self.wait_for_work().await;
            }
        }
    }

    async fn wait_for_work(&self) {
        tokio::select! {
            _ = self.wake_signal.notified() => {},
            _ = sleep(EMPTY_POLL_INTERVAL) => {},
        }
    }

    async fn wait_for_work_or_job_unlock(&self) {
        tokio::select! {
            _ = self.wake_signal.notified() => {},
            _ = self.job_lock.wait_until_unblocked() => {},
            _ = sleep(EMPTY_POLL_INTERVAL) => {},
        }
    }

    async fn try_acquire_heavy_permit(&self) -> Option<OwnedSemaphorePermit> {
        if let Ok(permit) = self.heavy_semaphore.clone().try_acquire_owned() {
            return Some(permit);
        }

        // non-priority heavy jobs wait in the semaphore queue, but wake-driven requeries
        // still get a chance to notice a newly promoted priority job and step aside.
        tokio::select! {
            biased;
            _ = self.wake_signal.notified() => None,
            _ = self.job_lock.wait_until_blocked() => None,
            permit = self.heavy_semaphore.clone().acquire_owned() => {
                Some(permit.expect("heavy job semaphore closed"))
            },
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
                .try_get_by::<Option<String>, _>(FILE_ID_COLUMN)
                .ok()
                .flatten();
            let asset_id = row
                .try_get_by::<Option<String>, _>(ASSET_ID_COLUMN)
                .ok()
                .flatten();
            let node_id = row
                .try_get_by::<Option<String>, _>(NODE_ID_COLUMN)
                .ok()
                .flatten();

            let mut target = PendingTargetRecord {
                subject_key: String::new(),
                version_key,
                file_id,
                asset_id,
                node_id,
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
                if existing.job_kind != self.handler.job_kind().code() {
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
                updated.asset_id = Set(target.asset_id);
                updated.node_id = Set(target.node_id.clone());
                updated.locked_at = Set(None);
                updated.priority_at = Set(None);
                updated.run_after = Set(Some(now));
                updated.last_error_message = Set(None);
                updated.attempt_count = Set(0);
                updated.updated_at = Set(now);
                updated.update(&self.database).await?;
                enqueued += 1;
                continue;
            }

            let job = jobs_entity::ActiveModel {
                job_kind: Set(self.handler.job_kind().code()),
                subject_key: Set(target.subject_key),
                version_key: Set(target.version_key),
                file_id: Set(target.file_id),
                asset_id: Set(target.asset_id),
                node_id: Set(target.node_id),
                locked_at: Set(None),
                priority_at: Set(None),
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
    async fn run_job_for_entry(
        &self,
        job: jobs_entity::Model,
        _heavy_permit: Option<OwnedSemaphorePermit>,
    ) -> anyhow::Result<()> {
        let job_kind = self.handler.job_kind();
        let policy = self.handler.execution_policy();
        let now = chrono::Utc::now().timestamp();
        let subject_key = job.subject_key.clone();
        let is_priority = job.priority_at.is_some();
        let is_heavy = self.handler.is_heavy() && !is_priority;
        let cancellation_token = CancellationToken::new();
        let run_ctx = JobRunContext::new(cancellation_token.clone());

        let start = Instant::now();
        tracing::info!(
            job_kind = ?job_kind,
            subject_key = %subject_key,
            is_heavy,
            "executing job"
        );

        if run_ctx.is_cancelled() {
            tracing::info!(
                job_kind = ?job_kind,
                subject_key = %subject_key,
                "job cancelled before execution"
            );
            self.persist_job_outcome(job, now, JobOutcome::cancelled(now))
                .await?;
            return Ok(());
        }

        let result = if is_heavy {
            let handler = self.handler.clone();
            let database = self.database.clone();
            let task_job = job.clone();
            let task =
                tokio::spawn(async move { handler.execute(&database, &task_job, &run_ctx).await });
            tokio::pin!(task);

            tokio::select! {
                result = &mut task => result.context("heavy job task panicked")?,
                _ = self.job_lock.wait_until_blocked() => {
                    tracing::info!(
                        job_kind = ?job_kind,
                        subject_key = %subject_key,
                        "job lock engaged; cancelling heavy job"
                    );
                    cancellation_token.cancel();
                    task.await.context("heavy job task panicked")?
                }
            }
        } else {
            self.handler.execute(&self.database, &job, &run_ctx).await
        };

        match result {
            Ok(JobRunResult::Complete) => {
                tracing::debug!(
                    job_kind = ?job_kind,
                    subject_key = %subject_key,
                    elapsed = ?start.elapsed(),
                    "finished job"
                );

                self.persist_job_outcome(job, now, JobOutcome::success())
                    .await?;
                CONTENT_UPDATE.emit();
            }
            Ok(JobRunResult::Cancelled) => {
                tracing::info!(
                    job_kind = ?job_kind,
                    subject_key = %subject_key,
                    elapsed = ?start.elapsed(),
                    "job cancelled"
                );

                self.persist_job_outcome(job, now, JobOutcome::cancelled(now))
                    .await?;
            }
            Err(error) if cancellation_token.is_cancelled() => {
                tracing::info!(
                    job_kind = ?job_kind,
                    subject_key = %subject_key,
                    elapsed = ?start.elapsed(),
                    error = %error,
                    "job exited after cancellation"
                );

                self.persist_job_outcome(job, now, JobOutcome::cancelled(now))
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
        updated.locked_at = Set(None);
        updated.priority_at = Set(None);
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

    fn cancelled(now: i64) -> Self {
        Self {
            run_after: Some(now),
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
