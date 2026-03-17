use crate::entities::{files, jobs as jobs_entity, libraries};
use crate::jobs::{FILE_ID_COLUMN, NODE_ID_COLUMN};
use anyhow::Context;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, sea_query::SelectStatement,
};
use std::path::PathBuf;

pub struct JobFileContext {
    pub file: files::Model,
    pub file_path: PathBuf,
}

pub fn expect_job_file_id(job: &jobs_entity::Model) -> anyhow::Result<String> {
    job.file_id
        .clone()
        .with_context(|| format!("job {} is missing file_id", job.id))
}

pub fn expect_job_asset_id(job: &jobs_entity::Model) -> anyhow::Result<String> {
    job.asset_id
        .clone()
        .with_context(|| format!("job {} is missing asset_id", job.id))
}

pub fn expect_job_node_id<'a>(job: &'a jobs_entity::Model) -> anyhow::Result<&'a str> {
    job.node_id
        .as_deref()
        .with_context(|| format!("job {} is missing node_id", job.id))
}

pub fn base_file_targets_query() -> SelectStatement {
    let mut query = files::Entity::find()
        .select_only()
        .column_as(files::Column::Id, FILE_ID_COLUMN)
        .filter(files::Column::UnavailableAt.is_null())
        .order_by_asc(files::Column::Id);
    QuerySelect::query(&mut query).to_owned()
}

pub fn base_node_id_alias() -> &'static str {
    NODE_ID_COLUMN
}

pub async fn load_job_file_context(
    pool: &DatabaseConnection,
    file_id: &str,
    job_kind: jobs_entity::JobKind,
) -> anyhow::Result<Option<JobFileContext>> {
    let maybe_file = files::Entity::find_by_id(file_id)
        .find_also_related(libraries::Entity)
        .one(pool)
        .await
        .with_context(|| format!("failed to fetch file {file_id}"))?;

    let Some((file, library)) = maybe_file else {
        return Ok(None);
    };
    let Some(library) = library else {
        return Ok(None);
    };

    if file.unavailable_at.is_some() {
        return Ok(None);
    }

    let file_path = PathBuf::from(&library.path).join(&file.relative_path);
    if !file_path.exists() {
        tracing::warn!(
            job_kind = ?job_kind,
            file_id,
            path = %file_path.display(),
            "file path missing while executing job"
        );

        files::Entity::update(files::ActiveModel {
            id: Set(file.id.clone()),
            unavailable_at: Set(Some(chrono::Utc::now().timestamp())),
            ..Default::default()
        })
        .exec(pool)
        .await?;

        anyhow::bail!("file path missing while executing job");
    }

    Ok(Some(JobFileContext { file, file_path }))
}
