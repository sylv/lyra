use crate::{
    entities::{
        file_probe, files, jobs as jobs_entity, libraries, node_files, nodes, nodes::NodeKind,
    },
    file_analysis,
    json_encoding,
    jobs::{Job, JobLease, JobOutcome},
    segment_markers::{StoredFileSegment, StoredFileSegmentKind, intro_segment_from_range},
};
use anyhow::Context;
use lyra_marker::{
    INTRO_DETECTION_BATCH_MAX_FILES, INTRO_DETECTION_BATCH_MIN_FILES, IntroDetectionInputFile,
    detect_intros,
};
use lyra_probe::ProbeData;
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
    probe_data: ProbeData,
    audio_fingerprint: Option<Vec<u8>>,
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
    audio_fingerprint: Option<Vec<u8>>,
    segments_json: Option<Vec<u8>>,
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
                            Expr::col((files::Entity, files::Column::SegmentsJson)).is_null(),
                        )
                        .to_owned(),
                ),
            )
            .filter(
                Expr::col(nodes::Column::Id).not_in_subquery(
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
                            Expr::col((files::Entity, files::Column::Id)).not_in_subquery(
                                Query::select()
                                    .column(file_probe::Column::FileId)
                                    .from(file_probe::Entity)
                                    .to_owned(),
                            ),
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
                    probe_data: file.probe_data.clone(),
                    fingerprint_cache: file.audio_fingerprint.clone(),
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

                let next_fingerprint = (!detection.fingerprint_cache.is_empty())
                    .then_some(detection.fingerprint_cache.clone());
                if batch_file.audio_fingerprint != next_fingerprint {
                    store_audio_fingerprint(db, &batch_file.file_id, next_fingerprint.as_deref())
                        .await?;
                    if let Some(file) = root_files
                        .iter_mut()
                        .find(|file| file.file_id == batch_file.file_id)
                    {
                        file.audio_fingerprint = next_fingerprint;
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
    db: &DatabaseConnection,
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
        let file_id = row.file_id.clone();
        let file_path = PathBuf::from(row.library_path).join(row.relative_path);
        let segments = decode_segments_payload(row.segments_json.as_deref(), &file_id);
        let has_intro_marker = segments.as_ref().is_some_and(|segments| {
            segments
                .iter()
                .any(|segment| segment.kind == StoredFileSegmentKind::Intro)
        });

        output.push(RootFile {
            file_id,
            file_path,
            probe_data: file_analysis::load_cached_probe(db, &row.file_id)
                .await?
                .with_context(|| format!("missing cached probe data for file {}", row.file_id))?,
            audio_fingerprint: row.audio_fingerprint,
            season_id: row.season_id,
            item_order: row.item_order,
            has_intro_marker,
            pending_segments: row.segments_json.is_none(),
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
    let mut batch = vec![seed.clone()];
    let mut selected = HashSet::from([seed.file_id.clone()]);

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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_file(file_id: &str, season_id: &str, item_order: i64) -> RootFile {
        RootFile {
            file_id: file_id.to_owned(),
            file_path: PathBuf::from(format!("/tmp/{file_id}.mkv")),
            probe_data: ProbeData {
                duration_secs: Some(1.0),
                overall_bit_rate: None,
                streams: Vec::new(),
            },
            audio_fingerprint: None,
            season_id: Some(season_id.to_owned()),
            item_order,
            has_intro_marker: false,
            pending_segments: true,
        }
    }

    #[test]
    fn build_intro_batch_keeps_seed_when_season_exceeds_batch_limit() {
        let season_id = "season-32";
        let files = (0..22)
            .map(|idx| test_file(&format!("file-{idx:02}"), season_id, idx))
            .collect::<Vec<_>>();

        let seed = files[21].clone();
        let batch = build_intro_batch(&seed, &files);

        assert_eq!(batch.len(), INTRO_DETECTION_BATCH_MAX_FILES);
        assert_eq!(batch[0].file_id, seed.file_id);
        assert!(batch.iter().any(|file| file.file_id == seed.file_id));
    }
}

fn decode_segments_payload(
    payload: Option<&[u8]>,
    file_id: &str,
) -> Option<Vec<StoredFileSegment>> {
    let payload = payload?;

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
        segments_json: Set(Some(payload)),
        ..Default::default()
    })
    .exec(db)
    .await?;

    Ok(())
}

async fn store_audio_fingerprint(
    db: &impl ConnectionTrait,
    file_id: &str,
    fingerprint: Option<&[u8]>,
) -> anyhow::Result<()> {
    files::Entity::update(files::ActiveModel {
        id: Set(file_id.to_string()),
        audio_fingerprint: Set(fingerprint.map(ToOwned::to_owned)),
        ..Default::default()
    })
    .exec(db)
    .await?;

    Ok(())
}
