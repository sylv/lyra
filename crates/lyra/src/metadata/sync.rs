use crate::entities::{
    jobs::JobKind, metadata_source::MetadataSource, node_metadata, nodes, nodes::NodeKind,
};
use crate::jobs::delete_job_row;
use crate::metadata::remote::{MatchedRoot, lookup_series_items, match_root};
use crate::metadata::store::{
    clear_remote_node_metadata_for_root, clear_remote_node_metadata_for_root_except,
    clear_root_cast, replace_root_cast, upsert_remote_episode_metadata_for_batch,
    upsert_remote_node_metadata_from_movie, upsert_remote_node_metadata_from_series,
    upsert_remote_season_metadata_for_batch,
};
use lyra_metadata::MetadataProvider;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection,
    EntityTrait, QueryFilter, QueryOrder,
};
use std::{collections::HashMap, sync::Arc};

pub async fn mark_root_dirty(pool: &impl ConnectionTrait, root_id: &str) -> anyhow::Result<()> {
    delete_job_row(pool, JobKind::NodeSyncMetadataRoot, root_id).await
}

// Metadata sync is root-scoped so provider choice, remote writes, and stale cleanup all happen
// in one place for a root and its descendants.
pub async fn sync_root(
    pool: &DatabaseConnection,
    providers: &[Arc<dyn MetadataProvider>],
    root: &nodes::Model,
) -> anyhow::Result<()> {
    let now = chrono::Utc::now().timestamp();
    let season_nodes = load_root_nodes(pool, &root.id, NodeKind::Season).await?;
    let episode_nodes = load_root_nodes(pool, &root.id, NodeKind::Episode).await?;
    let mut errors = Vec::new();

    for provider in providers {
        let matched = match match_root(pool, provider.as_ref(), root).await {
            Ok(Some(matched)) => matched,
            Ok(None) => continue,
            Err(error) => {
                errors.push(format!(
                    "provider {} failed to match: {error:#}",
                    provider.id()
                ));
                continue;
            }
        };

        match matched {
            MatchedRoot::Movie { metadata } => {
                upsert_remote_node_metadata_from_movie(
                    pool,
                    &root.id,
                    provider.id(),
                    &metadata,
                    now,
                )
                .await?;
                let people = provider
                    .lookup_people_metadata(
                        &metadata
                            .cast
                            .iter()
                            .map(|credit| credit.provider_person_id.clone())
                            .collect::<Vec<_>>(),
                    )
                    .await?;
                replace_root_cast(pool, &root.id, provider.id(), &metadata.cast, &people, now)
                    .await?;
                clear_remote_node_metadata_for_root_except(pool, &root.id, &[root.id.clone()])
                    .await?;
                return Ok(());
            }
            MatchedRoot::Series {
                candidate,
                metadata,
            } => {
                let items =
                    lookup_series_items(provider.as_ref(), &root.id, &candidate, &episode_nodes)
                        .await?;
                upsert_remote_node_metadata_from_series(
                    pool,
                    &root.id,
                    provider.id(),
                    &metadata,
                    now,
                )
                .await?;
                let people = provider
                    .lookup_people_metadata(
                        &metadata
                            .cast
                            .iter()
                            .map(|credit| credit.provider_person_id.clone())
                            .collect::<Vec<_>>(),
                    )
                    .await?;
                replace_root_cast(pool, &root.id, provider.id(), &metadata.cast, &people, now)
                    .await?;

                let mut matched_node_ids = vec![root.id.clone()];
                matched_node_ids.extend(
                    upsert_remote_season_metadata_for_batch(
                        pool,
                        provider.id(),
                        &season_nodes,
                        &items.seasons,
                        now,
                    )
                    .await?,
                );
                matched_node_ids.extend(
                    upsert_remote_episode_metadata_for_batch(
                        pool,
                        provider.id(),
                        &episode_nodes,
                        &items.episodes,
                        now,
                    )
                    .await?,
                );
                reconcile_series_air_dates(
                    pool,
                    provider.id(),
                    root,
                    &season_nodes,
                    &episode_nodes,
                    now,
                )
                .await?;

                clear_remote_node_metadata_for_root_except(pool, &root.id, &matched_node_ids)
                    .await?;
                return Ok(());
            }
        }
    }

    clear_remote_node_metadata_for_root(pool, &root.id).await?;
    clear_root_cast(pool, &root.id).await?;

    if errors.is_empty() {
        anyhow::bail!("no metadata provider matched root {}", root.id);
    }

    anyhow::bail!(
        "metadata sync failed for root {}: {}",
        root.id,
        errors.join("; ")
    );
}

async fn load_root_nodes(
    pool: &DatabaseConnection,
    root_id: &str,
    kind: NodeKind,
) -> anyhow::Result<Vec<nodes::Model>> {
    Ok(nodes::Entity::find()
        .filter(nodes::Column::RootId.eq(root_id))
        .filter(nodes::Column::Kind.eq(kind))
        .order_by_asc(nodes::Column::Order)
        .order_by_asc(nodes::Column::Id)
        .all(pool)
        .await?)
}

// TMDb's root and season air dates are useful fallbacks, but once we have child metadata we want
// parent bounds to reflect the actual matched episodes we expose in the UI.
async fn reconcile_series_air_dates(
    pool: &DatabaseConnection,
    provider_id: &str,
    root: &nodes::Model,
    season_nodes: &[nodes::Model],
    episode_nodes: &[nodes::Model],
    now: i64,
) -> anyhow::Result<()> {
    let mut remote_metadata = load_remote_metadata_map(
        pool,
        provider_id,
        std::iter::once(root.id.as_str())
            .chain(season_nodes.iter().map(|node| node.id.as_str()))
            .chain(episode_nodes.iter().map(|node| node.id.as_str()))
            .collect(),
    )
    .await?;

    let mut updated_season_bounds = HashMap::new();
    for season in season_nodes {
        let child_bounds = aggregate_air_dates(
            episode_nodes
                .iter()
                .filter(|node| node.parent_id.as_deref() == Some(season.id.as_str()))
                .filter_map(|node| {
                    remote_metadata
                        .get(&node.id)
                        .map(|metadata| (metadata.first_aired, metadata.last_aired))
                }),
        );
        let fallback = remote_metadata
            .get(&season.id)
            .map(|metadata| (metadata.first_aired, metadata.last_aired))
            .unwrap_or((None, None));
        let season_bounds = prefer_primary_air_dates(child_bounds, fallback);
        updated_season_bounds.insert(season.id.clone(), season_bounds);
        update_remote_air_dates(pool, &mut remote_metadata, &season.id, season_bounds, now).await?;
    }

    let episode_bounds = aggregate_air_dates(episode_nodes.iter().filter_map(|node| {
        remote_metadata
            .get(&node.id)
            .map(|metadata| (metadata.first_aired, metadata.last_aired))
    }));
    let season_bounds = aggregate_air_dates(updated_season_bounds.values().copied());
    let root_fallback = remote_metadata
        .get(&root.id)
        .map(|metadata| (metadata.first_aired, metadata.last_aired))
        .unwrap_or((None, None));
    let root_bounds = prefer_primary_air_dates(
        merge_air_dates(episode_bounds, season_bounds),
        root_fallback,
    );
    update_remote_air_dates(pool, &mut remote_metadata, &root.id, root_bounds, now).await?;

    Ok(())
}

async fn load_remote_metadata_map(
    pool: &DatabaseConnection,
    provider_id: &str,
    node_ids: Vec<&str>,
) -> anyhow::Result<HashMap<String, node_metadata::Model>> {
    if node_ids.is_empty() {
        return Ok(HashMap::new());
    }

    Ok(node_metadata::Entity::find()
        .filter(node_metadata::Column::Source.eq(MetadataSource::Remote))
        .filter(node_metadata::Column::ProviderId.eq(provider_id))
        .filter(node_metadata::Column::NodeId.is_in(node_ids))
        .all(pool)
        .await?
        .into_iter()
        .map(|metadata| (metadata.node_id.clone(), metadata))
        .collect())
}

fn aggregate_air_dates(
    rows: impl IntoIterator<Item = (Option<i64>, Option<i64>)>,
) -> (Option<i64>, Option<i64>) {
    rows.into_iter().fold((None, None), merge_air_dates)
}

fn merge_air_dates(
    left: (Option<i64>, Option<i64>),
    right: (Option<i64>, Option<i64>),
) -> (Option<i64>, Option<i64>) {
    let first = match (left.0.or(left.1), right.0.or(right.1)) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    };
    let last = match (left.1.or(left.0), right.1.or(right.0)) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    };

    (first, last)
}

fn prefer_primary_air_dates(
    primary: (Option<i64>, Option<i64>),
    fallback: (Option<i64>, Option<i64>),
) -> (Option<i64>, Option<i64>) {
    (primary.0.or(fallback.0), primary.1.or(fallback.1))
}

async fn update_remote_air_dates(
    pool: &DatabaseConnection,
    metadata_by_node_id: &mut HashMap<String, node_metadata::Model>,
    node_id: &str,
    air_dates: (Option<i64>, Option<i64>),
    now: i64,
) -> anyhow::Result<()> {
    let Some(existing) = metadata_by_node_id.get(node_id).cloned() else {
        return Ok(());
    };
    if (existing.first_aired, existing.last_aired) == air_dates {
        return Ok(());
    }

    let mut active: node_metadata::ActiveModel = existing.into();
    active.first_aired = Set(air_dates.0);
    active.last_aired = Set(air_dates.1);
    active.updated_at = Set(now);
    let updated = active.update(pool).await?;
    metadata_by_node_id.insert(updated.node_id.clone(), updated);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{
        libraries, metadata_source::MetadataSource, node_metadata, people, root_node_cast,
    };
    use async_trait::async_trait;
    use lyra_metadata::{
        CastCredit, EpisodeMetadata, ImageSet, MetadataProvider, MovieCandidate, MovieMetadata,
        MovieRootMatchRequest, PersonMetadata, Scored, SeasonMetadata, SeriesCandidate,
        SeriesItemsRequest, SeriesItemsResult, SeriesMetadata, SeriesRootMatchRequest,
    };
    use sea_orm::{ActiveValue::Set, Database};
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    async fn setup_test_db() -> anyhow::Result<DatabaseConnection> {
        let pool = Database::connect("sqlite::memory:").await?;
        sqlx::migrate!("../../migrations")
            .run(pool.get_sqlite_connection_pool())
            .await?;
        Ok(pool)
    }

    async fn insert_library(pool: &DatabaseConnection) -> anyhow::Result<()> {
        libraries::Entity::insert(libraries::ActiveModel {
            id: Set("lib".to_owned()),
            path: Set("/library".to_owned()),
            name: Set("Library".to_owned()),
            pinned: Set(false),
            last_scanned_at: Set(None),
            unavailable_at: Set(None),
            created_at: Set(0),
        })
        .exec(pool)
        .await?;
        Ok(())
    }

    async fn insert_node(
        pool: &DatabaseConnection,
        id: &str,
        root_id: &str,
        parent_id: Option<&str>,
        kind: NodeKind,
        name: &str,
        season_number: Option<i64>,
        episode_number: Option<i64>,
        order: i64,
    ) -> anyhow::Result<nodes::Model> {
        nodes::Entity::insert(nodes::ActiveModel {
            id: Set(id.to_owned()),
            library_id: Set("lib".to_owned()),
            root_id: Set(root_id.to_owned()),
            parent_id: Set(parent_id.map(str::to_owned)),
            kind: Set(kind),
            name: Set(name.to_owned()),
            order: Set(order),
            season_number: Set(season_number),
            episode_number: Set(episode_number),
            last_added_at: Set(0),
            last_fingerprint_version: Set(None),
            unavailable_at: Set(None),
            created_at: Set(0),
            updated_at: Set(0),
        })
        .exec(pool)
        .await?;

        Ok(nodes::Entity::find_by_id(id.to_owned())
            .one(pool)
            .await?
            .unwrap())
    }

    async fn insert_local_metadata(
        pool: &DatabaseConnection,
        node_id: &str,
        name: &str,
    ) -> anyhow::Result<()> {
        node_metadata::Entity::insert(node_metadata::ActiveModel {
            id: Set(format!("local-{node_id}")),
            node_id: Set(node_id.to_owned()),
            source: Set(MetadataSource::Local),
            provider_id: Set("local".to_owned()),
            name: Set(name.to_owned()),
            created_at: Set(0),
            updated_at: Set(0),
            ..Default::default()
        })
        .exec(pool)
        .await?;
        Ok(())
    }

    struct FakeProvider {
        id: &'static str,
        match_result: MatchResult,
        cast: Vec<CastCredit>,
        people_metadata: Vec<PersonMetadata>,
        series_items_calls: AtomicUsize,
    }

    enum MatchResult {
        NoMatch,
        Series,
    }

    #[async_trait]
    impl MetadataProvider for FakeProvider {
        fn id(&self) -> &'static str {
            self.id
        }

        async fn match_series_root(
            &self,
            _req: SeriesRootMatchRequest,
        ) -> anyhow::Result<Vec<Scored<SeriesCandidate>>> {
            match self.match_result {
                MatchResult::NoMatch => Ok(Vec::new()),
                MatchResult::Series => Ok(vec![Scored {
                    value: SeriesCandidate {
                        tmdb_id: 1,
                        name: "Matched Show".to_owned(),
                        first_air_year: None,
                    },
                    score: 1.0,
                }]),
            }
        }

        async fn lookup_series_metadata(
            &self,
            _candidate: &SeriesCandidate,
        ) -> anyhow::Result<SeriesMetadata> {
            Ok(SeriesMetadata {
                imdb_id: Some("tt1234567".to_owned()),
                tmdb_id: Some(1),
                name: "Matched Show".to_owned(),
                description: Some(format!("series from {}", self.id)),
                score_display: None,
                score_normalized: None,
                first_aired: None,
                last_aired: None,
                status: None,
                tagline: None,
                next_aired: None,
                genres: Vec::new(),
                content_ratings: Vec::new(),
                cast: self.cast.clone(),
                recommendations: Vec::new(),
                images: ImageSet::default(),
            })
        }

        async fn lookup_series_items(
            &self,
            req: SeriesItemsRequest,
        ) -> anyhow::Result<SeriesItemsResult> {
            self.series_items_calls.fetch_add(1, Ordering::Relaxed);

            Ok(SeriesItemsResult {
                seasons: vec![SeasonMetadata {
                    root_id: req.root_id.clone(),
                    season_number: 1,
                    name: "Season 1".to_owned(),
                    description: Some(format!("season from {}", self.id)),
                    score_display: None,
                    score_normalized: None,
                    first_aired: None,
                    last_aired: None,
                    status: None,
                    tagline: None,
                    next_aired: None,
                    genres: Vec::new(),
                    content_ratings: Vec::new(),
                    recommendations: Vec::new(),
                    images: ImageSet::default(),
                }],
                episodes: req
                    .items
                    .into_iter()
                    .filter(|item| item.item_id == "episode-1")
                    .map(|item| EpisodeMetadata {
                        item_id: item.item_id,
                        name: "Episode 1".to_owned(),
                        description: Some(format!("episode from {}", self.id)),
                        score_display: None,
                        score_normalized: None,
                        first_aired: None,
                        last_aired: None,
                        status: None,
                        tagline: None,
                        next_aired: None,
                        genres: Vec::new(),
                        content_ratings: Vec::new(),
                        recommendations: Vec::new(),
                        images: ImageSet::default(),
                    })
                    .collect(),
            })
        }

        async fn lookup_people_metadata(
            &self,
            provider_person_ids: &[String],
        ) -> anyhow::Result<Vec<PersonMetadata>> {
            Ok(self
                .people_metadata
                .iter()
                .filter(|person| {
                    provider_person_ids
                        .iter()
                        .any(|id| id == &person.provider_person_id)
                })
                .cloned()
                .collect())
        }

        async fn match_movie_root(
            &self,
            _req: MovieRootMatchRequest,
        ) -> anyhow::Result<Vec<Scored<MovieCandidate>>> {
            Ok(Vec::new())
        }

        async fn lookup_movie_metadata(
            &self,
            _candidate: &MovieCandidate,
        ) -> anyhow::Result<MovieMetadata> {
            anyhow::bail!("movie lookup not used in this test")
        }
    }

    #[tokio::test]
    async fn sync_root_uses_first_matching_provider_and_clears_stale_remote_rows()
    -> anyhow::Result<()> {
        let pool = setup_test_db().await?;
        insert_library(&pool).await?;
        let root = insert_node(
            &pool,
            "root",
            "root",
            None,
            NodeKind::Series,
            "Show",
            None,
            None,
            0,
        )
        .await?;
        insert_node(
            &pool,
            "season-1",
            "root",
            Some("root"),
            NodeKind::Season,
            "Season 1",
            Some(1),
            None,
            1,
        )
        .await?;
        insert_node(
            &pool,
            "episode-1",
            "root",
            Some("season-1"),
            NodeKind::Episode,
            "Episode 1",
            Some(1),
            Some(1),
            2,
        )
        .await?;
        insert_node(
            &pool,
            "episode-2",
            "root",
            Some("season-1"),
            NodeKind::Episode,
            "Episode 2",
            Some(1),
            Some(2),
            3,
        )
        .await?;

        insert_local_metadata(&pool, "root", "Show").await?;
        insert_local_metadata(&pool, "season-1", "Season 1").await?;
        insert_local_metadata(&pool, "episode-1", "Episode 1").await?;
        insert_local_metadata(&pool, "episode-2", "Episode 2").await?;

        node_metadata::Entity::insert(node_metadata::ActiveModel {
            id: Set("stale-episode-2".to_owned()),
            node_id: Set("episode-2".to_owned()),
            source: Set(MetadataSource::Remote),
            provider_id: Set("stale".to_owned()),
            name: Set("Old Episode 2".to_owned()),
            created_at: Set(0),
            updated_at: Set(0),
            ..Default::default()
        })
        .exec(&pool)
        .await?;

        let first = Arc::new(FakeProvider {
            id: "first",
            match_result: MatchResult::NoMatch,
            cast: Vec::new(),
            people_metadata: Vec::new(),
            series_items_calls: AtomicUsize::new(0),
        });
        let second = Arc::new(FakeProvider {
            id: "second",
            match_result: MatchResult::Series,
            cast: Vec::new(),
            people_metadata: Vec::new(),
            series_items_calls: AtomicUsize::new(0),
        });

        sync_root(&pool, &[first.clone(), second.clone()], &root).await?;

        assert_eq!(first.series_items_calls.load(Ordering::Relaxed), 0);
        assert_eq!(second.series_items_calls.load(Ordering::Relaxed), 1);

        let remote_rows = node_metadata::Entity::find()
            .filter(node_metadata::Column::Source.eq(MetadataSource::Remote))
            .all(&pool)
            .await?;

        assert_eq!(remote_rows.len(), 3);
        assert!(remote_rows.iter().all(|row| row.provider_id == "second"));
        assert!(remote_rows.iter().any(|row| row.node_id == "root"));
        assert!(remote_rows.iter().any(|row| row.node_id == "season-1"));
        assert!(remote_rows.iter().any(|row| row.node_id == "episode-1"));
        assert!(!remote_rows.iter().any(|row| row.node_id == "episode-2"));

        Ok(())
    }

    #[tokio::test]
    async fn sync_root_clears_stale_remote_rows_when_no_provider_matches() -> anyhow::Result<()> {
        let pool = setup_test_db().await?;
        insert_library(&pool).await?;
        let root = insert_node(
            &pool,
            "root",
            "root",
            None,
            NodeKind::Series,
            "Show",
            None,
            None,
            0,
        )
        .await?;
        insert_local_metadata(&pool, "root", "Show").await?;

        node_metadata::Entity::insert(node_metadata::ActiveModel {
            id: Set("stale-root".to_owned()),
            node_id: Set("root".to_owned()),
            source: Set(MetadataSource::Remote),
            provider_id: Set("stale".to_owned()),
            name: Set("Old Root".to_owned()),
            created_at: Set(0),
            updated_at: Set(0),
            ..Default::default()
        })
        .exec(&pool)
        .await?;

        let provider = Arc::new(FakeProvider {
            id: "nomatch",
            match_result: MatchResult::NoMatch,
            cast: Vec::new(),
            people_metadata: Vec::new(),
            series_items_calls: AtomicUsize::new(0),
        });

        let result = sync_root(&pool, &[provider], &root).await;
        assert!(result.is_err());

        let remote_rows = node_metadata::Entity::find()
            .filter(node_metadata::Column::Source.eq(MetadataSource::Remote))
            .all(&pool)
            .await?;
        assert!(remote_rows.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn sync_root_reuses_people_across_roots_and_replaces_root_cast() -> anyhow::Result<()> {
        let pool = setup_test_db().await?;
        insert_library(&pool).await?;

        let root_one = insert_node(
            &pool,
            "root-1",
            "root-1",
            None,
            NodeKind::Series,
            "Show One",
            None,
            None,
            0,
        )
        .await?;
        let root_two = insert_node(
            &pool,
            "root-2",
            "root-2",
            None,
            NodeKind::Series,
            "Show Two",
            None,
            None,
            0,
        )
        .await?;
        insert_local_metadata(&pool, "root-1", "Show One").await?;
        insert_local_metadata(&pool, "root-2", "Show Two").await?;

        let provider: Arc<dyn MetadataProvider> = Arc::new(FakeProvider {
            id: "tmdb",
            match_result: MatchResult::Series,
            cast: vec![CastCredit {
                provider_person_id: "7".to_owned(),
                name: "Shared Actor".to_owned(),
                character_name: Some("Lead".to_owned()),
                department: None,
            }],
            people_metadata: vec![PersonMetadata {
                provider_person_id: "7".to_owned(),
                name: "Shared Actor".to_owned(),
                birthday: Some("1970-01-01".to_owned()),
                description: Some("Biography".to_owned()),
                profile_image_url: Some("https://image.tmdb.org/t/p/w342/profile.jpg".to_owned()),
            }],
            series_items_calls: AtomicUsize::new(0),
        });

        sync_root(&pool, std::slice::from_ref(&provider), &root_one).await?;
        sync_root(&pool, std::slice::from_ref(&provider), &root_two).await?;

        let people_rows = people::Entity::find().all(&pool).await?;
        assert_eq!(people_rows.len(), 1);
        assert_eq!(people_rows[0].provider_id, "tmdb");
        assert_eq!(people_rows[0].provider_person_id, "7");
        assert!(people_rows[0].profile_asset_id.is_some());

        let cast_rows = root_node_cast::Entity::find()
            .order_by_asc(root_node_cast::Column::RootNodeId)
            .all(&pool)
            .await?;
        assert_eq!(cast_rows.len(), 2);
        assert_eq!(cast_rows[0].person_id, people_rows[0].id);
        assert_eq!(cast_rows[1].person_id, people_rows[0].id);

        Ok(())
    }

    #[tokio::test]
    async fn reconcile_series_air_dates_prefers_episode_bounds_for_parents() -> anyhow::Result<()> {
        let pool = setup_test_db().await?;
        insert_library(&pool).await?;
        let root = insert_node(
            &pool,
            "root",
            "root",
            None,
            NodeKind::Series,
            "Show",
            None,
            None,
            0,
        )
        .await?;
        let season = insert_node(
            &pool,
            "season-1",
            "root",
            Some("root"),
            NodeKind::Season,
            "Season 1",
            Some(1),
            None,
            1,
        )
        .await?;
        let episode_one = insert_node(
            &pool,
            "episode-1",
            "root",
            Some("season-1"),
            NodeKind::Episode,
            "Episode 1",
            Some(1),
            Some(1),
            2,
        )
        .await?;
        let episode_two = insert_node(
            &pool,
            "episode-2",
            "root",
            Some("season-1"),
            NodeKind::Episode,
            "Episode 2",
            Some(1),
            Some(2),
            3,
        )
        .await?;

        node_metadata::Entity::insert_many([
            node_metadata::ActiveModel {
                id: Set("remote-root".to_owned()),
                node_id: Set("root".to_owned()),
                source: Set(MetadataSource::Remote),
                provider_id: Set("tmdb".to_owned()),
                name: Set("Show".to_owned()),
                first_aired: Set(Some(10)),
                last_aired: Set(Some(20)),
                created_at: Set(0),
                updated_at: Set(0),
                ..Default::default()
            },
            node_metadata::ActiveModel {
                id: Set("remote-season-1".to_owned()),
                node_id: Set("season-1".to_owned()),
                source: Set(MetadataSource::Remote),
                provider_id: Set("tmdb".to_owned()),
                name: Set("Season 1".to_owned()),
                first_aired: Set(Some(11)),
                last_aired: Set(Some(19)),
                created_at: Set(0),
                updated_at: Set(0),
                ..Default::default()
            },
            node_metadata::ActiveModel {
                id: Set("remote-episode-1".to_owned()),
                node_id: Set("episode-1".to_owned()),
                source: Set(MetadataSource::Remote),
                provider_id: Set("tmdb".to_owned()),
                name: Set("Episode 1".to_owned()),
                first_aired: Set(Some(12)),
                last_aired: Set(Some(12)),
                created_at: Set(0),
                updated_at: Set(0),
                ..Default::default()
            },
            node_metadata::ActiveModel {
                id: Set("remote-episode-2".to_owned()),
                node_id: Set("episode-2".to_owned()),
                source: Set(MetadataSource::Remote),
                provider_id: Set("tmdb".to_owned()),
                name: Set("Episode 2".to_owned()),
                first_aired: Set(Some(15)),
                last_aired: Set(Some(18)),
                created_at: Set(0),
                updated_at: Set(0),
                ..Default::default()
            },
        ])
        .exec(&pool)
        .await?;

        reconcile_series_air_dates(
            &pool,
            "tmdb",
            &root,
            &[season],
            &[episode_one, episode_two],
            42,
        )
        .await?;

        let root_remote = node_metadata::Entity::find_by_id("remote-root")
            .one(&pool)
            .await?
            .unwrap();
        let season_remote = node_metadata::Entity::find_by_id("remote-season-1")
            .one(&pool)
            .await?
            .unwrap();

        assert_eq!(root_remote.first_aired, Some(12));
        assert_eq!(root_remote.last_aired, Some(18));
        assert_eq!(season_remote.first_aired, Some(12));
        assert_eq!(season_remote.last_aired, Some(18));

        Ok(())
    }
}
