use crate::jobs::{Job, JobLease, JobOutcome, JobScheduling};
use crate::{
    assets,
    config::get_config,
    entities::{
        assets as assets_entity, file_probe, file_subtitles,
        file_subtitles::{SubtitleKind, SubtitleSource},
        jobs as jobs_entity,
    },
    ids,
    subtitle_files::{convert_bitmap_subtitle_to_vtt, convert_text_subtitle_to_vtt},
    subtitles::mime_type_for_subtitle_kind,
};
use anyhow::Context;
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::Set,
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Select,
    TransactionTrait,
    sea_query::{Expr, ExprTrait, Func, SimpleExpr},
};

#[derive(Debug, Default)]
pub struct FileSubtitleProcessJob;

#[async_trait::async_trait]
impl Job for FileSubtitleProcessJob {
    type Entity = file_subtitles::Entity;
    type Model = file_subtitles::Model;

    const JOB_KIND: jobs_entity::JobKind = jobs_entity::JobKind::FileProcessSubtitle;
    const SCHEDULING: JobScheduling = JobScheduling::Heavy(2);

    fn query(&self) -> Select<Self::Entity> {
        file_subtitles::Entity::find()
            .filter(file_subtitles::Column::Source.eq(SubtitleSource::Extracted))
            .filter(file_subtitles::Column::ProcessedAt.is_null())
            .filter(file_subtitles::Column::Kind.is_in([
                SubtitleKind::Srt,
                SubtitleKind::Ass,
                SubtitleKind::MovText,
                SubtitleKind::Text,
                SubtitleKind::Ttml,
                SubtitleKind::Pgs,
                SubtitleKind::VobSub,
            ]))
            .order_by_asc(file_subtitles::Column::Id)
    }

    fn target_id(&self, target: &Self::Model) -> String {
        target.id.clone()
    }

    async fn run(
        &self,
        db: &DatabaseConnection,
        row: Self::Model,
        _lease: &JobLease,
    ) -> anyhow::Result<JobOutcome> {
        let asset = assets_entity::Entity::find_by_id(row.asset_id.clone())
            .one(db)
            .await?
            .context("subtitle asset disappeared before processing")?;
        let hash_sha256 = asset
            .hash_sha256
            .as_deref()
            .context("subtitle asset missing local hash")?;
        let mime_type = asset
            .mime_type
            .as_deref()
            .context("subtitle asset missing mime type")?;
        let asset_path = crate::assets::storage::get_asset_output_path_from_mime_and_encoding(
            hash_sha256,
            mime_type,
            asset.content_encoding.as_deref(),
        )?;
        let stored_bytes = tokio::fs::read(&asset_path).await?;
        let source_bytes = match asset.content_encoding.as_deref() {
            Some("zstd") => zstd::decode_all(std::io::Cursor::new(stored_bytes))
                .context("failed to decompress subtitle asset")?,
            Some(other) => anyhow::bail!("unsupported subtitle asset encoding {other}"),
            None => stored_bytes,
        };

        let derived_bytes = match row.kind {
            SubtitleKind::Pgs | SubtitleKind::VobSub => {
                let probe = file_probe::Entity::find_by_id(row.file_id.clone())
                    .one(db)
                    .await?
                    .context("subtitle OCR requires probe data")?
                    .get_probe()
                    .context("failed to decode subtitle OCR probe data")?;
                convert_bitmap_subtitle_to_vtt(
                    &row,
                    &source_bytes,
                    &probe,
                    &get_config().data_dir.join("subtitle-ocr-models"),
                )
                .await?
            }
            _ => {
                let temp_dir = tempfile::tempdir().context("failed to create subtitle temp dir")?;
                let input_path = temp_dir
                    .path()
                    .join(format!("source.{}", extension_for_kind(row.kind)));
                let output_path = temp_dir.path().join("converted.vtt");
                tokio::fs::write(&input_path, &source_bytes).await?;
                convert_text_subtitle_to_vtt(&input_path, &output_path).await?
            }
        };

        let derived_source = match row.kind {
            SubtitleKind::Pgs | SubtitleKind::VobSub => SubtitleSource::Ocr,
            _ => SubtitleSource::Converted,
        };
        let processed_at = chrono::Utc::now().timestamp();

        let tx = db.begin().await?;
        let existing_derived = file_subtitles::Entity::find()
            .filter(file_subtitles::Column::DerivedFromSubtitleId.eq(row.id.clone()))
            .filter(file_subtitles::Column::Kind.eq(SubtitleKind::Vtt))
            .filter(file_subtitles::Column::Source.eq(derived_source))
            .one(&tx)
            .await?;

        let derived_asset = assets::create_local_file_asset_from_bytes(
            &tx,
            &derived_bytes,
            mime_type_for_subtitle_kind(SubtitleKind::Vtt),
            assets_entity::AssetKind::Subtitle,
        )
        .await?;

        if let Some(existing_derived) = existing_derived {
            let mut active: file_subtitles::ActiveModel = existing_derived.into();
            active.asset_id = Set(derived_asset.id);
            active.language_bcp47 = Set(row.language_bcp47.clone());
            active.display_name = Set(row.display_name.clone());
            active.disposition_bits = Set(row.disposition_bits);
            active.last_seen_at = Set(row.last_seen_at);
            active.updated_at = Set(row.last_seen_at);
            active.update(&tx).await?;
        } else {
            file_subtitles::Entity::insert(file_subtitles::ActiveModel {
                id: Set(ids::generate_ulid()),
                file_id: Set(row.file_id.clone()),
                asset_id: Set(derived_asset.id),
                derived_from_subtitle_id: Set(Some(row.id.clone())),
                kind: Set(SubtitleKind::Vtt),
                stream_index: Set(row.stream_index),
                source: Set(derived_source),
                language_bcp47: Set(row.language_bcp47.clone()),
                display_name: Set(row.display_name.clone()),
                disposition_bits: Set(row.disposition_bits),
                last_seen_at: Set(row.last_seen_at),
                processed_at: Set(Some(row.last_seen_at)),
                created_at: Set(row.last_seen_at),
                updated_at: Set(row.last_seen_at),
            })
            .exec(&tx)
            .await?;
        }

        let mut source_active: file_subtitles::ActiveModel = row.into();
        source_active.processed_at = Set(Some(processed_at));
        source_active.updated_at = Set(processed_at);
        source_active.update(&tx).await?;

        tx.commit().await?;
        Ok(JobOutcome::Complete)
    }
}

fn extension_for_kind(kind: SubtitleKind) -> &'static str {
    match kind {
        SubtitleKind::Srt => "srt",
        SubtitleKind::Ass => "ass",
        SubtitleKind::MovText | SubtitleKind::Text => "txt",
        SubtitleKind::Ttml => "ttml",
        SubtitleKind::Vtt => "vtt",
        SubtitleKind::Pgs => "sup",
        SubtitleKind::VobSub => "tar",
    }
}
