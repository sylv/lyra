use crate::entities::{jobs::JobKind, nodes, nodes::NodeKind};
use crate::jobs::delete_job_row;
use crate::metadata::remote::{MatchedRoot, lookup_series_items, match_root};
use crate::metadata::store::{
    clear_remote_node_metadata_for_root, clear_remote_node_metadata_for_root_except,
    upsert_remote_episode_metadata_for_batch, upsert_remote_node_metadata_from_movie,
    upsert_remote_node_metadata_from_series, upsert_remote_season_metadata_for_batch,
};
use lyra_metadata::MetadataProvider;
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
};
use std::sync::Arc;

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

                clear_remote_node_metadata_for_root_except(pool, &root.id, &matched_node_ids)
                    .await?;
                return Ok(());
            }
        }
    }

    clear_remote_node_metadata_for_root(pool, &root.id).await?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{libraries, metadata_source::MetadataSource, node_metadata};
    use async_trait::async_trait;
    use lyra_metadata::{
        EpisodeMetadata, ImageSet, MetadataProvider, MovieCandidate, MovieMetadata,
        MovieRootMatchRequest, Scored, SeasonMetadata, SeriesCandidate, SeriesItemsRequest,
        SeriesItemsResult, SeriesMetadata, SeriesRootMatchRequest,
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
            last_scanned_at: Set(None),
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
                released_at: None,
                ended_at: None,
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
                    released_at: None,
                    ended_at: None,
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
                        released_at: None,
                        images: ImageSet::default(),
                    })
                    .collect(),
            })
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
            series_items_calls: AtomicUsize::new(0),
        });
        let second = Arc::new(FakeProvider {
            id: "second",
            match_result: MatchResult::Series,
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
}
