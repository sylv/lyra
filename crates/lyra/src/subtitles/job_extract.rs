use crate::jobs::{Job, JobLease, JobOutcome, JobScheduling};
use crate::media::{get_job_file_path, load_cached_probe};
use crate::{
    entities::{file_probe, file_subtitles, files, jobs as jobs_entity},
    subtitles::files::{
        can_extract_webvtt_subtitle, extract_webvtt_subtitle_batch,
        subtitle_descriptor_from_stream, upsert_extracted_subtitle,
    },
};
use anyhow::Context;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, Condition, DatabaseConnection, EntityTrait, ExprTrait,
    QueryFilter, QueryOrder, Select, TransactionTrait,
};

#[derive(Debug, Default)]
pub struct FileSubtitleExtractJob;

#[async_trait::async_trait]
impl Job for FileSubtitleExtractJob {
    type Entity = files::Entity;
    type Model = files::Model;

    const JOB_KIND: jobs_entity::JobKind = jobs_entity::JobKind::FileExtractSubtitles;
    const SCHEDULING: JobScheduling = JobScheduling::Heavy(1);

    fn query(&self) -> Select<Self::Entity> {
        files::Entity::find()
            .filter(files::Column::UnavailableAt.is_null())
            .filter(files::Column::SubtitlesExtractedAt.is_null())
            .filter(
                Condition::all()
                    .add(
                        sea_orm::sea_query::Expr::col((files::Entity, files::Column::Id))
                            .in_subquery(
                                sea_orm::sea_query::Query::select()
                                    .column(file_probe::Column::FileId)
                                    .from(file_probe::Entity)
                                    .to_owned(),
                            ),
                    )
                    .add(
                        sea_orm::sea_query::Expr::col((files::Entity, files::Column::Id))
                            .not_in_subquery(
                                sea_orm::sea_query::Query::select()
                                    .column(jobs_entity::Column::TargetId)
                                    .from(jobs_entity::Entity)
                                    .and_where(
                                        jobs_entity::Column::JobKind
                                            .eq(jobs_entity::JobKind::FileProbe.code()),
                                    )
                                    .and_where(
                                        jobs_entity::Column::State
                                            .eq(jobs_entity::JobState::Running),
                                    )
                                    .to_owned(),
                            ),
                    ),
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

        let Some(probe_data) = load_cached_probe(db, &file.id).await? else {
            tracing::info!(
                file_id = %file.id,
                "deferring subtitle extraction until cached probe data is available"
            );
            file_probe::Entity::delete_by_id(file.id.clone())
                .exec(db)
                .await?;
            return Ok(JobOutcome::Cancelled);
        };
        let now = chrono::Utc::now().timestamp();

        let existing = file_subtitles::Entity::find()
            .filter(file_subtitles::Column::FileId.eq(file.id.clone()))
            .all(db)
            .await?;

        let mut descriptors = Vec::new();
        let mut pending_extractions = Vec::new();
        for stream in probe_data
            .streams
            .iter()
            .filter(|stream| stream.kind() == lyra_probe::StreamKind::Subtitle)
        {
            if !can_extract_webvtt_subtitle(stream) {
                tracing::warn!(
                    file_id = %file.id,
                    stream_index = stream.index,
                    codec = %stream.codec,
                    "skipping subtitle stream that cannot be extracted to WebVTT"
                );
                continue;
            }

            let Some(descriptor) = subtitle_descriptor_from_stream(stream) else {
                continue;
            };

            let existing_row = existing
                .iter()
                .find(|row| row.source_stream_index == descriptor.source_stream_index);
            pending_extractions.push((stream, descriptor.clone()));
            descriptors.push((descriptor, existing_row.cloned()));
        }

        let extracted_bytes = extract_webvtt_subtitle_batch(
            &file_path,
            pending_extractions
                .iter()
                .map(|(stream, descriptor)| (*stream, descriptor)),
            ctx.get_cancellation_token(),
        )
        .await?;
        let Some(extracted_bytes) = extracted_bytes else {
            return Ok(JobOutcome::Cancelled);
        };
        if ctx.is_cancelled() {
            return Ok(JobOutcome::Cancelled);
        }

        let tx = db.begin().await?;
        for (descriptor, existing_row) in descriptors {
            if ctx.is_cancelled() {
                return Ok(JobOutcome::Cancelled);
            }
            let bytes = extracted_bytes
                .get(&descriptor.source_stream_index)
                .context("missing extracted subtitle bytes for stream")?;
            upsert_extracted_subtitle(
                &tx,
                &file.id,
                existing_row.as_ref(),
                &descriptor,
                bytes,
                now,
            )
            .await?;
        }

        files::Entity::update(files::ActiveModel {
            id: Set(file.id),
            subtitles_extracted_at: Set(Some(now)),
            ..Default::default()
        })
        .exec(&tx)
        .await?;

        tx.commit().await?;
        Ok(JobOutcome::Complete)
    }
}
