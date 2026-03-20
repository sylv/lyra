use crate::entities::{files, node_files, node_metadata, nodes, nodes::NodeKind, watch_progress};
use crate::ids;
use chrono::Utc;
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, JoinType, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait, Set, TransactionTrait,
};
use std::collections::{HashMap, HashSet};

const WRITE_CHUNK_SIZE: usize = 100;
const CONFLICT_EPSILON: f32 = 0.0001;

#[derive(Debug, Clone)]
pub struct ImportWatchStatesRequest {
    pub user_id: String,
    pub accessible_library_ids: Option<Vec<String>>,
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
    node_id: String,
    file_id: String,
}

#[derive(Debug, Clone)]
struct PendingWrite {
    node_id: String,
    file_id: String,
    progress_percent: f32,
    viewed_at: Option<i64>,
}

#[derive(Debug, Clone)]
pub enum MatchOutcome {
    Matched { node_id: String, file_id: String },
    Unmatched { reason: String, ambiguous: bool },
}

#[derive(Debug, Default)]
struct MatchLookups {
    root_ids_by_tmdb: HashMap<i64, Vec<String>>,
    root_ids_by_imdb: HashMap<String, Vec<String>>,
    file_ids_by_signature: HashMap<(String, i64), Vec<String>>,
    node_ids_by_file_id: HashMap<String, Vec<String>>,
    movie_node_ids_by_root_id: HashMap<String, Vec<String>>,
    episode_node_ids_by_root_and_number: HashMap<(String, i64, i64), Vec<String>>,
    fallback_file_id_by_node_id: HashMap<String, String>,
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

fn get_single_match<T>(values: &[T]) -> Option<&T> {
    if values.len() == 1 {
        values.first()
    } else {
        None
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
    let file_basename = row
        .file_basename
        .as_deref()
        .and_then(basename_from_path)
        .or_else(|| row.file_path.as_deref().and_then(basename_from_path));

    let imdb_id = row.imdb_id.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    });

    NormalizedImportWatchStateRow {
        source_item_id: row.source_item_id,
        title: row.title,
        source: row.source,
        media_type: row.media_type,
        season_number: row.season_number,
        episode_number: row.episode_number,
        progress_percent: watch_progress::normalize_progress_percent(row.progress_percent),
        viewed_at: row.viewed_at,
        file_basename,
        file_size_bytes: row.file_size_bytes,
        imdb_id,
        tmdb_id: row.tmdb_id,
    }
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
            if let Some(root_id) = get_single_match(root_ids) {
                return match_row_by_root(row, root_id, lookups);
            }
            return MatchOutcome::Unmatched {
                reason: format!("TMDB ID {tmdb_id} matched multiple roots"),
                ambiguous: true,
            };
        }
    }

    if let Some(imdb_id) = row.imdb_id.as_ref() {
        if let Some(root_ids) = lookups.root_ids_by_imdb.get(imdb_id) {
            if let Some(root_id) = get_single_match(root_ids) {
                return match_row_by_root(row, root_id, lookups);
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

    let Some(node_ids) = lookups.node_ids_by_file_id.get(file_id) else {
        return MatchOutcome::Unmatched {
            reason: "Matched file had no linked nodes".to_string(),
            ambiguous: false,
        };
    };

    let Some(node_id) = get_single_match(node_ids) else {
        return MatchOutcome::Unmatched {
            reason: "Matched file is linked to multiple nodes".to_string(),
            ambiguous: true,
        };
    };

    MatchOutcome::Matched {
        node_id: node_id.clone(),
        file_id: file_id.clone(),
    }
}

fn match_row_by_root(
    row: &NormalizedImportWatchStateRow,
    root_id: &str,
    lookups: &MatchLookups,
) -> MatchOutcome {
    let node_id = if is_episode_like(row) {
        let (Some(season_number), Some(episode_number)) = (row.season_number, row.episode_number)
        else {
            return MatchOutcome::Unmatched {
                reason: "Episode match requires season and episode numbers".to_string(),
                ambiguous: false,
            };
        };

        let key = (root_id.to_string(), season_number, episode_number);
        let Some(node_ids) = lookups.episode_node_ids_by_root_and_number.get(&key) else {
            return MatchOutcome::Unmatched {
                reason: "No episode matched root + season/episode".to_string(),
                ambiguous: false,
            };
        };

        let Some(node_id) = get_single_match(node_ids) else {
            return MatchOutcome::Unmatched {
                reason: "Episode match was ambiguous for root + season/episode".to_string(),
                ambiguous: true,
            };
        };

        node_id
    } else {
        let Some(node_ids) = lookups.movie_node_ids_by_root_id.get(root_id) else {
            return MatchOutcome::Unmatched {
                reason: "No movie node matched root".to_string(),
                ambiguous: false,
            };
        };

        let Some(node_id) = get_single_match(node_ids) else {
            return MatchOutcome::Unmatched {
                reason: "Movie match was ambiguous for root".to_string(),
                ambiguous: true,
            };
        };

        node_id
    };

    let Some(file_id) = lookups.fallback_file_id_by_node_id.get(node_id) else {
        return MatchOutcome::Unmatched {
            reason: "Matched node has no writable file".to_string(),
            ambiguous: false,
        };
    };

    MatchOutcome::Matched {
        node_id: node_id.clone(),
        file_id: file_id.clone(),
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
    let lookups = load_match_lookups(
        pool,
        &normalized_rows,
        request.accessible_library_ids.as_deref(),
    )
    .await?;

    let mut matched_rows = Vec::new();
    let mut unmatched_rows = Vec::new();
    for (row_index, row) in normalized_rows.iter().enumerate() {
        match match_row(row, &lookups) {
            MatchOutcome::Matched { node_id, file_id } => matched_rows.push(MatchedRow {
                row_index,
                row: row.clone(),
                node_id,
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

    let matched_node_ids = matched_rows
        .iter()
        .map(|row| row.node_id.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let mut existing_progress_by_node_id = HashMap::new();
    if !matched_node_ids.is_empty() {
        let rows = watch_progress::Entity::find()
            .filter(watch_progress::Column::UserId.eq(request.user_id.clone()))
            .filter(watch_progress::Column::NodeId.is_in(matched_node_ids))
            .all(pool)
            .await?;
        existing_progress_by_node_id.extend(rows.into_iter().map(|row| (row.node_id.clone(), row)));
    }

    let mut conflicts = Vec::new();
    let mut pending_writes = Vec::new();
    let mut will_insert = 0_i32;
    let mut will_overwrite = 0_i32;
    let mut skipped = 0_i32;

    for matched_row in &matched_rows {
        let Some(existing_progress) = existing_progress_by_node_id.get(&matched_row.node_id) else {
            will_insert += 1;
            pending_writes.push(PendingWrite {
                node_id: matched_row.node_id.clone(),
                file_id: matched_row.file_id.clone(),
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
                item_id: matched_row.node_id.clone(),
                existing_progress_percent: existing_progress.progress_percent,
                imported_progress_percent: matched_row.row.progress_percent,
                reason: "Existing watch state differs".to_string(),
            });

            if request.overwrite_conflicts {
                will_overwrite += 1;
                pending_writes.push(PendingWrite {
                    node_id: matched_row.node_id.clone(),
                    file_id: matched_row.file_id.clone(),
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
                    id: Set(ids::generate_ulid()),
                    user_id: Set(request.user_id.clone()),
                    node_id: Set(write.node_id.clone()),
                    file_id: Set(write.file_id.clone()),
                    progress_percent: Set(write.progress_percent),
                    created_at: Set(updated_at),
                    updated_at: Set(updated_at),
                    ..Default::default()
                })
                .on_conflict(
                    OnConflict::columns([
                        watch_progress::Column::UserId,
                        watch_progress::Column::NodeId,
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

// keep lookup construction concentrated so import matching stays deterministic across rescans.
async fn load_match_lookups(
    pool: &DatabaseConnection,
    rows: &[NormalizedImportWatchStateRow],
    accessible_library_ids: Option<&[String]>,
) -> Result<MatchLookups, sea_orm::DbErr> {
    let tmdb_ids = rows
        .iter()
        .filter_map(|row| row.tmdb_id)
        .collect::<HashSet<_>>();
    let imdb_ids = rows
        .iter()
        .filter_map(|row| row.imdb_id.clone())
        .collect::<HashSet<_>>();
    let file_sizes = rows
        .iter()
        .filter_map(|row| row.file_basename.as_ref().and(row.file_size_bytes))
        .collect::<HashSet<_>>();

    let mut lookups = MatchLookups::default();
    let mut matched_root_ids = HashSet::new();

    if !tmdb_ids.is_empty() || !imdb_ids.is_empty() {
        let mut metadata_condition = Condition::any();
        if !tmdb_ids.is_empty() {
            metadata_condition =
                metadata_condition.add(node_metadata::Column::TmdbId.is_in(tmdb_ids));
        }
        if !imdb_ids.is_empty() {
            metadata_condition =
                metadata_condition.add(node_metadata::Column::ImdbId.is_in(imdb_ids));
        }

        let mut metadata_query = node_metadata::Entity::find()
            .join(JoinType::InnerJoin, node_metadata::Relation::Nodes.def())
            .filter(metadata_condition)
            .filter(nodes::Column::ParentId.is_null());
        if let Some(library_ids) = accessible_library_ids {
            metadata_query =
                metadata_query.filter(nodes::Column::LibraryId.is_in(library_ids.to_vec()));
        }
        let metadata_rows = metadata_query.all(pool).await?;

        for metadata in metadata_rows {
            if let Some(tmdb_id) = metadata.tmdb_id {
                let root_ids = lookups.root_ids_by_tmdb.entry(tmdb_id).or_default();
                push_unique(root_ids, metadata.node_id.clone());
                matched_root_ids.insert(metadata.node_id.clone());
            }
            if let Some(imdb_id) = metadata.imdb_id {
                let root_ids = lookups.root_ids_by_imdb.entry(imdb_id).or_default();
                push_unique(root_ids, metadata.node_id.clone());
                matched_root_ids.insert(metadata.node_id.clone());
            }
        }
    }

    if !file_sizes.is_empty() {
        let mut candidate_files_query = files::Entity::find()
            .filter(files::Column::UnavailableAt.is_null())
            .filter(files::Column::SizeBytes.is_in(file_sizes));
        if let Some(library_ids) = accessible_library_ids {
            candidate_files_query =
                candidate_files_query.filter(files::Column::LibraryId.is_in(library_ids.to_vec()));
        }
        let candidate_files = candidate_files_query.all(pool).await?;

        for file in candidate_files {
            if let Some(relative_basename) = basename_from_path(&file.relative_path) {
                let signature = (relative_basename, file.size_bytes);
                let file_ids = lookups.file_ids_by_signature.entry(signature).or_default();
                push_unique(file_ids, file.id);
            }
        }
    }

    let signature_file_ids = lookups
        .file_ids_by_signature
        .values()
        .flat_map(|ids| ids.iter().cloned())
        .collect::<HashSet<_>>();

    if !signature_file_ids.is_empty() {
        let signature_links = node_files::Entity::find()
            .filter(
                node_files::Column::FileId
                    .is_in(signature_file_ids.into_iter().collect::<Vec<_>>()),
            )
            .all(pool)
            .await?;

        for link in signature_links {
            let node_ids = lookups.node_ids_by_file_id.entry(link.file_id).or_default();
            push_unique(node_ids, link.node_id);
        }
    }

    if !matched_root_ids.is_empty() {
        let mut playable_nodes_query = nodes::Entity::find()
            .filter(
                nodes::Column::RootId
                    .is_in(matched_root_ids.clone().into_iter().collect::<Vec<_>>()),
            )
            .filter(nodes::Column::Kind.is_in([NodeKind::Movie, NodeKind::Episode]));
        if let Some(library_ids) = accessible_library_ids {
            playable_nodes_query =
                playable_nodes_query.filter(nodes::Column::LibraryId.is_in(library_ids.to_vec()));
        }
        let playable_nodes = playable_nodes_query.all(pool).await?;

        let mut seasons_query = nodes::Entity::find()
            .filter(nodes::Column::RootId.is_in(matched_root_ids.into_iter().collect::<Vec<_>>()))
            .filter(nodes::Column::Kind.eq(NodeKind::Season));
        if let Some(library_ids) = accessible_library_ids {
            seasons_query =
                seasons_query.filter(nodes::Column::LibraryId.is_in(library_ids.to_vec()));
        }
        let seasons_by_id = seasons_query
            .all(pool)
            .await?
            .into_iter()
            .map(|node| (node.id.clone(), node))
            .collect::<HashMap<_, _>>();

        for node in &playable_nodes {
            match node.kind {
                NodeKind::Movie => {
                    let node_ids = lookups
                        .movie_node_ids_by_root_id
                        .entry(node.root_id.clone())
                        .or_default();
                    push_unique(node_ids, node.id.clone());
                }
                NodeKind::Episode => {
                    let Some(episode_number) = node.episode_number else {
                        continue;
                    };
                    let season_number = node
                        .parent_id
                        .as_ref()
                        .and_then(|parent_id| seasons_by_id.get(parent_id))
                        .and_then(|season| season.season_number)
                        .unwrap_or(0);
                    let key = (node.root_id.clone(), season_number, episode_number);
                    let node_ids = lookups
                        .episode_node_ids_by_root_and_number
                        .entry(key)
                        .or_default();
                    push_unique(node_ids, node.id.clone());
                }
                _ => {}
            }
        }

        let playable_ids = playable_nodes
            .into_iter()
            .map(|node| node.id)
            .collect::<Vec<_>>();
        let linked_files = node_files::Entity::find()
            .filter(node_files::Column::NodeId.is_in(playable_ids))
            .join(JoinType::InnerJoin, node_files::Relation::Files.def())
            .filter(files::Column::UnavailableAt.is_null())
            .order_by_asc(node_files::Column::Order)
            .order_by_asc(node_files::Column::FileId)
            .all(pool)
            .await?;

        for link in linked_files {
            lookups
                .fallback_file_id_by_node_id
                .entry(link.node_id.clone())
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
    for node_ids in lookups.node_ids_by_file_id.values_mut() {
        node_ids.sort();
    }
    for node_ids in lookups.movie_node_ids_by_root_id.values_mut() {
        node_ids.sort();
    }
    for node_ids in lookups.episode_node_ids_by_root_and_number.values_mut() {
        node_ids.sort();
    }

    Ok(lookups)
}
