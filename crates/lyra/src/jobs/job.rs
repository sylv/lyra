use crate::entities::jobs as jobs_entity;
use crate::jobs::semaphore::JobLease;
use sea_orm::Iterable;
use sea_orm::{
    DatabaseConnection, EntityTrait, FromQueryResult, ModelTrait, PrimaryKeyToColumn, Select,
};

const DEFAULT_BACKOFF_SECONDS: &[i64] = &[
    // 1 day
    24 * 60 * 60,
    // 7 days
    7 * 24 * 60 * 60,
    // 30 days
    30 * 24 * 60 * 60,
];

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

    pub(crate) fn next_retry_at(&self, now: i64, attempt_count: i64) -> Option<i64> {
        self.backoff_seconds
            .get(attempt_count.saturating_sub(1) as usize)
            .map(|offset| now + offset)
    }
}

#[async_trait::async_trait]
pub trait Job: Send + Sync + 'static {
    type Entity: EntityTrait<Model = Self::Model>;
    type Model: ModelTrait<Entity = Self::Entity> + FromQueryResult + Send + Sync;

    const JOB_KIND: jobs_entity::JobKind;
    const IS_HEAVY: bool = false;

    fn query(&self) -> Select<Self::Entity>;

    fn target_id(&self, target: &Self::Model) -> String;

    fn execution_policy(&self) -> JobExecutionPolicy {
        JobExecutionPolicy::default()
    }

    fn target_id_column(&self) -> <Self::Entity as EntityTrait>::Column {
        let mut it = <Self::Entity as EntityTrait>::PrimaryKey::iter();
        let pk = it
            .next()
            .expect("Job target entity must have a primary key");
        assert!(
            it.next().is_none(),
            "Job target entity must have exactly one primary key column"
        );
        <<Self::Entity as EntityTrait>::PrimaryKey as PrimaryKeyToColumn>::into_column(pk)
    }

    async fn run(
        &self,
        db: &DatabaseConnection,
        target: Self::Model,
        lease: &JobLease,
    ) -> anyhow::Result<JobOutcome>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobOutcome {
    Complete,
    Cancelled,
}
