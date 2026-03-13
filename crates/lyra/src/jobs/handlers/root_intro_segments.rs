use crate::{
    entities::{
        files, item_files, items, jobs as jobs_entity, libraries,
        roots::{self, RootKind},
        seasons,
    },
    jobs::{JobHandler, JobTarget, ROOT_ID_COLUMN, VERSION_KEY_COLUMN, handlers::shared},
    json_encoding,
    segment_markers::{StoredFileSegment, StoredFileSegmentKind, intro_segment_from_range},
};
use anyhow::Context;
use lyra_marker::{
    INTRO_DETECTION_BATCH_MAX_FILES, INTRO_DETECTION_BATCH_MIN_FILES, IntroDetectionInputFile,
    detect_intros,
};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, FromQueryResult, JoinType,
    QueryFilter, QueryOrder, QuerySelect, RelationTrait, sea_query::SelectStatement,
};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

#[derive(Debug, Default)]
pub struct RootIntroSegmentsJob;

#[derive(Clone, Debug)]
struct RootFile {
    file_id: i64,
    file_path: PathBuf,
    audio_fingerprint: Vec<u8>,
    season_id: Option<String>,
    season_order: Option<i64>,
    item_order: i64,
    has_intro_marker: bool,
    pending_segments: bool,
}

#[derive(Debug, FromQueryResult)]
struct RootFileQueryRow {
    file_id: i64,
    relative_path: String,
    library_path: String,
    season_id: Option<String>,
    season_order: Option<i64>,
    item_order: i64,
    audio_fingerprint: Vec<u8>,
    segments_json: Vec<u8>,
}

#[async_trait::async_trait]
impl JobHandler for RootIntroSegmentsJob {
    fn job_kind(&self) -> jobs_entity::JobKind {
        jobs_entity::JobKind::RootGenerateIntroSegments
    }

    fn targets(&self) -> (JobTarget, SelectStatement) {
        let mut query = item_files::Entity::find()
            .join(JoinType::InnerJoin, item_files::Relation::Items.def())
            .join(JoinType::InnerJoin, items::Relation::Roots.def())
            .join(JoinType::InnerJoin, item_files::Relation::Files.def())
            .filter(roots::Column::Kind.eq(RootKind::Series))
            .filter(files::Column::UnavailableAt.is_null())
            .filter(files::Column::SegmentsJson.eq(Vec::<u8>::new()))
            .select_only()
            .column_as(items::Column::RootId, ROOT_ID_COLUMN)
            .column_as(roots::Column::LastAddedAt, VERSION_KEY_COLUMN)
            .distinct()
            .order_by_asc(items::Column::RootId);
        (JobTarget::Root, QuerySelect::query(&mut query).to_owned())
    }

    async fn execute(
        &self,
        pool: &DatabaseConnection,
        job: &jobs_entity::Model,
    ) -> anyhow::Result<()> {
        let root_id = shared::expect_job_root_id(job)?;

        let mut root_files = load_root_files(pool, root_id).await?;
        if root_files.len() < INTRO_DETECTION_BATCH_MIN_FILES {
            return Ok(());
        }

        let mut pending_file_ids = root_files
            .iter()
            .filter_map(|file| {
                if file.pending_segments {
                    Some(file.file_id)
                } else {
                    None
                }
            })
            .collect::<HashSet<_>>();

        while !pending_file_ids.is_empty() {
            let Some(seed) = root_files
                .iter()
                .find(|file| pending_file_ids.contains(&file.file_id))
                .cloned()
            else {
                break;
            };

            let batch = build_intro_batch(&seed, &root_files);
            if batch.len() < INTRO_DETECTION_BATCH_MIN_FILES {
                pending_file_ids.remove(&seed.file_id);
                continue;
            }

            let target_file_ids = batch
                .iter()
                .filter_map(|file| {
                    if pending_file_ids.contains(&file.file_id) {
                        Some(file.file_id)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            if target_file_ids.is_empty() {
                pending_file_ids.remove(&seed.file_id);
                continue;
            }

            let target_file_id_set = target_file_ids.iter().copied().collect::<HashSet<_>>();

            let detection_inputs = batch
                .iter()
                .map(|file| IntroDetectionInputFile {
                    path: file.file_path.clone(),
                    fingerprint_cache: if file.audio_fingerprint.is_empty() {
                        None
                    } else {
                        Some(file.audio_fingerprint.clone())
                    },
                })
                .collect::<Vec<_>>();

            let outcome = tokio::task::spawn_blocking(move || detect_intros(&detection_inputs))
                .await
                .context("intro detection task panicked")?;

            match outcome {
                Ok(detections) => {
                    let detections_by_path = detections
                        .into_iter()
                        .map(|detection| (detection.path.clone(), detection))
                        .collect::<HashMap<_, _>>();

                    for batch_file in &batch {
                        let Some(detection) = detections_by_path.get(&batch_file.file_path) else {
                            continue;
                        };

                        if batch_file.audio_fingerprint != detection.fingerprint_cache {
                            store_audio_fingerprint(
                                pool,
                                batch_file.file_id,
                                &detection.fingerprint_cache,
                            )
                            .await?;
                            if let Some(file) = root_files
                                .iter_mut()
                                .find(|file| file.file_id == batch_file.file_id)
                            {
                                file.audio_fingerprint = detection.fingerprint_cache.clone();
                            }
                        }

                        if !target_file_id_set.contains(&batch_file.file_id) {
                            continue;
                        }

                        let segments = detection
                            .intro
                            .and_then(intro_segment_from_range)
                            .into_iter()
                            .collect::<Vec<_>>();

                        store_segments(pool, batch_file.file_id, &segments).await?;

                        if let Some(file) = root_files
                            .iter_mut()
                            .find(|file| file.file_id == batch_file.file_id)
                        {
                            file.has_intro_marker = segments
                                .iter()
                                .any(|segment| segment.kind == StoredFileSegmentKind::Intro);
                            file.pending_segments = false;
                        }
                    }
                }
                Err(error) => {
                    tracing::warn!(root_id, error = ?error, "intro detection batch failed");
                }
            }

            for file_id in target_file_ids {
                pending_file_ids.remove(&file_id);
            }
        }

        Ok(())
    }
}

async fn load_root_files(
    pool: &DatabaseConnection,
    root_id: &str,
) -> anyhow::Result<Vec<RootFile>> {
    let rows = item_files::Entity::find()
        .join(JoinType::InnerJoin, item_files::Relation::Items.def())
        .join(JoinType::InnerJoin, item_files::Relation::Files.def())
        .join(JoinType::InnerJoin, files::Relation::Libraries.def())
        .join(JoinType::LeftJoin, items::Relation::Seasons.def())
        .filter(items::Column::RootId.eq(root_id.to_string()))
        .filter(files::Column::UnavailableAt.is_null())
        .select_only()
        .column_as(files::Column::Id, "file_id")
        .column_as(files::Column::RelativePath, "relative_path")
        .column_as(libraries::Column::Path, "library_path")
        .column_as(items::Column::SeasonId, "season_id")
        .column_as(seasons::Column::Order, "season_order")
        .column_as(items::Column::Order, "item_order")
        .column_as(files::Column::AudioFingerprint, "audio_fingerprint")
        .column_as(files::Column::SegmentsJson, "segments_json")
        .order_by_asc(seasons::Column::Order)
        .order_by_asc(items::Column::Order)
        .order_by_asc(files::Column::Id)
        .into_model::<RootFileQueryRow>()
        .all(pool)
        .await?;

    let mut unique_rows = Vec::new();
    let mut seen_file_ids = HashSet::new();
    for row in rows {
        if seen_file_ids.insert(row.file_id) {
            unique_rows.push(row);
        }
    }

    let mut output = Vec::with_capacity(unique_rows.len());
    for row in unique_rows {
        let file_path = PathBuf::from(row.library_path).join(row.relative_path);
        let segments = decode_segments_payload(&row.segments_json, row.file_id);
        let has_intro_marker = segments.as_ref().is_some_and(|segments| {
            segments
                .iter()
                .any(|segment| segment.kind == StoredFileSegmentKind::Intro)
        });

        output.push(RootFile {
            file_id: row.file_id,
            file_path,
            audio_fingerprint: row.audio_fingerprint,
            season_id: row.season_id,
            season_order: row.season_order,
            item_order: row.item_order,
            has_intro_marker,
            pending_segments: row.segments_json.is_empty() || segments.is_none(),
        });
    }

    output.sort_by(|left, right| {
        left.season_order
            .unwrap_or(i64::MAX)
            .cmp(&right.season_order.unwrap_or(i64::MAX))
            .then_with(|| left.item_order.cmp(&right.item_order))
            .then_with(|| left.file_id.cmp(&right.file_id))
    });

    Ok(output)
}

fn build_intro_batch(seed: &RootFile, files: &[RootFile]) -> Vec<RootFile> {
    let mut batch = Vec::new();
    let mut selected = HashSet::new();

    for file in files {
        if selected.contains(&file.file_id) {
            continue;
        }
        if file.season_id == seed.season_id && !file.has_intro_marker {
            selected.insert(file.file_id);
            batch.push(file.clone());
        }
        if batch.len() >= INTRO_DETECTION_BATCH_MAX_FILES {
            return batch;
        }
    }

    for file in files {
        if selected.contains(&file.file_id) {
            continue;
        }
        if file.season_id == seed.season_id {
            selected.insert(file.file_id);
            batch.push(file.clone());
        }
        if batch.len() >= INTRO_DETECTION_BATCH_MAX_FILES {
            return batch;
        }
    }

    for file in files {
        if selected.contains(&file.file_id) {
            continue;
        }
        if !file.has_intro_marker {
            selected.insert(file.file_id);
            batch.push(file.clone());
        }
        if batch.len() >= INTRO_DETECTION_BATCH_MAX_FILES {
            return batch;
        }
    }

    for file in files {
        if selected.contains(&file.file_id) {
            continue;
        }
        selected.insert(file.file_id);
        batch.push(file.clone());
        if batch.len() >= INTRO_DETECTION_BATCH_MAX_FILES {
            return batch;
        }
    }

    batch
}

fn decode_segments_payload(payload: &[u8], file_id: i64) -> Option<Vec<StoredFileSegment>> {
    if payload.is_empty() {
        return None;
    }

    match json_encoding::decode_json_zstd::<Vec<StoredFileSegment>>(payload) {
        Ok(segments) => Some(segments),
        Err(error) => {
            tracing::warn!(file_id, error = ?error, "failed to decode file segments payload");
            None
        }
    }
}

async fn store_segments(
    pool: &DatabaseConnection,
    file_id: i64,
    segments: &[StoredFileSegment],
) -> anyhow::Result<()> {
    let payload = json_encoding::encode_json_zstd(&segments)
        .with_context(|| format!("failed to encode intro segments for file {file_id}"))?;

    files::Entity::update(files::ActiveModel {
        id: Set(file_id),
        segments_json: Set(payload),
        ..Default::default()
    })
    .exec(pool)
    .await?;

    Ok(())
}

async fn store_audio_fingerprint(
    pool: &DatabaseConnection,
    file_id: i64,
    fingerprint: &[u8],
) -> anyhow::Result<()> {
    files::Entity::update(files::ActiveModel {
        id: Set(file_id),
        audio_fingerprint: Set(fingerprint.to_vec()),
        ..Default::default()
    })
    .exec(pool)
    .await?;

    Ok(())
}
