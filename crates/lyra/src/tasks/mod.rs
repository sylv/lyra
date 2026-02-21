use crate::entities::tasks;
use anyhow::Context;
use async_graphql::Enum;
use chrono::Duration;
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::Set,
    ColumnTrait, DatabaseConnection, DeriveActiveEnum, EntityTrait, EnumIter, Order, QueryFilter,
    QueryOrder, TransactionTrait,
    prelude::Expr,
    sea_query::{ExprTrait, NullOrdering},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::time::Instant;

const RECONCILE_INTERVAL: Duration = Duration::minutes(15);

#[derive(
    Debug, Enum, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum,
)]
#[sea_orm(rs_type = "i64", db_type = "Integer")]
pub enum TaskScopeKind {
    File = 0,
    Asset = 1,
}

pub struct TaskLike<T: Serialize> {
    scope_kind: TaskScopeKind,
    scope_id: String,
    input_args: Option<T>,
    version_hash: Option<String>,
}

pub struct TaskExecutionPolicy {
    ratelimit_secs: Option<Duration>,
    backoff: Vec<Duration>,
}

impl Default for TaskExecutionPolicy {
    fn default() -> Self {
        Self {
            ratelimit_secs: None,
            backoff: vec![
                Duration::minutes(30),
                Duration::hours(6),
                Duration::days(1),
                Duration::days(7),
            ],
        }
    }
}

impl TaskExecutionPolicy {
    fn max_attempts(&self) -> usize {
        self.backoff.len() + 1
    }
}

#[async_trait::async_trait]
pub trait TaskHandler: Send + Sync {
    type InputArgs: Serialize + DeserializeOwned + Send + Sync;

    fn task_type(&self) -> &'static str;
    fn version_number(&self) -> i64;

    fn execution_policy(&self) -> TaskExecutionPolicy {
        TaskExecutionPolicy::default()
    }

    async fn reconcile(&self, pool: &DatabaseConnection) -> Vec<TaskLike<Self::InputArgs>>;

    async fn execute(
        &self,
        pool: &DatabaseConnection,
        task: &tasks::Model,
        args: Self::InputArgs,
    ) -> anyhow::Result<()>;
}

pub struct TaskManager<T: TaskHandler> {
    handler: Box<dyn TaskHandler<InputArgs = T::InputArgs>>,
    database: DatabaseConnection,
}

impl<T: TaskHandler> TaskManager<T> {
    pub fn new(
        handler: Box<dyn TaskHandler<InputArgs = T::InputArgs>>,
        database: DatabaseConnection,
    ) -> Self {
        Self { handler, database }
    }

    pub async fn start_thread(&self) -> anyhow::Result<()> {
        let mut last_reconcile: Option<Instant> = None;
        loop {
            let now = chrono::Utc::now().timestamp() as i64;
            let should_reconcile = match last_reconcile {
                None => true,
                Some(last) => last.elapsed() > RECONCILE_INTERVAL.to_std().unwrap(),
            };

            if should_reconcile {
                self.reconcile(now).await?;
                last_reconcile = Some(Instant::now());
            }

            let to_run = {
                // todo: we should be able to do this without a transaction by using a nested select
                // but sea_orm does not seem to like optional updates
                let mut tx = self.database.begin().await?;
                let to_run = tasks::Entity::find()
                    .filter(tasks::Column::TaskType.eq(self.handler.task_type()))
                    .filter(Expr::col(tasks::Column::LockedAt).is_null())
                    .filter(
                        Expr::col(tasks::Column::ExecuteAfter)
                            .lte(now)
                            .is_not_null(),
                    )
                    .order_by_with_nulls(
                        tasks::Column::ExecuteAfter,
                        Order::Asc,
                        NullOrdering::Last,
                    )
                    .one(&mut tx)
                    .await?;

                if let Some(task) = to_run {
                    tasks::Entity::update(tasks::ActiveModel {
                        id: Set(task.id),
                        locked_at: Set(Some(now)),
                        ..Default::default()
                    })
                    .exec(&mut tx)
                    .await?;

                    tx.commit().await?;
                    Some(task)
                } else {
                    None
                }
            };

            if let Some(to_run) = to_run {
                let input_args: T::InputArgs =
                    Self::decode_input_args(to_run.input_args.as_deref()).with_context(|| {
                        format!(
                            "failed to decode input_args for task id={} type={}",
                            to_run.id, to_run.task_type
                        )
                    })?;

                match self
                    .handler
                    .execute(&self.database, &to_run, input_args)
                    .await
                {
                    Ok(_) => {
                        // mark task as completed by setting execute_after to null
                        tasks::Entity::update(tasks::ActiveModel {
                            id: Set(to_run.id),
                            execute_after: Set(None),
                            locked_at: Set(None),
                            last_run_at: Set(Some(now)),
                            ..Default::default()
                        })
                        .exec(&self.database)
                        .await?;
                    }
                    Err(e) => {
                        let policy = self.handler.execution_policy();
                        let attempt_count = to_run.attempt_count + 1;
                        let should_retry = attempt_count < policy.max_attempts() as i64;
                        let execute_after = if should_retry {
                            Some(
                                (chrono::Utc::now() + policy.backoff[attempt_count as usize - 1])
                                    .timestamp() as i64,
                            )
                        } else {
                            None
                        };

                        tasks::Entity::update(tasks::ActiveModel {
                            id: Set(to_run.id),
                            last_error_message: Set(Some(e.to_string())),
                            locked_at: Set(None),
                            last_run_at: Set(Some(now)),
                            attempt_count: Set(attempt_count),
                            execute_after: Set(execute_after),
                            ..Default::default()
                        })
                        .exec(&self.database)
                        .await?;
                    }
                }
            }
        }
    }

    async fn reconcile(&self, now: i64) -> anyhow::Result<()> {
        let target_tasks = self.handler.reconcile(&self.database).await;
        let mut tx = self.database.begin().await?;
        for task in target_tasks {
            let input_args = task
                .input_args
                .as_ref()
                .map(serde_json::to_vec)
                .transpose()
                .context("failed to serialize task input_args")?;

            let task_type = self.handler.task_type();
            let mut active = tasks::ActiveModel {
                task_type: Set(task_type.to_string()),
                scope_kind: Set(task.scope_kind),
                scope_id: Set(task.scope_id.clone()),
                input_args: Set(input_args),
                version_number: Set(self.handler.version_number()),
                version_hash: Set(task.version_hash.clone()),
                ..Default::default()
            };

            let existing = tasks::Entity::find()
                .filter(tasks::Column::TaskType.eq(self.handler.task_type()))
                .filter(tasks::Column::ScopeKind.eq(task.scope_kind as i64))
                .filter(tasks::Column::ScopeId.eq(task.scope_id.clone()))
                .one(&mut tx)
                .await?;

            if let Some(existing) = existing {
                // if version number/hash is different, we reschedule the task. otherwise, we just update the input args
                active.id = Set(existing.id);
                if existing.version_hash != task.version_hash
                    || existing.version_number != self.handler.version_number()
                {
                    active.id = Set(existing.id);
                    active.attempt_count = Set(0);
                    active.last_error_message = Set(None);
                    active.execute_after = Set(Some(now));
                }

                active.update(&mut tx).await?;
            } else {
                active.insert(&mut tx).await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    fn decode_input_args(raw: Option<&[u8]>) -> anyhow::Result<T::InputArgs> {
        match raw {
            Some(bytes) => Ok(serde_json::from_slice(bytes)?),
            None => Ok(serde_json::from_value(serde_json::Value::Null)?),
        }
    }
}
