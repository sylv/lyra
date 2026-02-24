use crate::entities::{
    files, item_files, item_node_matches, items, node_match_status::NodeMatchStatus,
    root_node_matches, roots, seasons,
};
use crate::metadata::shared::{
    ItemMatchRowInput, MAX_HINT_FILES, MAX_ITEMS_PER_TICK, MAX_ROOTS_PER_TICK, RootMatchRowInput,
    next_attempts, retry_backoff_seconds,
};
use crate::metadata::store::{
    clear_remote_item_metadata_for_batch, overwrite_remote_item_metadata_for_batch,
    overwrite_remote_movie_metadata_for_batch, overwrite_remote_season_metadata_for_batch,
    upsert_item_match_rows, upsert_remote_root_metadata_from_movie,
    upsert_remote_root_metadata_from_series, upsert_root_match_row,
};
use anyhow::Context;
use lyra_metadata::{
    EpisodeMetadata, MetadataProvider, MovieMetadata, MovieRootMatchRequest, RootMatchHint,
    SeasonMetadata, SeriesItem, SeriesItemsRequest, SeriesMetadata, SeriesRootMatchRequest,
};
use lyra_parser::parse_files;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, JoinType, QueryFilter, QueryOrder, QuerySelect,
    RelationTrait,
};
use std::collections::{HashMap, HashSet};

pub async fn process_roots(
    pool: &DatabaseConnection,
    provider: &dyn MetadataProvider,
    now: i64,
) -> anyhow::Result<bool> {
    let provider_id = provider.id();
    let mut changed = false;
    let root_rows = roots::Entity::find()
        .order_by_asc(roots::Column::LastAddedAt)
        .order_by_asc(roots::Column::Id)
        .limit(MAX_ROOTS_PER_TICK as u64)
        .all(pool)
        .await?;

    for root in root_rows {
        let existing = root_node_matches::Entity::find()
            .filter(root_node_matches::Column::RootId.eq(root.id.clone()))
            .filter(root_node_matches::Column::ProviderId.eq(provider_id))
            .one(pool)
            .await?;

        if !should_attempt_root_match(&root, existing.as_ref(), now) {
            continue;
        }

        let attempts = existing.as_ref().map(|row| row.attempts + 1).unwrap_or(1);
        let match_result = run_root_match_attempt(pool, provider, &root).await;
        match match_result {
            Ok(RootMatchAttempt::MatchedSeries(metadata)) => {
                upsert_remote_root_metadata_from_series(
                    pool,
                    &root.id,
                    provider_id,
                    &metadata,
                    now,
                )
                .await?;
                upsert_root_match_row(
                    pool,
                    RootMatchRowInput {
                        root_id: root.id.clone(),
                        provider_id: provider_id.to_string(),
                        status: NodeMatchStatus::Matched,
                        last_attempted_at: Some(now),
                        last_added_at: Some(root.last_added_at),
                        last_error_message: None,
                        retry_after: None,
                        attempts,
                        created_at: now,
                        updated_at: now,
                    },
                )
                .await?;
                tracing::info!(
                    provider_id,
                    root_id = %root.id,
                    root_kind = ?root.kind,
                    attempts,
                    matched_name = %metadata.name,
                    "matched root metadata (series)"
                );
                changed = true;
            }
            Ok(RootMatchAttempt::MatchedMovie(metadata)) => {
                upsert_remote_root_metadata_from_movie(pool, &root.id, provider_id, &metadata, now)
                    .await?;
                upsert_root_match_row(
                    pool,
                    RootMatchRowInput {
                        root_id: root.id.clone(),
                        provider_id: provider_id.to_string(),
                        status: NodeMatchStatus::Matched,
                        last_attempted_at: Some(now),
                        last_added_at: Some(root.last_added_at),
                        last_error_message: None,
                        retry_after: None,
                        attempts,
                        created_at: now,
                        updated_at: now,
                    },
                )
                .await?;
                tracing::info!(
                    provider_id,
                    root_id = %root.id,
                    root_kind = ?root.kind,
                    attempts,
                    matched_name = %metadata.name,
                    "matched root metadata (movie)"
                );
                changed = true;
            }
            Ok(RootMatchAttempt::Unmatched) => {
                let retry_after = now + retry_backoff_seconds(attempts);
                upsert_root_match_row(
                    pool,
                    RootMatchRowInput {
                        root_id: root.id.clone(),
                        provider_id: provider_id.to_string(),
                        status: NodeMatchStatus::Unmatched,
                        last_attempted_at: Some(now),
                        last_added_at: Some(root.last_added_at),
                        last_error_message: None,
                        retry_after: Some(retry_after),
                        attempts,
                        created_at: now,
                        updated_at: now,
                    },
                )
                .await?;
                tracing::warn!(
                    provider_id,
                    root_id = %root.id,
                    root_kind = ?root.kind,
                    attempts,
                    retry_after,
                    "root metadata match returned unmatched"
                );
                changed = true;
            }
            Err(error) => {
                let status = existing
                    .as_ref()
                    .map(|row| row.status)
                    .unwrap_or(NodeMatchStatus::Unmatched);
                let retry_after = now + retry_backoff_seconds(attempts);
                upsert_root_match_row(
                    pool,
                    RootMatchRowInput {
                        root_id: root.id.clone(),
                        provider_id: provider_id.to_string(),
                        status,
                        last_attempted_at: Some(now),
                        last_added_at: Some(root.last_added_at),
                        last_error_message: Some(error.to_string()),
                        retry_after: Some(retry_after),
                        attempts,
                        created_at: now,
                        updated_at: now,
                    },
                )
                .await?;
                tracing::warn!(
                    provider_id,
                    root_id = %root.id,
                    root_kind = ?root.kind,
                    attempts,
                    retry_after,
                    error = ?error,
                    "root metadata match attempt failed"
                );
                changed = true;
            }
        }
    }

    Ok(changed)
}

pub async fn process_items(
    pool: &DatabaseConnection,
    provider: &dyn MetadataProvider,
    now: i64,
) -> anyhow::Result<bool> {
    let provider_id = provider.id();
    let all_items = items::Entity::find()
        .order_by_asc(items::Column::RootId)
        .order_by_asc(items::Column::Order)
        .order_by_asc(items::Column::Id)
        .all(pool)
        .await?;

    let mut due_items = Vec::new();
    for item in all_items {
        let existing = item_node_matches::Entity::find()
            .filter(item_node_matches::Column::ItemId.eq(item.id.clone()))
            .filter(item_node_matches::Column::ProviderId.eq(provider_id))
            .one(pool)
            .await?;

        if should_attempt_item_match(&item, existing.as_ref(), now) {
            due_items.push(item);
        }

        if due_items.len() >= MAX_ITEMS_PER_TICK {
            break;
        }
    }

    let mut processed_batches = HashSet::new();
    let mut changed = false;
    for item in due_items {
        let batch_key = item_batch_key(&item);
        if !processed_batches.insert(batch_key.clone()) {
            continue;
        }
        let batch_changed = process_item_batch(pool, provider, &batch_key, now)
            .await
            .with_context(|| format!("failed processing item batch key={batch_key}"))?;
        changed = changed || batch_changed;
    }

    Ok(changed)
}

async fn process_item_batch(
    pool: &DatabaseConnection,
    provider: &dyn MetadataProvider,
    batch_key: &str,
    now: i64,
) -> anyhow::Result<bool> {
    let provider_id = provider.id();
    let batch = load_item_batch(pool, batch_key).await?;
    if batch.is_empty() {
        return Ok(false);
    }

    let root_id = batch
        .first()
        .map(|item| item.root_id.clone())
        .context("item batch missing root_id")?;
    let root = roots::Entity::find_by_id(root_id.clone())
        .one(pool)
        .await?
        .context("root not found for item batch")?;

    tracing::debug!(
        provider_id,
        batch_key,
        root_id = %root.id,
        item_count = batch.len(),
        "processing metadata item batch"
    );

    let item_ids = batch.iter().map(|item| item.id.clone()).collect::<Vec<_>>();
    let existing_rows = item_node_matches::Entity::find()
        .filter(item_node_matches::Column::ProviderId.eq(provider_id))
        .filter(item_node_matches::Column::ItemId.is_in(item_ids.clone()))
        .all(pool)
        .await?
        .into_iter()
        .map(|row| (row.item_id.clone(), row))
        .collect::<HashMap<_, _>>();

    match run_item_batch_match(pool, provider, &root, &batch).await {
        Ok(ItemBatchAttempt::SeriesMatch { seasons, episodes }) => {
            overwrite_remote_season_metadata_for_batch(
                pool,
                &root_id,
                provider_id,
                &batch,
                &seasons,
                now,
            )
            .await?;
            overwrite_remote_item_metadata_for_batch(pool, provider_id, &batch, &episodes, now)
                .await?;

            let matched_ids = episodes
                .iter()
                .map(|episode| episode.item_id.clone())
                .collect::<HashSet<_>>();
            let matched_count = matched_ids.len();
            let unmatched_count = batch.len().saturating_sub(matched_count);
            let rows = batch
                .iter()
                .map(|item| ItemMatchRowInput {
                    root_id: item.root_id.clone(),
                    item_id: item.id.clone(),
                    provider_id: provider_id.to_string(),
                    status: if matched_ids.contains(&item.id) {
                        NodeMatchStatus::Matched
                    } else {
                        NodeMatchStatus::Unmatched
                    },
                    last_attempted_at: Some(now),
                    last_added_at: Some(item.last_added_at),
                    last_error_message: None,
                    retry_after: if matched_ids.contains(&item.id) {
                        None
                    } else {
                        Some(now + retry_backoff_seconds(next_attempts(&existing_rows, &item.id)))
                    },
                    attempts: next_attempts(&existing_rows, &item.id),
                    created_at: now,
                    updated_at: now,
                })
                .collect::<Vec<_>>();
            upsert_item_match_rows(pool, rows).await?;
            tracing::info!(
                provider_id,
                batch_key,
                root_id = %root.id,
                matched_count,
                unmatched_count,
                season_metadata_count = seasons.len(),
                "matched metadata for series item batch"
            );
        }
        Ok(ItemBatchAttempt::MovieMatch { metadata }) => {
            overwrite_remote_movie_metadata_for_batch(pool, provider_id, &batch, &metadata, now)
                .await?;
            let rows = batch
                .iter()
                .map(|item| ItemMatchRowInput {
                    root_id: item.root_id.clone(),
                    item_id: item.id.clone(),
                    provider_id: provider_id.to_string(),
                    status: NodeMatchStatus::Matched,
                    last_attempted_at: Some(now),
                    last_added_at: Some(item.last_added_at),
                    last_error_message: None,
                    retry_after: None,
                    attempts: next_attempts(&existing_rows, &item.id),
                    created_at: now,
                    updated_at: now,
                })
                .collect::<Vec<_>>();
            upsert_item_match_rows(pool, rows).await?;
            tracing::info!(
                provider_id,
                batch_key,
                root_id = %root.id,
                item_count = batch.len(),
                matched_name = %metadata.name,
                "matched metadata for movie item batch"
            );
        }
        Ok(ItemBatchAttempt::Unmatched) => {
            clear_remote_item_metadata_for_batch(pool, &item_ids).await?;
            let rows = batch
                .iter()
                .map(|item| {
                    let attempts = next_attempts(&existing_rows, &item.id);
                    ItemMatchRowInput {
                        root_id: item.root_id.clone(),
                        item_id: item.id.clone(),
                        provider_id: provider_id.to_string(),
                        status: NodeMatchStatus::Unmatched,
                        last_attempted_at: Some(now),
                        last_added_at: Some(item.last_added_at),
                        last_error_message: None,
                        retry_after: Some(now + retry_backoff_seconds(attempts)),
                        attempts,
                        created_at: now,
                        updated_at: now,
                    }
                })
                .collect::<Vec<_>>();
            upsert_item_match_rows(pool, rows).await?;
            tracing::warn!(
                provider_id,
                batch_key,
                root_id = %root.id,
                item_count = batch.len(),
                "item metadata batch returned unmatched"
            );
        }
        Err(error) => {
            let rows = batch
                .iter()
                .map(|item| {
                    let attempts = next_attempts(&existing_rows, &item.id);
                    let status = existing_rows
                        .get(&item.id)
                        .map(|row| row.status)
                        .unwrap_or(NodeMatchStatus::Unmatched);
                    ItemMatchRowInput {
                        root_id: item.root_id.clone(),
                        item_id: item.id.clone(),
                        provider_id: provider_id.to_string(),
                        status,
                        last_attempted_at: Some(now),
                        last_added_at: Some(item.last_added_at),
                        last_error_message: Some(error.to_string()),
                        retry_after: Some(now + retry_backoff_seconds(attempts)),
                        attempts,
                        created_at: now,
                        updated_at: now,
                    }
                })
                .collect::<Vec<_>>();
            upsert_item_match_rows(pool, rows).await?;
            tracing::warn!(
                provider_id,
                batch_key,
                root_id = %root.id,
                item_count = batch.len(),
                error = ?error,
                "item metadata batch failed"
            );
        }
    }

    Ok(true)
}

async fn run_root_match_attempt(
    pool: &DatabaseConnection,
    provider: &dyn MetadataProvider,
    root: &roots::Model,
) -> anyhow::Result<RootMatchAttempt> {
    let hint = load_root_match_hint(pool, root).await?;
    match root.kind {
        roots::RootKind::Series => {
            let candidates = provider
                .match_series_root(SeriesRootMatchRequest { hint })
                .await?;
            let Some(candidate) = candidates.first() else {
                return Ok(RootMatchAttempt::Unmatched);
            };
            let metadata = provider.lookup_series_metadata(&candidate.value).await?;
            Ok(RootMatchAttempt::MatchedSeries(metadata))
        }
        roots::RootKind::Movie => {
            let candidates = provider
                .match_movie_root(MovieRootMatchRequest { hint })
                .await?;
            let Some(candidate) = candidates.first() else {
                return Ok(RootMatchAttempt::Unmatched);
            };
            let metadata = provider.lookup_movie_metadata(&candidate.value).await?;
            Ok(RootMatchAttempt::MatchedMovie(metadata))
        }
    }
}

async fn run_item_batch_match(
    pool: &DatabaseConnection,
    provider: &dyn MetadataProvider,
    root: &roots::Model,
    batch: &[items::Model],
) -> anyhow::Result<ItemBatchAttempt> {
    let hint = load_root_match_hint(pool, root).await?;
    match root.kind {
        roots::RootKind::Movie => {
            let candidates = provider
                .match_movie_root(MovieRootMatchRequest { hint })
                .await?;
            let Some(candidate) = candidates.first() else {
                return Ok(ItemBatchAttempt::Unmatched);
            };
            let metadata = provider.lookup_movie_metadata(&candidate.value).await?;
            Ok(ItemBatchAttempt::MovieMatch { metadata })
        }
        roots::RootKind::Series => {
            let candidates = provider
                .match_series_root(SeriesRootMatchRequest { hint })
                .await?;
            let Some(candidate) = candidates.first() else {
                return Ok(ItemBatchAttempt::Unmatched);
            };

            let season_numbers = load_season_numbers(pool, batch).await?;
            let req = SeriesItemsRequest {
                root_id: root.id.clone(),
                candidate: candidate.value.clone(),
                items: batch
                    .iter()
                    .map(|item| SeriesItem {
                        item_id: item.id.clone(),
                        season_number: item
                            .season_id
                            .as_ref()
                            .and_then(|season_id| season_numbers.get(season_id).copied())
                            .and_then(|value| i32::try_from(value).ok()),
                        episode_number: item
                            .episode_number
                            .and_then(|value| i32::try_from(value).ok()),
                        name: item.name.clone(),
                    })
                    .collect::<Vec<_>>(),
            };
            let results = provider.lookup_series_items(req).await?;
            Ok(ItemBatchAttempt::SeriesMatch {
                seasons: results.seasons,
                episodes: results.episodes,
            })
        }
    }
}

async fn load_root_match_hint(
    pool: &DatabaseConnection,
    root: &roots::Model,
) -> anyhow::Result<RootMatchHint> {
    let file_paths = item_files::Entity::find()
        .join(JoinType::InnerJoin, item_files::Relation::Items.def())
        .join(JoinType::InnerJoin, item_files::Relation::Files.def())
        .filter(items::Column::RootId.eq(root.id.clone()))
        .select_only()
        .column(files::Column::RelativePath)
        .distinct()
        .limit(MAX_HINT_FILES)
        .into_tuple::<String>()
        .all(pool)
        .await?;

    if file_paths.is_empty() {
        return Ok(RootMatchHint {
            title: root.name.clone(),
            start_year: None,
            end_year: None,
            imdb_id: None,
            tmdb_id: None,
        });
    }

    let parsed_files = parse_files(file_paths).await;
    let mut years = HashMap::<i32, usize>::new();
    for parsed in &parsed_files {
        if let Some(year) = parsed.start_year.and_then(|year| i32::try_from(year).ok()) {
            years
                .entry(year)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
    }

    Ok(RootMatchHint {
        title: root.name.clone(),
        start_year: years
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(year, _)| year),
        end_year: parsed_files
            .iter()
            .filter_map(|parsed| parsed.end_year.and_then(|value| i32::try_from(value).ok()))
            .max(),
        imdb_id: parsed_files
            .iter()
            .find_map(|parsed| parsed.imdb_id.clone()),
        tmdb_id: parsed_files.iter().find_map(|parsed| parsed.tmdb_id),
    })
}

async fn load_item_batch(
    pool: &DatabaseConnection,
    batch_key: &str,
) -> anyhow::Result<Vec<items::Model>> {
    if let Some(season_id) = batch_key.strip_prefix("season:") {
        return items::Entity::find()
            .filter(items::Column::SeasonId.eq(season_id.to_string()))
            .order_by_asc(items::Column::Order)
            .order_by_asc(items::Column::Id)
            .all(pool)
            .await
            .map_err(Into::into);
    }

    let root_id = batch_key
        .strip_prefix("root:")
        .context("invalid batch key format")?;
    items::Entity::find()
        .filter(items::Column::RootId.eq(root_id.to_string()))
        .order_by_asc(items::Column::Order)
        .order_by_asc(items::Column::Id)
        .all(pool)
        .await
        .map_err(Into::into)
}

async fn load_season_numbers(
    pool: &DatabaseConnection,
    batch: &[items::Model],
) -> anyhow::Result<HashMap<String, i64>> {
    let season_ids = batch
        .iter()
        .filter_map(|item| item.season_id.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if season_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = seasons::Entity::find()
        .filter(seasons::Column::Id.is_in(season_ids))
        .all(pool)
        .await?;
    Ok(rows
        .into_iter()
        .map(|season| (season.id, season.season_number))
        .collect::<HashMap<_, _>>())
}

fn should_attempt_root_match(
    root: &roots::Model,
    existing: Option<&root_node_matches::Model>,
    now: i64,
) -> bool {
    let Some(existing) = existing else {
        return true;
    };
    if existing
        .retry_after
        .is_some_and(|retry_after| retry_after > now)
    {
        return false;
    }
    !(existing.status == NodeMatchStatus::Matched
        && existing.last_error_message.is_none()
        && existing.last_added_at.unwrap_or(0) >= root.last_added_at)
}

fn should_attempt_item_match(
    item: &items::Model,
    existing: Option<&item_node_matches::Model>,
    now: i64,
) -> bool {
    let Some(existing) = existing else {
        return true;
    };
    if existing
        .retry_after
        .is_some_and(|retry_after| retry_after > now)
    {
        return false;
    }
    !(existing.status == NodeMatchStatus::Matched
        && existing.last_error_message.is_none()
        && existing.last_added_at.unwrap_or(0) >= item.last_added_at)
}

fn item_batch_key(item: &items::Model) -> String {
    if let Some(season_id) = &item.season_id {
        format!("season:{season_id}")
    } else {
        format!("root:{}", item.root_id)
    }
}

enum RootMatchAttempt {
    MatchedSeries(SeriesMetadata),
    MatchedMovie(MovieMetadata),
    Unmatched,
}

enum ItemBatchAttempt {
    SeriesMatch {
        seasons: Vec<SeasonMetadata>,
        episodes: Vec<EpisodeMetadata>,
    },
    MovieMatch {
        metadata: MovieMetadata,
    },
    Unmatched,
}
