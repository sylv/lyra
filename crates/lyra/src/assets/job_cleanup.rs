use crate::config::get_config;
use crate::entities::{assets, jobs as jobs_entity};
use crate::jobs::{Job, JobLease, JobOutcome};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, Select,
    sea_query::{Alias, Expr, Query},
};

#[derive(Debug, Default)]
pub struct AssetCleanupJob;

#[async_trait::async_trait]
impl Job for AssetCleanupJob {
    type Entity = assets::Entity;
    type Model = assets::Model;

    const JOB_KIND: jobs_entity::JobKind = jobs_entity::JobKind::AssetCleanup;

    fn query(&self) -> Select<Self::Entity> {
        let references = Alias::new("asset_references");
        let stale_before = chrono::Utc::now().timestamp() - 6 * 60 * 60;
        assets::Entity::find()
            .filter(assets::Column::UpdatedAt.lte(stale_before))
            .filter(
                Expr::col((assets::Entity, assets::Column::Id)).not_in_subquery(
                    Query::select()
                        .column((references.clone(), Alias::new("asset_id")))
                        .from(references)
                        .to_owned(),
                ),
            )
            .order_by_asc(assets::Column::Id)
    }

    fn target_id(&self, target: &Self::Model) -> String {
        target.id.clone()
    }

    async fn run(
        &self,
        db: &DatabaseConnection,
        asset: Self::Model,
        _ctx: &JobLease,
    ) -> anyhow::Result<JobOutcome> {
        let shared_hash_exists = if let Some(hash) = asset.hash_sha256.as_deref() {
            assets::Entity::find()
                .filter(assets::Column::Id.ne(asset.id.clone()))
                .filter(assets::Column::HashSha256.eq(hash.to_string()))
                .filter(
                    Condition::any()
                        .add(assets::Column::SourceUrl.is_not_null())
                        .add(assets::Column::HashSha256.is_not_null()),
                )
                .one(db)
                .await?
                .is_some()
        } else {
            false
        };

        if let (Some(hash), Some(mime_type)) =
            (asset.hash_sha256.as_deref(), asset.mime_type.as_deref())
            && !shared_hash_exists
        {
            let path = super::storage::get_asset_output_path_from_mime_and_encoding(
                hash,
                mime_type,
                asset.content_encoding.as_deref(),
            )?;
            remove_file_if_exists(&path).await?;
            remove_transformed_cache(hash).await?;
        }

        let active: assets::ActiveModel = asset.into();
        active.delete(db).await?;
        Ok(JobOutcome::Complete)
    }
}

async fn remove_file_if_exists(path: &std::path::Path) -> anyhow::Result<()> {
    match tokio::fs::remove_file(path).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

async fn remove_transformed_cache(hash: &str) -> anyhow::Result<()> {
    let mut dir = match tokio::fs::read_dir(get_config().get_image_dir()).await {
        Ok(dir) => dir,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };

    let prefix = format!("{hash}_");
    while let Some(entry) = dir.next_entry().await? {
        let file_name = entry.file_name();
        if file_name.to_string_lossy().starts_with(&prefix) {
            let _ = tokio::fs::remove_file(entry.path()).await;
        }
    }

    Ok(())
}
