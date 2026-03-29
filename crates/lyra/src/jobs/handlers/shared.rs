use crate::entities::{files, jobs as jobs_entity, libraries};
use anyhow::Context;
use sea_orm::{ActiveValue::Set, ConnectionTrait, EntityTrait};
use std::path::PathBuf;

pub async fn get_job_file_path(
    pool: &impl ConnectionTrait,
    file: &files::Model,
    job_kind: jobs_entity::JobKind,
) -> anyhow::Result<Option<PathBuf>> {
    let maybe_library = libraries::Entity::find_by_id(file.library_id.clone())
        .one(pool)
        .await
        .with_context(|| format!("failed to fetch library for file {}", file.id))?;

    let Some(library) = maybe_library else {
        return Ok(None);
    };

    if file.unavailable_at.is_some() {
        return Ok(None);
    }

    let file_path = PathBuf::from(&library.path).join(&file.relative_path);
    if !file_path.exists() {
        tracing::warn!(
            job_kind = ?job_kind,
            file_id = file.id,
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

    Ok(Some(file_path))
}
