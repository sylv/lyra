use crate::jobs::{Job, JobLease, JobOutcome};
use crate::{
    entities::{files, jobs as jobs_entity, libraries, node_files, nodes, nodes::NodeKind},
    json_encoding,
    segment_markers::{StoredFileSegment, StoredFileSegmentKind, intro_segment_from_range},
};
use anyhow::Context;
use lyra_marker::{
    INTRO_DETECTION_BATCH_MAX_FILES, INTRO_DETECTION_BATCH_MIN_FILES, IntroDetectionInputFile,
    detect_intros,
};
use sea_orm::{
    ActiveValue::Set,
    ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, FromQueryResult, JoinType,
    QueryFilter, QueryOrder, QuerySelect, RelationTrait, Select,
    sea_query::{Expr, Query},
};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

#[derive(Debug, Default)]
pub struct RootIntroSegmentsJob;

#[derive(Clone, Debug)]
struct RootFile {
    file_id: String,
    file_path: PathBuf,
    audio_fingerprint: Vec<u8>,
    season_id: Option<String>,
    item_order: i64,
    has_intro_marker: bool,
    pending_segments: bool,
}

#[derive(Debug, FromQueryResult)]
struct RootFileQueryRow {
    file_id: String,
    relative_path: String,
    library_path: String,
    season_id: Option<String>,
    item_order: i64,
    audio_fingerprint: Vec<u8>,
    segments_json: Vec<u8>,
}

#[async_trait::async_trait]
impl Job for RootIntroSegmentsJob {
    type Entity = nodes::Entity;
    type Model = nodes::Model;

    const JOB_KIND: jobs_entity::JobKind = jobs_entity::JobKind::NodeGenerateIntroSegments;
    const IS_HEAVY: bool = true;

    fn query(&self) -> Select<Self::Entity> {
        nodes::Entity::find()
            .filter(nodes::Column::ParentId.is_null())
            .filter(nodes::Column::Kind.eq(NodeKind::Series))
            .filter(
                Expr::col(nodes::Column::Id).in_subquery(
                    Query::select()
                        .column(nodes::Column::RootId)
                        .from(node_files::Entity)
                        .inner_join(
                            nodes::Entity,
                            Expr::col((node_files::Entity, node_files::Column::NodeId))
                                .equals((nodes::Entity, nodes::Column::Id)),
                        )
                        .inner_join(
                            files::Entity,
                            Expr::col((node_files::Entity, node_files::Column::FileId))
                                .equals((files::Entity, files::Column::Id)),
                        )
                        .and_where(
                            Expr::col((nodes::Entity, nodes::Column::Kind)).eq(NodeKind::Episode),
                        )
                        .and_where(
                            Expr::col((files::Entity, files::Column::UnavailableAt)).is_null(),
                        )
                        .and_where(
                            Expr::col((files::Entity, files::Column::SegmentsJson))
                                .eq(Vec::<u8>::new()),
                        )
                        .to_owned(),
                ),
            )
            .order_by_asc(nodes::Column::Id)
    }

    fn target_id(&self, target: &Self::Model) -> String {
        target.id.clone()
    }

    async fn run(
        &self,
        db: &DatabaseConnection,
        root: Self::Model,
        ctx: &JobLease,
    ) -> anyhow::Result<JobOutcome> {
        let mut root_files = load_root_files(db, &root.id).await?;
        if root_files.len() < INTRO_DETECTION_BATCH_MIN_FILES {
            return Ok(JobOutcome::Complete);
        }

        let mut pending_file_ids = root_files
            .iter()
            .filter_map(|file| file.pending_segments.then(|| file.file_id.clone()))
            .collect::<HashSet<_>>();

        while !pending_file_ids.is_empty() {
            if ctx.is_cancelled() {
                return Ok(JobOutcome::Cancelled);
            }

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
                    pending_file_ids
                        .contains(&file.file_id)
                        .then(|| file.file_id.clone())
                })
                .collect::<Vec<_>>();

            if target_file_ids.is_empty() {
                pending_file_ids.remove(&seed.file_id);
                continue;
            }

            let target_file_id_set = target_file_ids.iter().cloned().collect::<HashSet<_>>();
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

            let Some(detections) =
                detect_intros(&detection_inputs, ctx.get_cancellation_token()).await?
            else {
                return Ok(JobOutcome::Cancelled);
            };

            let detections_by_path = detections
                .into_iter()
                .map(|detection| (detection.path.clone(), detection))
                .collect::<HashMap<_, _>>();

            for batch_file in &batch {
                let Some(detection) = detections_by_path.get(&batch_file.file_path) else {
                    continue;
                };

                if batch_file.audio_fingerprint != detection.fingerprint_cache {
                    store_audio_fingerprint(db, &batch_file.file_id, &detection.fingerprint_cache)
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

                store_segments(db, &batch_file.file_id, &segments).await?;

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

            for file_id in target_file_ids {
                pending_file_ids.remove(&file_id);
            }
        }

        Ok(JobOutcome::Complete)
    }
}

async fn load_root_files(
    db: &impl ConnectionTrait,
    root_id: &str,
) -> anyhow::Result<Vec<RootFile>> {
    let rows = node_files::Entity::find()
        .join(JoinType::InnerJoin, node_files::Relation::Nodes.def())
        .join(JoinType::InnerJoin, node_files::Relation::Files.def())
        .join(JoinType::InnerJoin, files::Relation::Libraries.def())
        .filter(nodes::Column::RootId.eq(root_id.to_string()))
        .filter(nodes::Column::Kind.eq(NodeKind::Episode))
        .filter(files::Column::UnavailableAt.is_null())
        .select_only()
        .column_as(files::Column::Id, "file_id")
        .column_as(files::Column::RelativePath, "relative_path")
        .column_as(libraries::Column::Path, "library_path")
        .column_as(nodes::Column::ParentId, "season_id")
        .column_as(nodes::Column::Order, "item_order")
        .column_as(files::Column::AudioFingerprint, "audio_fingerprint")
        .column_as(files::Column::SegmentsJson, "segments_json")
        .order_by_asc(nodes::Column::Order)
        .order_by_asc(files::Column::Id)
        .into_model::<RootFileQueryRow>()
        .all(db)
        .await?;

    let mut unique_rows = Vec::new();
    let mut seen_file_ids = HashSet::new();
    for row in rows {
        if seen_file_ids.insert(row.file_id.clone()) {
            unique_rows.push(row);
        }
    }

    let mut output = Vec::with_capacity(unique_rows.len());
    for row in unique_rows {
        let file_path = PathBuf::from(row.library_path).join(row.relative_path);
        let segments = decode_segments_payload(&row.segments_json, &row.file_id);
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
            item_order: row.item_order,
            has_intro_marker,
            pending_segments: row.segments_json.is_empty() || segments.is_none(),
        });
    }

    output.sort_by(|left, right| {
        left.season_id
            .cmp(&right.season_id)
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
            selected.insert(file.file_id.clone());
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
            selected.insert(file.file_id.clone());
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
            selected.insert(file.file_id.clone());
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
        selected.insert(file.file_id.clone());
        batch.push(file.clone());
        if batch.len() >= INTRO_DETECTION_BATCH_MAX_FILES {
            return batch;
        }
    }

    batch
}

fn decode_segments_payload(payload: &[u8], file_id: &str) -> Option<Vec<StoredFileSegment>> {
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
    db: &impl ConnectionTrait,
    file_id: &str,
    segments: &[StoredFileSegment],
) -> anyhow::Result<()> {
    let payload = json_encoding::encode_json_zstd(&segments)
        .with_context(|| format!("failed to encode intro segments for file {file_id}"))?;

    files::Entity::update(files::ActiveModel {
        id: Set(file_id.to_string()),
        segments_json: Set(payload),
        ..Default::default()
    })
    .exec(db)
    .await?;

    Ok(())
}

async fn store_audio_fingerprint(
    db: &impl ConnectionTrait,
    file_id: &str,
    fingerprint: &[u8],
) -> anyhow::Result<()> {
    files::Entity::update(files::ActiveModel {
        id: Set(file_id.to_string()),
        audio_fingerprint: Set(fingerprint.to_vec()),
        ..Default::default()
    })
    .exec(db)
    .await?;

    Ok(())
}
