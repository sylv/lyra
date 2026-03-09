use crate::entities::{
    assets as assets_entity,
    file_assets::{self, FileAssetRole},
    files, jobs as jobs_entity, libraries,
};
use crate::jobs::{JobTargetId, TARGET_ID_COLUMN};
use anyhow::Context;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, TransactionTrait, sea_query::SelectStatement,
};
use std::path::PathBuf;

pub struct JobFileContext {
    pub file: files::Model,
    pub file_path: PathBuf,
}

pub fn expect_file_target(target_id: &JobTargetId) -> anyhow::Result<i64> {
    match target_id {
        JobTargetId::File(file_id) => Ok(*file_id),
        other => anyhow::bail!("expected file target, got {other:?}"),
    }
}

pub fn expect_asset_target(target_id: &JobTargetId) -> anyhow::Result<i64> {
    match target_id {
        JobTargetId::Asset(asset_id) => Ok(*asset_id),
        other => anyhow::bail!("expected asset target, got {other:?}"),
    }
}

pub fn base_file_targets_query() -> SelectStatement {
    let mut query = files::Entity::find()
        .select_only()
        .column_as(files::Column::Id, TARGET_ID_COLUMN)
        .filter(files::Column::UnavailableAt.is_null())
        .order_by_asc(files::Column::Id);
    QuerySelect::query(&mut query).to_owned()
}

pub async fn load_job_file_context(
    pool: &DatabaseConnection,
    file_id: i64,
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
            id: Set(file.id),
            unavailable_at: Set(Some(chrono::Utc::now().timestamp())),
            ..Default::default()
        })
        .exec(pool)
        .await?;

        anyhow::bail!("file path missing while executing job");
    }

    Ok(Some(JobFileContext { file, file_path }))
}

pub async fn cleanup_file_assets_for_role(
    pool: &DatabaseConnection,
    file_id: i64,
    role: FileAssetRole,
) -> anyhow::Result<()> {
    let tx = pool.begin().await?;
    let stale_asset_ids: Vec<i64> = file_assets::Entity::find()
        .filter(file_assets::Column::FileId.eq(file_id))
        .filter(file_assets::Column::Role.eq(role))
        .all(&tx)
        .await?
        .into_iter()
        .map(|row| row.asset_id)
        .collect();

    file_assets::Entity::delete_many()
        .filter(file_assets::Column::FileId.eq(file_id))
        .filter(file_assets::Column::Role.eq(role))
        .exec(&tx)
        .await?;

    if !stale_asset_ids.is_empty() {
        assets_entity::Entity::delete_many()
            .filter(assets_entity::Column::Id.is_in(stale_asset_ids))
            .exec(&tx)
            .await?;
    }

    tx.commit().await?;
    Ok(())
}
