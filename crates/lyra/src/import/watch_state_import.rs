use crate::entities::{files, item_files, items, root_metadata, seasons, watch_progress};
use chrono::Utc;
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
    TransactionTrait,
};
use std::collections::{HashMap, HashSet};

const WRITE_CHUNK_SIZE: usize = 100;
const CONFLICT_EPSILON: f32 = 0.0001;

#[derive(Debug, Clone)]
pub struct ImportWatchStatesRequest {
    pub user_id: String,
    pub overwrite_conflicts: bool,
    pub rows: Vec<ImportWatchStateRow>,
}

#[derive(Debug, Clone)]
pub struct ImportWatchStateRow {
    pub source: String,
    pub source_item_id: Option<String>,
    pub title: Option<String>,
    pub media_type: Option<String>,
    pub season_number: Option<i64>,
    pub episode_number: Option<i64>,
    pub progress_percent: f32,
    pub viewed_at: Option<i64>,
    pub file_path: Option<String>,
    pub file_basename: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub imdb_id: Option<String>,
    pub tmdb_id: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct ImportWatchStateConflictData {
    pub row_index: i32,
    pub source_item_id: Option<String>,
    pub title: Option<String>,
    pub item_id: String,
    pub existing_progress_percent: f32,
    pub imported_progress_percent: f32,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct ImportWatchStateUnmatchedData {
    pub row_index: i32,
    pub source_item_id: Option<String>,
    pub title: Option<String>,
    pub reason: String,
    pub ambiguous: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ImportWatchStatesResultData {
    pub dry_run: bool,
    pub total_rows: i32,
    pub matched_rows: i32,
    pub unmatched_rows: i32,
    pub conflict_rows: i32,
    pub will_insert: i32,
    pub will_overwrite: i32,
    pub imported: i32,
    pub skipped: i32,
    pub conflicts: Vec<ImportWatchStateConflictData>,
    pub unmatched: Vec<ImportWatchStateUnmatchedData>,
}

#[derive(Debug, Clone)]
struct NormalizedImportWatchStateRow {
    source_item_id: Option<String>,
    title: Option<String>,
    source: String,
    media_type: Option<String>,
    season_number: Option<i64>,
    episode_number: Option<i64>,
    progress_percent: f32,
    viewed_at: Option<i64>,
    file_basename: Option<String>,
    file_size_bytes: Option<i64>,
    imdb_id: Option<String>,
    tmdb_id: Option<i64>,
}

#[derive(Debug, Clone)]
struct MatchedRow {
    row_index: usize,
    row: NormalizedImportWatchStateRow,
    item_id: String,
    file_id: i64,
}

#[derive(Debug, Clone)]
struct PendingWrite {
    item_id: String,
    file_id: i64,
    progress_percent: f32,
    viewed_at: Option<i64>,
}

#[derive(Debug, Clone)]
pub enum MatchOutcome {
    Matched { item_id: String, file_id: i64 },
    Unmatched { reason: String, ambiguous: bool },
}

#[derive(Debug, Default)]
struct MatchLookups {
    root_ids_by_tmdb: HashMap<i64, Vec<String>>,
    root_ids_by_imdb: HashMap<String, Vec<String>>,
    file_ids_by_signature: HashMap<(String, i64), Vec<i64>>,
    item_ids_by_file_id: HashMap<i64, Vec<String>>,
    movie_item_ids_by_root_id: HashMap<String, Vec<String>>,
    episode_item_ids_by_root_and_number: HashMap<(String, i64, i64), Vec<String>>,
    fallback_file_id_by_item_id: HashMap<String, i64>,
}

pub async fn dry_run(
    pool: &DatabaseConnection,
    request: ImportWatchStatesRequest,
) -> Result<ImportWatchStatesResultData, sea_orm::DbErr> {
    run_import(pool, request, true).await
}

pub async fn commit(
    pool: &DatabaseConnection,
    request: ImportWatchStatesRequest,
) -> Result<ImportWatchStatesResultData, sea_orm::DbErr> {
    run_import(pool, request, false).await
}

fn match_row(row: &NormalizedImportWatchStateRow, lookups: &MatchLookups) -> MatchOutcome {
    let source = row.source.trim().to_ascii_lowercase();
    if source != "plex" {
        return MatchOutcome::Unmatched {
            reason: "Unsupported source; expected 'plex'".to_string(),
            ambiguous: false,
        };
    }

    if let Some(tmdb_id) = row.tmdb_id {
        if let Some(root_ids) = lookups.root_ids_by_tmdb.get(&tmdb_id) {
            if let Some(single_root_id) = get_single_match(root_ids) {
                return match_row_by_root(row, single_root_id, lookups);
            }

            return MatchOutcome::Unmatched {
                reason: format!("TMDB ID {tmdb_id} matched multiple roots"),
                ambiguous: true,
            };
        }
    }

    if let Some(imdb_id) = row.imdb_id.as_ref() {
        if let Some(root_ids) = lookups.root_ids_by_imdb.get(imdb_id) {
            if let Some(single_root_id) = get_single_match(root_ids) {
                return match_row_by_root(row, single_root_id, lookups);
            }

            return MatchOutcome::Unmatched {
                reason: format!("IMDb ID {imdb_id} matched multiple roots"),
                ambiguous: true,
            };
        }
    }

    let (Some(file_basename), Some(file_size_bytes)) = (&row.file_basename, row.file_size_bytes)
    else {
        return MatchOutcome::Unmatched {
            reason: "Missing file signature for fallback match".to_string(),
            ambiguous: false,
        };
    };

    let signature_key = (file_basename.clone(), file_size_bytes);
    let Some(file_ids) = lookups.file_ids_by_signature.get(&signature_key) else {
        return MatchOutcome::Unmatched {
            reason: "No file matched by basename + size".to_string(),
            ambiguous: false,
        };
    };

    let Some(file_id) = get_single_match(file_ids) else {
        return MatchOutcome::Unmatched {
            reason: "File signature matched multiple files".to_string(),
            ambiguous: true,
        };
    };

    let Some(item_ids) = lookups.item_ids_by_file_id.get(file_id) else {
        return MatchOutcome::Unmatched {
            reason: "Matched file had no linked items".to_string(),
            ambiguous: false,
        };
    };

    let Some(item_id) = get_single_match(item_ids) else {
        return MatchOutcome::Unmatched {
            reason: "Matched file is linked to multiple items".to_string(),
            ambiguous: true,
        };
    };

    MatchOutcome::Matched {
        item_id: item_id.clone(),
        file_id: *file_id,
    }
}

fn get_single_match<T>(values: &[T]) -> Option<&T> {
    if values.len() == 1 {
        values.first()
    } else {
        None
    }
}

fn match_row_by_root(
    row: &NormalizedImportWatchStateRow,
    root_id: &str,
    lookups: &MatchLookups,
) -> MatchOutcome {
    let is_episode = is_episode_like(row);
    let item_id = if is_episode {
        let (Some(season_number), Some(episode_number)) = (row.season_number, row.episode_number)
        else {
            return MatchOutcome::Unmatched {
                reason: "Episode match requires season and episode numbers".to_string(),
                ambiguous: false,
            };
        };

        let key = (root_id.to_string(), season_number, episode_number);
        let Some(item_ids) = lookups.episode_item_ids_by_root_and_number.get(&key) else {
            return MatchOutcome::Unmatched {
                reason: "No episode matched root + season/episode".to_string(),
                ambiguous: false,
            };
        };

        let Some(item_id) = get_single_match(item_ids) else {
            return MatchOutcome::Unmatched {
                reason: "Episode match was ambiguous for root + season/episode".to_string(),
                ambiguous: true,
            };
        };

        item_id
    } else {
        let Some(item_ids) = lookups.movie_item_ids_by_root_id.get(root_id) else {
            return MatchOutcome::Unmatched {
                reason: "No movie item matched root".to_string(),
                ambiguous: false,
            };
        };

        let Some(item_id) = get_single_match(item_ids) else {
            return MatchOutcome::Unmatched {
                reason: "Movie match was ambiguous for root".to_string(),
                ambiguous: true,
            };
        };

        item_id
    };

    let Some(file_id) = lookups.fallback_file_id_by_item_id.get(item_id) else {
        return MatchOutcome::Unmatched {
            reason: "Matched item has no writable file".to_string(),
            ambiguous: false,
        };
    };

    MatchOutcome::Matched {
        item_id: item_id.clone(),
        file_id: *file_id,
    }
}

fn is_episode_like(row: &NormalizedImportWatchStateRow) -> bool {
    if let Some(media_type) = row.media_type.as_ref() {
        let normalized = media_type.trim().to_ascii_lowercase();
        if normalized == "episode" {
            return true;
        }
        if normalized == "movie" {
            return false;
        }
    }

    row.season_number.is_some() || row.episode_number.is_some()
}

fn basename_from_path(path: &str) -> Option<String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return None;
    }

    let candidate = trimmed
        .rsplit(['/', '\\'])
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())?;

    Some(candidate.to_string())
}

fn push_unique<T>(items: &mut Vec<T>, value: T)
where
    T: PartialEq,
{
    if !items.iter().any(|existing| *existing == value) {
        items.push(value);
    }
}

fn normalize_row(row: ImportWatchStateRow) -> NormalizedImportWatchStateRow {
    let progress_percent = watch_progress::normalize_progress_percent(row.progress_percent);

    let file_basename = row
        .file_basename
        .as_deref()
        .and_then(basename_from_path)
        .or_else(|| row.file_path.as_deref().and_then(basename_from_path));

    let imdb_id = row
        .imdb_id
        .and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .or(None);

    NormalizedImportWatchStateRow {
        source_item_id: row.source_item_id,
        title: row.title,
        source: row.source,
        media_type: row.media_type,
        season_number: row.season_number,
        episode_number: row.episode_number,
        progress_percent,
        viewed_at: row.viewed_at,
        file_basename,
        file_size_bytes: row.file_size_bytes,
        imdb_id,
        tmdb_id: row.tmdb_id,
    }
}

async fn run_import(
    pool: &DatabaseConnection,
    request: ImportWatchStatesRequest,
    dry_run: bool,
) -> Result<ImportWatchStatesResultData, sea_orm::DbErr> {
    let normalized_rows = request
        .rows
        .into_iter()
        .map(normalize_row)
        .collect::<Vec<_>>();
    let lookups = load_match_lookups(pool, &normalized_rows).await?;

    let mut matched_rows = Vec::new();
    let mut unmatched_rows = Vec::new();

    for (row_index, row) in normalized_rows.iter().enumerate() {
        match match_row(row, &lookups) {
            MatchOutcome::Matched { item_id, file_id } => matched_rows.push(MatchedRow {
                row_index,
                row: row.clone(),
                item_id,
                file_id,
            }),
            MatchOutcome::Unmatched { reason, ambiguous } => {
                unmatched_rows.push(ImportWatchStateUnmatchedData {
                    row_index: row_index as i32,
                    source_item_id: row.source_item_id.clone(),
                    title: row.title.clone(),
                    reason,
                    ambiguous,
                });
            }
        }
    }

    let matched_item_ids = matched_rows
        .iter()
        .map(|row| row.item_id.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let mut existing_progress_by_item_id = HashMap::new();
    if !matched_item_ids.is_empty() {
        let existing_progress_rows = watch_progress::Entity::find()
            .filter(watch_progress::Column::UserId.eq(request.user_id.clone()))
            .filter(watch_progress::Column::ItemId.is_in(matched_item_ids))
            .all(pool)
            .await?;

        existing_progress_by_item_id.extend(
            existing_progress_rows
                .into_iter()
                .map(|progress| (progress.item_id.clone(), progress)),
        );
    }

    let mut conflicts = Vec::new();
    let mut pending_writes = Vec::new();
    let mut will_insert = 0_i32;
    let mut will_overwrite = 0_i32;
    let mut skipped = 0_i32;

    for matched_row in &matched_rows {
        let Some(existing_progress) = existing_progress_by_item_id.get(&matched_row.item_id) else {
            will_insert += 1;
            pending_writes.push(PendingWrite {
                item_id: matched_row.item_id.clone(),
                file_id: matched_row.file_id,
                progress_percent: matched_row.row.progress_percent,
                viewed_at: matched_row.row.viewed_at,
            });
            continue;
        };

        if (existing_progress.progress_percent - matched_row.row.progress_percent).abs()
            > CONFLICT_EPSILON
        {
            conflicts.push(ImportWatchStateConflictData {
                row_index: matched_row.row_index as i32,
                source_item_id: matched_row.row.source_item_id.clone(),
                title: matched_row.row.title.clone(),
                item_id: matched_row.item_id.clone(),
                existing_progress_percent: existing_progress.progress_percent,
                imported_progress_percent: matched_row.row.progress_percent,
                reason: "Existing watch state differs".to_string(),
            });

            if request.overwrite_conflicts {
                will_overwrite += 1;
                pending_writes.push(PendingWrite {
                    item_id: matched_row.item_id.clone(),
                    file_id: matched_row.file_id,
                    progress_percent: matched_row.row.progress_percent,
                    viewed_at: matched_row.row.viewed_at,
                });
            } else if !dry_run {
                skipped += 1;
            }
        } else if !dry_run {
            skipped += 1;
        }
    }

    let mut imported = 0_i32;
    if !dry_run {
        for chunk in pending_writes.chunks(WRITE_CHUNK_SIZE) {
            let transaction = pool.begin().await?;
            for write in chunk {
                let updated_at = write.viewed_at.unwrap_or_else(|| Utc::now().timestamp());
                watch_progress::Entity::insert(watch_progress::ActiveModel {
                    user_id: Set(request.user_id.clone()),
                    item_id: Set(write.item_id.clone()),
                    file_id: Set(write.file_id),
                    progress_percent: Set(write.progress_percent),
                    updated_at: Set(updated_at),
                    ..Default::default()
                })
                .on_conflict(
                    OnConflict::columns([
                        watch_progress::Column::UserId,
                        watch_progress::Column::ItemId,
                    ])
                    .update_columns([
                        watch_progress::Column::FileId,
                        watch_progress::Column::ProgressPercent,
                        watch_progress::Column::UpdatedAt,
                    ])
                    .to_owned(),
                )
                .exec(&transaction)
                .await?;
            }
            transaction.commit().await?;
            imported += chunk.len() as i32;
        }
    }

    Ok(ImportWatchStatesResultData {
        dry_run,
        total_rows: normalized_rows.len() as i32,
        matched_rows: matched_rows.len() as i32,
        unmatched_rows: unmatched_rows.len() as i32,
        conflict_rows: conflicts.len() as i32,
        will_insert,
        will_overwrite,
        imported,
        skipped,
        conflicts,
        unmatched: unmatched_rows,
    })
}

async fn load_match_lookups(
    pool: &DatabaseConnection,
    rows: &[NormalizedImportWatchStateRow],
) -> Result<MatchLookups, sea_orm::DbErr> {
    let tmdb_ids = rows
        .iter()
        .filter_map(|row| row.tmdb_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let imdb_ids = rows
        .iter()
        .filter_map(|row| row.imdb_id.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let file_sizes = rows
        .iter()
        .filter_map(|row| {
            if row.file_basename.is_some() {
                row.file_size_bytes
            } else {
                None
            }
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let mut lookups = MatchLookups::default();
    let mut matched_root_ids = HashSet::new();

    if !tmdb_ids.is_empty() || !imdb_ids.is_empty() {
        let mut metadata_condition = Condition::any();
        if !tmdb_ids.is_empty() {
            metadata_condition =
                metadata_condition.add(root_metadata::Column::TmdbId.is_in(tmdb_ids));
        }
        if !imdb_ids.is_empty() {
            metadata_condition =
                metadata_condition.add(root_metadata::Column::ImdbId.is_in(imdb_ids));
        }

        let metadata_rows = root_metadata::Entity::find()
            .filter(metadata_condition)
            .all(pool)
            .await?;

        for metadata in metadata_rows {
            if let Some(tmdb_id) = metadata.tmdb_id {
                let root_ids = lookups.root_ids_by_tmdb.entry(tmdb_id).or_default();
                push_unique(root_ids, metadata.root_id.clone());
                matched_root_ids.insert(metadata.root_id.clone());
            }

            if let Some(imdb_id) = metadata.imdb_id {
                let root_ids = lookups.root_ids_by_imdb.entry(imdb_id).or_default();
                push_unique(root_ids, metadata.root_id.clone());
                matched_root_ids.insert(metadata.root_id.clone());
            }
        }
    }

    let mut signature_file_ids = HashSet::new();
    if !file_sizes.is_empty() {
        let candidate_files = files::Entity::find()
            .filter(files::Column::UnavailableAt.is_null())
            .filter(files::Column::SizeBytes.is_in(file_sizes))
            .all(pool)
            .await?;

        for file in candidate_files {
            if let Some(relative_basename) = basename_from_path(&file.relative_path) {
                let signature = (relative_basename, file.size_bytes);
                let file_ids = lookups.file_ids_by_signature.entry(signature).or_default();
                push_unique(file_ids, file.id);
                signature_file_ids.insert(file.id);
            }
        }
    }

    if !signature_file_ids.is_empty() {
        let signature_file_items = item_files::Entity::find()
            .filter(
                item_files::Column::FileId
                    .is_in(signature_file_ids.clone().into_iter().collect::<Vec<_>>()),
            )
            .all(pool)
            .await?;

        for link in signature_file_items {
            let item_ids = lookups.item_ids_by_file_id.entry(link.file_id).or_default();
            push_unique(item_ids, link.item_id);
        }
    }

    let mut root_item_rows = Vec::new();
    if !matched_root_ids.is_empty() {
        root_item_rows = items::Entity::find()
            .filter(
                items::Column::RootId
                    .is_in(matched_root_ids.clone().into_iter().collect::<Vec<_>>()),
            )
            .all(pool)
            .await?;

        let seasons_by_id = seasons::Entity::find()
            .filter(seasons::Column::RootId.is_in(matched_root_ids.into_iter().collect::<Vec<_>>()))
            .all(pool)
            .await?
            .into_iter()
            .map(|season| (season.id.clone(), season))
            .collect::<HashMap<_, _>>();

        for item in &root_item_rows {
            if let Some(primary_file_id) = item.primary_file_id {
                lookups
                    .fallback_file_id_by_item_id
                    .entry(item.id.clone())
                    .or_insert(primary_file_id);
            }

            match item.kind {
                items::ItemKind::Movie => {
                    let item_ids = lookups
                        .movie_item_ids_by_root_id
                        .entry(item.root_id.clone())
                        .or_default();
                    push_unique(item_ids, item.id.clone());
                }
                items::ItemKind::Episode => {
                    let (Some(season_id), Some(episode_number)) =
                        (item.season_id.as_ref(), item.episode_number)
                    else {
                        continue;
                    };
                    let Some(season) = seasons_by_id.get(season_id) else {
                        continue;
                    };

                    let key = (item.root_id.clone(), season.season_number, episode_number);
                    let item_ids = lookups
                        .episode_item_ids_by_root_and_number
                        .entry(key)
                        .or_default();
                    push_unique(item_ids, item.id.clone());
                }
            }
        }
    }

    if !root_item_rows.is_empty() {
        let item_ids = root_item_rows
            .iter()
            .map(|item| item.id.clone())
            .collect::<Vec<_>>();

        let linked_files = item_files::Entity::find()
            .filter(item_files::Column::ItemId.is_in(item_ids))
            .order_by_asc(item_files::Column::Order)
            .all(pool)
            .await?;

        for link in linked_files {
            lookups
                .fallback_file_id_by_item_id
                .entry(link.item_id.clone())
                .or_insert(link.file_id);
        }
    }

    for root_ids in lookups.root_ids_by_tmdb.values_mut() {
        root_ids.sort();
    }
    for root_ids in lookups.root_ids_by_imdb.values_mut() {
        root_ids.sort();
    }
    for file_ids in lookups.file_ids_by_signature.values_mut() {
        file_ids.sort();
    }
    for item_ids in lookups.item_ids_by_file_id.values_mut() {
        item_ids.sort();
    }
    for item_ids in lookups.movie_item_ids_by_root_id.values_mut() {
        item_ids.sort();
    }
    for item_ids in lookups.episode_item_ids_by_root_and_number.values_mut() {
        item_ids.sort();
    }

    Ok(lookups)
}
