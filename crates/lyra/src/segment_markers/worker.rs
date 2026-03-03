use crate::{
    entities::{
        file_segments::{self, FileSegmentsStatus},
        files, item_files, items, libraries,
        roots::{self, RootKind},
        seasons,
    },
    json_encoding,
    segment_markers::{StoredFileSegment, StoredFileSegmentKind, intro_segment_from_range},
};
use anyhow::Context;
use lyra_marker::{
    INTRO_DETECTION_BATCH_MAX_FILES, INTRO_DETECTION_BATCH_MIN_FILES, detect_intros,
};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, Condition, DatabaseConnection, EntityTrait, FromQueryResult,
    JoinType, QueryFilter, QueryOrder, QuerySelect, RelationTrait, sea_query::OnConflict,
};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};
use tokio::time::{Duration, sleep};

const WORKER_INTERVAL: Duration = Duration::from_secs(60);
const MAX_ROOTS_PER_TICK: usize = 3;
const RETRY_BACKOFF_SECONDS: &[i64] = &[30 * 60, 2 * 60 * 60, 24 * 60 * 60];

#[derive(Clone, Debug)]
struct RootFile {
    file_id: i64,
    file_path: PathBuf,
    season_id: Option<String>,
    season_order: Option<i64>,
    item_order: i64,
    has_intro_marker: bool,
    segment_row: Option<file_segments::Model>,
}

#[derive(Debug, FromQueryResult)]
struct RootFileQueryRow {
    file_id: i64,
    relative_path: String,
    library_path: String,
    season_id: Option<String>,
    season_order: Option<i64>,
    item_order: i64,
}

pub async fn start_file_segment_worker(pool: DatabaseConnection) -> anyhow::Result<()> {
    tracing::info!(
        interval_secs = WORKER_INTERVAL.as_secs(),
        "file segment worker started"
    );

    loop {
        let now = chrono::Utc::now().timestamp();
        if let Err(error) = run_tick(&pool, now).await {
            tracing::error!(error = ?error, "file segment worker tick failed");
        }

        sleep(WORKER_INTERVAL).await;
    }
}

async fn run_tick(pool: &DatabaseConnection, now: i64) -> anyhow::Result<()> {
    let pending_root_ids = load_pending_series_root_ids(pool, now).await?;
    for root_id in pending_root_ids.into_iter().take(MAX_ROOTS_PER_TICK) {
        if let Err(error) = process_root(pool, &root_id, now).await {
            tracing::warn!(root_id, error = ?error, "failed processing root intro segments");
        }
    }
    Ok(())
}

async fn load_pending_series_root_ids(
    pool: &DatabaseConnection,
    now: i64,
) -> anyhow::Result<Vec<String>> {
    let roots = item_files::Entity::find()
        .join(JoinType::InnerJoin, item_files::Relation::Items.def())
        .join(JoinType::InnerJoin, items::Relation::Roots.def())
        .join(JoinType::InnerJoin, item_files::Relation::Files.def())
        .join(JoinType::LeftJoin, files::Relation::FileSegments.def())
        .filter(roots::Column::Kind.eq(RootKind::Series))
        .filter(files::Column::UnavailableAt.is_null())
        .filter(files::Column::CorruptedAt.is_null())
        .filter(pending_segment_condition(now))
        .select_only()
        .column(items::Column::RootId)
        .distinct()
        .order_by_asc(items::Column::RootId)
        .into_tuple()
        .all(pool)
        .await?;

    Ok(roots)
}

async fn process_root(pool: &DatabaseConnection, root_id: &str, now: i64) -> anyhow::Result<()> {
    let mut root_files = load_root_files(pool, root_id).await?;
    if root_files.len() < INTRO_DETECTION_BATCH_MIN_FILES {
        return Ok(());
    }

    let mut pending_file_ids = root_files
        .iter()
        .filter_map(|file| {
            if is_pending_segment_row(file.segment_row.as_ref(), now) {
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

        let batch_paths = batch
            .iter()
            .map(|file| file.file_path.clone())
            .collect::<Vec<_>>();

        let outcome = tokio::task::spawn_blocking(move || detect_intros(&batch_paths))
            .await
            .context("intro detection task panicked")?;

        match outcome {
            Ok(detections) => {
                let detections_by_path = detections
                    .into_iter()
                    .map(|detection| (detection.path, detection.intro))
                    .collect::<HashMap<_, _>>();

                for file_id in &target_file_ids {
                    let Some(target_file) = root_files.iter().find(|file| file.file_id == *file_id)
                    else {
                        continue;
                    };

                    let segments = detections_by_path
                        .get(&target_file.file_path)
                        .and_then(|maybe_intro| maybe_intro.as_ref().copied())
                        .and_then(intro_segment_from_range)
                        .into_iter()
                        .collect::<Vec<_>>();

                    upsert_ready_segments(pool, *file_id, &segments, now).await?;

                    if let Some(file) = root_files.iter_mut().find(|file| file.file_id == *file_id)
                    {
                        file.has_intro_marker = segments
                            .iter()
                            .any(|segment| segment.kind == StoredFileSegmentKind::Intro);
                        file.segment_row = Some(file_segments::Model {
                            file_id: *file_id,
                            segment_list: json_encoding::encode_json_zstd(&segments)?,
                            status: FileSegmentsStatus::Ready,
                            attempts: 0,
                            last_attempted_at: Some(now),
                            retry_after: None,
                            last_error_message: None,
                            created_at: now,
                            updated_at: now,
                        });
                    }
                }
            }
            Err(error) => {
                for file_id in &target_file_ids {
                    let prior_attempts = root_files
                        .iter()
                        .find(|file| file.file_id == *file_id)
                        .and_then(|file| file.segment_row.as_ref())
                        .map(|row| row.attempts)
                        .unwrap_or(0);

                    let attempts = prior_attempts + 1;
                    let retry_after = now + retry_backoff_seconds(attempts);
                    upsert_error_segments(pool, *file_id, attempts, retry_after, &error, now)
                        .await?;

                    if let Some(file) = root_files.iter_mut().find(|file| file.file_id == *file_id)
                    {
                        file.has_intro_marker = false;
                        file.segment_row = Some(file_segments::Model {
                            file_id: *file_id,
                            segment_list: json_encoding::encode_json_zstd(
                                &Vec::<StoredFileSegment>::new(),
                            )?,
                            status: FileSegmentsStatus::Error,
                            attempts,
                            last_attempted_at: Some(now),
                            retry_after: Some(retry_after),
                            last_error_message: Some(error.to_string()),
                            created_at: now,
                            updated_at: now,
                        });
                    }
                }
            }
        }

        for file_id in target_file_ids {
            pending_file_ids.remove(&file_id);
        }
    }

    Ok(())
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
        .filter(files::Column::CorruptedAt.is_null())
        .select_only()
        .column_as(files::Column::Id, "file_id")
        .column_as(files::Column::RelativePath, "relative_path")
        .column_as(libraries::Column::Path, "library_path")
        .column_as(items::Column::SeasonId, "season_id")
        .column_as(seasons::Column::Order, "season_order")
        .column_as(items::Column::Order, "item_order")
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

    if unique_rows.is_empty() {
        return Ok(Vec::new());
    }

    let file_ids = unique_rows
        .iter()
        .map(|row| row.file_id)
        .collect::<Vec<_>>();
    let segment_rows = file_segments::Entity::find()
        .filter(file_segments::Column::FileId.is_in(file_ids))
        .all(pool)
        .await?;
    let segments_by_file_id = segment_rows
        .into_iter()
        .map(|row| (row.file_id, row))
        .collect::<HashMap<_, _>>();

    let mut output = Vec::with_capacity(unique_rows.len());
    for row in unique_rows {
        let file_path = PathBuf::from(row.library_path).join(row.relative_path);
        let segment_row = segments_by_file_id.get(&row.file_id).cloned();
        let has_intro_marker = segment_row
            .as_ref()
            .is_some_and(|row| file_has_intro_marker(row, row.file_id));

        output.push(RootFile {
            file_id: row.file_id,
            file_path,
            season_id: row.season_id,
            season_order: row.season_order,
            item_order: row.item_order,
            has_intro_marker,
            segment_row,
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

fn pending_segment_condition(now: i64) -> Condition {
    Condition::any()
        .add(file_segments::Column::FileId.is_null())
        .add(
            Condition::all()
                .add(file_segments::Column::Status.eq(FileSegmentsStatus::Error))
                .add(
                    Condition::any()
                        .add(file_segments::Column::RetryAfter.is_null())
                        .add(file_segments::Column::RetryAfter.lte(now)),
                ),
        )
}

fn is_pending_segment_row(row: Option<&file_segments::Model>, now: i64) -> bool {
    let Some(row) = row else {
        return true;
    };

    row.status == FileSegmentsStatus::Error
        && row.retry_after.is_none_or(|retry_after| retry_after <= now)
}

fn file_has_intro_marker(row: &file_segments::Model, file_id: i64) -> bool {
    if row.status != FileSegmentsStatus::Ready {
        return false;
    }

    match row.decode_segments() {
        Ok(segments) => segments
            .iter()
            .any(|segment| segment.kind == StoredFileSegmentKind::Intro),
        Err(error) => {
            tracing::warn!(file_id, error = ?error, "failed to decode file segments row");
            false
        }
    }
}

async fn upsert_ready_segments(
    pool: &DatabaseConnection,
    file_id: i64,
    segments: &[StoredFileSegment],
    now: i64,
) -> anyhow::Result<()> {
    let payload = json_encoding::encode_json_zstd(&segments)
        .with_context(|| format!("failed to encode intro segments for file {file_id}"))?;

    file_segments::Entity::insert(file_segments::ActiveModel {
        file_id: Set(file_id),
        segment_list: Set(payload),
        status: Set(FileSegmentsStatus::Ready),
        attempts: Set(0),
        last_attempted_at: Set(Some(now)),
        retry_after: Set(None),
        last_error_message: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    })
    .on_conflict(
        OnConflict::column(file_segments::Column::FileId)
            .update_columns([
                file_segments::Column::SegmentList,
                file_segments::Column::Status,
                file_segments::Column::Attempts,
                file_segments::Column::LastAttemptedAt,
                file_segments::Column::RetryAfter,
                file_segments::Column::LastErrorMessage,
                file_segments::Column::UpdatedAt,
            ])
            .to_owned(),
    )
    .exec(pool)
    .await?;

    Ok(())
}

async fn upsert_error_segments(
    pool: &DatabaseConnection,
    file_id: i64,
    attempts: i64,
    retry_after: i64,
    error: &anyhow::Error,
    now: i64,
) -> anyhow::Result<()> {
    let empty_segments = json_encoding::encode_json_zstd(&Vec::<StoredFileSegment>::new())
        .with_context(|| format!("failed to encode empty segment list for file {file_id}"))?;

    file_segments::Entity::insert(file_segments::ActiveModel {
        file_id: Set(file_id),
        segment_list: Set(empty_segments),
        status: Set(FileSegmentsStatus::Error),
        attempts: Set(attempts),
        last_attempted_at: Set(Some(now)),
        retry_after: Set(Some(retry_after)),
        last_error_message: Set(Some(error.to_string())),
        created_at: Set(now),
        updated_at: Set(now),
    })
    .on_conflict(
        OnConflict::column(file_segments::Column::FileId)
            .update_columns([
                file_segments::Column::SegmentList,
                file_segments::Column::Status,
                file_segments::Column::Attempts,
                file_segments::Column::LastAttemptedAt,
                file_segments::Column::RetryAfter,
                file_segments::Column::LastErrorMessage,
                file_segments::Column::UpdatedAt,
            ])
            .to_owned(),
    )
    .exec(pool)
    .await?;

    Ok(())
}

fn retry_backoff_seconds(attempts: i64) -> i64 {
    let index = attempts.saturating_sub(1) as usize;
    RETRY_BACKOFF_SECONDS
        .get(index)
        .copied()
        .unwrap_or(*RETRY_BACKOFF_SECONDS.last().unwrap_or(&(24 * 60 * 60)))
}
