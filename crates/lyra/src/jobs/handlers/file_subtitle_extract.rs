use crate::jobs::handlers::shared::get_job_file_path;
use crate::jobs::{Job, JobLease, JobOutcome, JobScheduling};
use crate::{
    entities::{
        file_probe, file_subtitles, file_subtitles::SubtitleSource, files, jobs as jobs_entity,
    },
    file_analysis,
    subtitle_files::{
        extract_subtitle_bytes_batch, refresh_derived_subtitles_last_seen,
        refresh_extracted_subtitle_metadata, subtitle_descriptor_from_stream,
        upsert_extracted_subtitle,
    },
};
use anyhow::Context;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, Select, TransactionTrait,
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
                Condition::all().add(
                    sea_orm::sea_query::Expr::col((files::Entity, files::Column::Id)).in_subquery(
                        sea_orm::sea_query::Query::select()
                            .column(file_probe::Column::FileId)
                            .from(file_probe::Entity)
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
        _ctx: &JobLease,
    ) -> anyhow::Result<JobOutcome> {
        let Some(file_path) = get_job_file_path(db, &file, Self::JOB_KIND).await? else {
            return Ok(JobOutcome::Complete);
        };

        let probe_data = file_analysis::load_cached_probe(db, &file.id)
            .await?
            .context("subtitle extraction requires cached probe data")?;
        let now = chrono::Utc::now().timestamp();

        let existing = file_subtitles::Entity::find()
            .filter(file_subtitles::Column::FileId.eq(file.id.clone()))
            .filter(file_subtitles::Column::Source.eq(SubtitleSource::Extracted))
            .all(db)
            .await?;

        let mut descriptors = Vec::new();
        let mut pending_extractions = Vec::new();
        for stream in probe_data
            .streams
            .iter()
            .filter(|stream| stream.kind() == lyra_probe::StreamKind::Subtitle)
        {
            let Some(descriptor) = subtitle_descriptor_from_stream(stream) else {
                continue;
            };

            let existing_row = existing.iter().find(|row| {
                row.stream_index == descriptor.stream_index
                    && row.derived_from_subtitle_id.is_none()
            });
            if existing_row.is_none() {
                pending_extractions.push((stream, descriptor.clone()));
            }
            descriptors.push((descriptor, existing_row.cloned()));
        }

        let extracted_bytes = extract_subtitle_bytes_batch(
            &file_path,
            pending_extractions
                .iter()
                .map(|(stream, descriptor)| (*stream, descriptor)),
        )
        .await?;

        let tx = db.begin().await?;
        for (descriptor, existing_row) in descriptors {
            let source_row = if let Some(row) = existing_row {
                refresh_extracted_subtitle_metadata(&tx, &row, &descriptor, now).await?
            } else {
                let bytes = extracted_bytes
                    .get(&descriptor.stream_index)
                    .context("missing extracted subtitle bytes for stream")?;
                upsert_extracted_subtitle(&tx, &file.id, &descriptor, &bytes, now).await?
            };

            refresh_derived_subtitles_last_seen(&tx, &source_row.id, &descriptor, now).await?;
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
