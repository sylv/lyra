use crate::jobs::handlers::shared::get_job_file_path;
use crate::jobs::{Job, JobLease, JobOutcome, JobScheduling};
use crate::{
    entities::{file_probe, files, jobs as jobs_entity},
    file_analysis, json_encoding,
};
use anyhow::Context;
use lyra_probe::{encode_probe_data_json_zstd, extract_keyframes, probe_with_cancellation};
use sea_orm::{
    ActiveValue::Set,
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Select,
    sea_query::{Expr, OnConflict, Query},
};

#[derive(Debug, Default)]
pub struct FileProbeJob;

#[async_trait::async_trait]
impl Job for FileProbeJob {
    type Entity = files::Entity;
    type Model = files::Model;

    const JOB_KIND: jobs_entity::JobKind = jobs_entity::JobKind::FileProbe;
    const SCHEDULING: JobScheduling = JobScheduling::Heavy(0);

    fn query(&self) -> Select<Self::Entity> {
        files::Entity::find()
            .filter(files::Column::UnavailableAt.is_null())
            .filter(
                Condition::any()
                    .add(
                        Expr::col((files::Entity, files::Column::Id)).not_in_subquery(
                            Query::select()
                                .column(file_probe::Column::FileId)
                                .from(file_probe::Entity)
                                .to_owned(),
                        ),
                    )
                    .add(files::Column::KeyframesJson.is_null()),
            )
            .order_by_asc(files::Column::Id)
    }

    fn target_id(&self, target: &Self::Model) -> String {
        target.id.clone()
    }

    async fn run(
        &self,
        db: &DatabaseConnection,
        file: Self::Model,
        ctx: &JobLease,
    ) -> anyhow::Result<JobOutcome> {
        let Some(file_path) = get_job_file_path(db, &file, Self::JOB_KIND).await? else {
            return Ok(JobOutcome::Complete);
        };

        let needs_probe = file_analysis::load_cached_probe(db, &file.id)
            .await?
            .is_none();
        let needs_keyframes = file_analysis::load_cached_keyframes(db, &file.id)
            .await?
            .is_none();

        if !needs_probe && !needs_keyframes {
            return Ok(JobOutcome::Complete);
        }

        if needs_probe {
            let probe = probe_with_cancellation(&file_path, ctx.get_cancellation_token()).await?;
            let Some(probe) = probe else {
                return Ok(JobOutcome::Cancelled);
            };

            let probe_blob =
                encode_probe_data_json_zstd(&probe).context("failed to encode probe payload")?;
            let now = chrono::Utc::now().timestamp();

            file_probe::Entity::insert(file_probe::ActiveModel {
                file_id: Set(file.id.clone()),
                probe: Set(probe_blob),
                generated_at: Set(now),
            })
            .on_conflict(
                OnConflict::column(file_probe::Column::FileId)
                    .update_columns([file_probe::Column::Probe, file_probe::Column::GeneratedAt])
                    .to_owned(),
            )
            .exec(db)
            .await
            .with_context(|| format!("failed storing probe for file {}", file.id))?;

            files::Entity::update(files::ActiveModel {
                id: Set(file.id.clone()),
                subtitles_extracted_at: Set(None),
                ..Default::default()
            })
            .exec(db)
            .await?;
        }

        if needs_keyframes {
            let keyframes = extract_keyframes(&file_path, ctx.get_cancellation_token()).await?;
            let Some(keyframes) = keyframes else {
                return Ok(JobOutcome::Cancelled);
            };

            let payload = json_encoding::encode_json_zstd(&keyframes)
                .context("failed to encode keyframe payload")?;

            files::Entity::update(files::ActiveModel {
                id: Set(file.id.clone()),
                keyframes_json: Set(Some(payload)),
                ..Default::default()
            })
            .exec(db)
            .await?;
        }

        Ok(JobOutcome::Complete)
    }
}
