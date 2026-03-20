use crate::entities::{
    files, metadata_source::MetadataSource, node_closure, node_files, node_metadata, nodes,
};
use crate::ids;
use crate::scanner::derive_nodes::{
    RootMaterializationPlan, WantedNode, build_closure_rows, build_root_materialization_plans,
    sort_nodes_topologically, verify_root_nodes,
};
use lyra_parser::{ParsedFile, parse_files};
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, JoinType, QueryFilter,
    QuerySelect, RelationTrait, TransactionTrait,
};
use std::collections::{HashMap, HashSet};
use std::path::Path as StdPath;

const PARKED_NODE_ORDER_OFFSET: i64 = 1_000_000_000;
const TEMP_NODE_ORDER_OFFSET: i64 = 2_000_000_000;

type ParsedFileRow = (files::Model, ParsedFile);

pub(crate) async fn parse_file_rows(rows: &[files::Model]) -> Vec<(files::Model, ParsedFile)> {
    let relative_paths = rows
        .iter()
        .map(|file| file.relative_path.clone())
        .collect::<Vec<_>>();
    let parsed_rows = parse_files(relative_paths).await;

    rows.iter()
        .cloned()
        .zip(parsed_rows.into_iter())
        .collect::<Vec<_>>()
}

pub(crate) async fn find_roots_for_file_ids(
    pool: &DatabaseConnection,
    file_ids: &[String],
) -> anyhow::Result<HashSet<String>> {
    if file_ids.is_empty() {
        return Ok(HashSet::new());
    }

    let root_ids = node_files::Entity::find()
        .join(JoinType::InnerJoin, node_files::Relation::Nodes.def())
        .filter(node_files::Column::FileId.is_in(file_ids.to_vec()))
        .select_only()
        .column(nodes::Column::RootId)
        .distinct()
        .into_tuple::<String>()
        .all(pool)
        .await?;

    Ok(root_ids.into_iter().collect())
}

pub(crate) async fn reconcile_root(
    pool: &DatabaseConnection,
    library_id: &str,
    library_root: &StdPath,
    root_id: &str,
    extra_rows: Vec<(files::Model, ParsedFile)>,
) -> anyhow::Result<()> {
    let existing_file_rows = load_root_file_rows(pool, library_id, root_id).await?;
    let parsed_existing_rows = parse_file_rows(&existing_file_rows).await;
    let parsed_rows = merge_parsed_file_rows(parsed_existing_rows, extra_rows);
    let root_plans = build_root_materialization_plans(library_root, &parsed_rows);
    let Some(plan) = root_plans.get(root_id) else {
        tracing::warn!(
            root_id,
            "touched root has no derived plan after reconciliation input"
        );
        return Ok(());
    };

    materialize_touched_root(pool, library_id, plan).await
}

async fn load_root_file_rows(
    pool: &DatabaseConnection,
    library_id: &str,
    root_id: &str,
) -> anyhow::Result<Vec<files::Model>> {
    let file_ids = node_files::Entity::find()
        .join(JoinType::InnerJoin, node_files::Relation::Nodes.def())
        .filter(nodes::Column::LibraryId.eq(library_id))
        .filter(nodes::Column::RootId.eq(root_id))
        .select_only()
        .column(node_files::Column::FileId)
        .distinct()
        .into_tuple::<String>()
        .all(pool)
        .await?;

    if file_ids.is_empty() {
        return Ok(Vec::new());
    }

    Ok(files::Entity::find()
        .filter(files::Column::Id.is_in(file_ids))
        .all(pool)
        .await?)
}

fn merge_parsed_file_rows(
    existing_rows: Vec<ParsedFileRow>,
    extra_rows: Vec<ParsedFileRow>,
) -> Vec<ParsedFileRow> {
    let mut rows_by_file_id = HashMap::new();

    for (file, parsed) in existing_rows {
        rows_by_file_id.insert(file.id.clone(), (file, parsed));
    }
    for (file, parsed) in extra_rows {
        rows_by_file_id.insert(file.id.clone(), (file, parsed));
    }

    let mut rows = rows_by_file_id.into_values().collect::<Vec<_>>();
    rows.sort_by(|a, b| a.0.id.cmp(&b.0.id));
    rows
}

pub(crate) async fn materialize_touched_root(
    pool: &DatabaseConnection,
    library_id: &str,
    plan: &RootMaterializationPlan,
) -> anyhow::Result<()> {
    verify_root_nodes(&plan.wanted_nodes.values().cloned().collect::<Vec<_>>())?;

    let now = chrono::Utc::now().timestamp();
    let txn = pool.begin().await?;

    let existing_nodes = nodes::Entity::find()
        .filter(nodes::Column::LibraryId.eq(library_id))
        .filter(nodes::Column::RootId.eq(plan.root_id.clone()))
        .all(&txn)
        .await?;
    let existing_node_ids = existing_nodes
        .iter()
        .map(|node| node.id.clone())
        .collect::<HashSet<_>>();

    let sorted_wanted_nodes = sort_nodes_topologically(&plan.wanted_nodes)?;
    let desired_node_ids = sorted_wanted_nodes
        .iter()
        .map(|node| node.id.clone())
        .collect::<HashSet<_>>();

    let node_rows = sorted_wanted_nodes
        .iter()
        .enumerate()
        .map(|(temp_order, wanted)| nodes::ActiveModel {
            id: Set(wanted.id.clone()),
            library_id: Set(library_id.to_owned()),
            root_id: Set(wanted.root_id.clone()),
            parent_id: Set(wanted.parent_id.clone()),
            kind: Set(wanted.kind),
            name: Set(wanted.name.clone()),
            order: Set(TEMP_NODE_ORDER_OFFSET + temp_order as i64),
            season_number: Set(wanted.season_number),
            episode_number: Set(wanted.episode_number),
            match_candidates_json: Set(None),
            last_added_at: Set(wanted.last_added_at),
            created_at: Set(now),
            updated_at: Set(now),
        })
        .collect::<Vec<_>>();
    if !node_rows.is_empty() {
        nodes::Entity::insert_many(node_rows)
            .on_conflict(
                OnConflict::column(nodes::Column::Id)
                    .update_columns([
                        nodes::Column::LibraryId,
                        nodes::Column::RootId,
                        nodes::Column::ParentId,
                        nodes::Column::Kind,
                        nodes::Column::Name,
                        nodes::Column::Order,
                        nodes::Column::SeasonNumber,
                        nodes::Column::EpisodeNumber,
                        nodes::Column::LastAddedAt,
                        nodes::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec(&txn)
            .await?;
    }

    if !existing_node_ids.is_empty() {
        node_closure::Entity::delete_many()
            .filter(node_closure::Column::DescendantId.is_in(existing_node_ids.iter().cloned()))
            .exec(&txn)
            .await?;

        node_files::Entity::delete_many()
            .filter(node_files::Column::NodeId.is_in(existing_node_ids.iter().cloned()))
            .exec(&txn)
            .await?;

        node_metadata::Entity::delete_many()
            .filter(node_metadata::Column::NodeId.is_in(existing_node_ids.iter().cloned()))
            .filter(node_metadata::Column::Source.eq(MetadataSource::Local))
            .exec(&txn)
            .await?;
    }

    let closure_rows = build_closure_rows(
        &plan.wanted_nodes,
        &sorted_wanted_nodes
            .iter()
            .map(|node| node.id.clone())
            .collect::<Vec<_>>(),
    )?;
    if !closure_rows.is_empty() {
        node_closure::Entity::insert_many(closure_rows.into_iter().map(|row| {
            node_closure::ActiveModel {
                ancestor_id: Set(row.ancestor_id),
                descendant_id: Set(row.descendant_id),
                depth: Set(row.depth),
            }
        }))
        .exec(&txn)
        .await?;
    }

    let node_file_rows = sorted_wanted_nodes
        .iter()
        .flat_map(|node| {
            node.attached_file_ids
                .iter()
                .enumerate()
                .map(move |(order, file_id)| node_files::ActiveModel {
                    node_id: Set(node.id.clone()),
                    file_id: Set(file_id.clone()),
                    order: Set(order as i64),
                    created_at: Set(now),
                    updated_at: Set(now),
                })
        })
        .collect::<Vec<_>>();
    if !node_file_rows.is_empty() {
        node_files::Entity::insert_many(node_file_rows)
            .exec(&txn)
            .await?;
    }

    let local_rows = sorted_wanted_nodes
        .iter()
        .map(|node| node_metadata::ActiveModel {
            id: Set(ids::generate_ulid()),
            node_id: Set(node.id.clone()),
            source: Set(MetadataSource::Local),
            provider_id: Set("local".to_owned()),
            imdb_id: Set(node.imdb_id.clone()),
            tmdb_id: Set(node.tmdb_id),
            name: Set(node.name.clone()),
            description: Set(None),
            score_display: Set(None),
            score_normalized: Set(None),
            released_at: Set(None),
            ended_at: Set(None),
            poster_asset_id: Set(None),
            thumbnail_asset_id: Set(None),
            background_asset_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        })
        .collect::<Vec<_>>();
    if !local_rows.is_empty() {
        node_metadata::Entity::insert_many(local_rows)
            .exec(&txn)
            .await?;
    }

    let obsolete_node_ids = existing_node_ids
        .difference(&desired_node_ids)
        .cloned()
        .collect::<Vec<_>>();
    if !obsolete_node_ids.is_empty() {
        nodes::Entity::delete_many()
            .filter(nodes::Column::Id.is_in(obsolete_node_ids))
            .exec(&txn)
            .await?;
    }

    txn.commit().await?;
    recompute_root_orders_with_sqlx(pool, &plan.root_id, now).await?;
    Ok(())
}

async fn recompute_root_orders_with_sqlx(
    pool: &DatabaseConnection,
    root_id: &str,
    now: i64,
) -> anyhow::Result<()> {
    let mut txn = pool.get_sqlite_connection_pool().begin().await?;

    sqlx::query!(
        r#"UPDATE nodes SET "order" = "order" + ? WHERE root_id = ?"#,
        PARKED_NODE_ORDER_OFFSET,
        root_id,
    )
    .execute(&mut *txn)
    .await?;

    sqlx::query!(
        r#"
        WITH ranked AS (
            SELECT
                id,
                row_number() OVER (
                    ORDER BY
                        CASE WHEN parent_id IS NULL THEN 0 ELSE 1 END,
                        COALESCE(season_number, 0),
                        CASE WHEN kind = 2 THEN 0 ELSE 1 END,
                        COALESCE(episode_number, 0),
                        id
                ) - 1 AS new_order
            FROM nodes
            WHERE root_id = ?
        )
        UPDATE nodes
        SET "order" = (
            SELECT new_order
            FROM ranked
            WHERE ranked.id = nodes.id
        )
        WHERE root_id = ?
        "#,
        root_id,
        root_id,
    )
    .execute(&mut *txn)
    .await?;

    sqlx::query!(
        r#"
        WITH ranked AS (
            SELECT
                nf.node_id,
                nf.file_id,
                row_number() OVER (
                    PARTITION BY nf.node_id
                    ORDER BY f.size_bytes DESC, nf.file_id ASC
                ) - 1 AS new_order
            FROM node_files nf
            INNER JOIN files f ON f.id = nf.file_id
            INNER JOIN nodes n ON n.id = nf.node_id
            WHERE n.root_id = ?
            AND n.kind IN (0, 3)
        )
        UPDATE node_files
        SET
            "order" = (
                SELECT new_order
                FROM ranked
                WHERE ranked.node_id = node_files.node_id
                AND ranked.file_id = node_files.file_id
            ),
            updated_at = ?
        WHERE node_id IN (
            SELECT id
            FROM nodes
            WHERE root_id = ?
            AND kind IN (0, 3)
        )
        "#,
        root_id,
        now,
        root_id,
    )
    .execute(&mut *txn)
    .await?;

    txn.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::libraries;
    use sea_orm::{Database, QueryOrder};

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

    async fn insert_file(
        pool: &DatabaseConnection,
        id: &str,
        relative_path: &str,
        size_bytes: i64,
        discovered_at: i64,
    ) -> anyhow::Result<()> {
        files::Entity::insert(files::ActiveModel {
            id: Set(id.to_owned()),
            library_id: Set("lib".to_owned()),
            relative_path: Set(relative_path.to_owned()),
            size_bytes: Set(size_bytes),
            audio_fingerprint: Set(Vec::new()),
            segments_json: Set(Vec::new()),
            keyframes_json: Set(Vec::new()),
            unavailable_at: Set(None),
            scanned_at: Set(Some(discovered_at)),
            discovered_at: Set(discovered_at),
            ..Default::default()
        })
        .exec(pool)
        .await?;
        Ok(())
    }

    #[tokio::test]
    async fn materialize_touched_root_recomputes_orders() -> anyhow::Result<()> {
        let pool = setup_test_db().await?;
        insert_library(&pool).await?;
        insert_file(&pool, "file-small", "show/s01e01-small.mkv", 100, 10).await?;
        insert_file(&pool, "file-large", "show/s01e01-large.mkv", 200, 11).await?;

        let root_id = "root".to_owned();
        let season_id = "season-1".to_owned();
        let episode_id = "episode-1".to_owned();
        let plan = RootMaterializationPlan {
            root_id: root_id.clone(),
            wanted_nodes: HashMap::from([
                (
                    root_id.clone(),
                    WantedNode {
                        id: root_id.clone(),
                        root_id: root_id.clone(),
                        parent_id: None,
                        kind: nodes::NodeKind::Series,
                        name: "Show".to_owned(),
                        season_number: None,
                        episode_number: None,
                        imdb_id: Some("tt1234567".to_owned()),
                        tmdb_id: Some(42),
                        last_added_at: 11,
                        attached_file_ids: Vec::new(),
                    },
                ),
                (
                    season_id.clone(),
                    WantedNode {
                        id: season_id.clone(),
                        root_id: root_id.clone(),
                        parent_id: Some(root_id.clone()),
                        kind: nodes::NodeKind::Season,
                        name: "Season 1".to_owned(),
                        season_number: Some(1),
                        episode_number: None,
                        imdb_id: None,
                        tmdb_id: None,
                        last_added_at: 11,
                        attached_file_ids: Vec::new(),
                    },
                ),
                (
                    episode_id.clone(),
                    WantedNode {
                        id: episode_id.clone(),
                        root_id: root_id.clone(),
                        parent_id: Some(season_id.clone()),
                        kind: nodes::NodeKind::Episode,
                        name: "Episode 1".to_owned(),
                        season_number: Some(1),
                        episode_number: Some(1),
                        imdb_id: None,
                        tmdb_id: None,
                        last_added_at: 11,
                        attached_file_ids: vec!["file-small".to_owned(), "file-large".to_owned()],
                    },
                ),
            ]),
        };

        materialize_touched_root(&pool, "lib", &plan).await?;

        let rows = nodes::Entity::find()
            .filter(nodes::Column::RootId.eq(root_id.clone()))
            .order_by_asc(nodes::Column::Order)
            .all(&pool)
            .await?;
        assert_eq!(
            rows.iter().map(|row| row.id.as_str()).collect::<Vec<_>>(),
            vec!["root", "season-1", "episode-1"]
        );

        let links = node_files::Entity::find()
            .filter(node_files::Column::NodeId.eq(episode_id.clone()))
            .order_by_asc(node_files::Column::Order)
            .all(&pool)
            .await?;
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].file_id, "file-large");
        assert_eq!(links[1].file_id, "file-small");

        let metadata_rows = node_metadata::Entity::find()
            .filter(node_metadata::Column::Source.eq(MetadataSource::Local))
            .order_by_asc(node_metadata::Column::NodeId)
            .all(&pool)
            .await?;
        assert_eq!(metadata_rows.len(), 3);
        assert_eq!(
            metadata_rows
                .iter()
                .find(|row| row.node_id == root_id)
                .and_then(|row| row.imdb_id.clone())
                .as_deref(),
            Some("tt1234567")
        );

        Ok(())
    }

    #[tokio::test]
    async fn materialize_touched_root_preserves_match_candidates_and_local_row_count()
    -> anyhow::Result<()> {
        let pool = setup_test_db().await?;
        insert_library(&pool).await?;
        insert_file(&pool, "movie-file", "movie/movie.mkv", 300, 20).await?;

        nodes::Entity::insert(nodes::ActiveModel {
            id: Set("movie-root".to_owned()),
            library_id: Set("lib".to_owned()),
            root_id: Set("movie-root".to_owned()),
            parent_id: Set(None),
            kind: Set(nodes::NodeKind::Movie),
            name: Set("Old Movie".to_owned()),
            order: Set(0),
            season_number: Set(None),
            episode_number: Set(None),
            match_candidates_json: Set(Some(vec![1, 2, 3])),
            last_added_at: Set(1),
            created_at: Set(1),
            updated_at: Set(1),
        })
        .exec(&pool)
        .await?;

        let plan = RootMaterializationPlan {
            root_id: "movie-root".to_owned(),
            wanted_nodes: HashMap::from([(
                "movie-root".to_owned(),
                WantedNode {
                    id: "movie-root".to_owned(),
                    root_id: "movie-root".to_owned(),
                    parent_id: None,
                    kind: nodes::NodeKind::Movie,
                    name: "New Movie".to_owned(),
                    season_number: None,
                    episode_number: None,
                    imdb_id: Some("tt7654321".to_owned()),
                    tmdb_id: Some(7),
                    last_added_at: 20,
                    attached_file_ids: vec!["movie-file".to_owned()],
                },
            )]),
        };

        materialize_touched_root(&pool, "lib", &plan).await?;
        materialize_touched_root(&pool, "lib", &plan).await?;

        let row = nodes::Entity::find_by_id("movie-root")
            .one(&pool)
            .await?
            .expect("movie root missing");
        assert_eq!(row.match_candidates_json, Some(vec![1, 2, 3]));

        let metadata_rows = node_metadata::Entity::find()
            .filter(node_metadata::Column::NodeId.eq("movie-root"))
            .filter(node_metadata::Column::Source.eq(MetadataSource::Local))
            .all(&pool)
            .await?;
        assert_eq!(metadata_rows.len(), 1);
        assert_eq!(metadata_rows[0].name, "New Movie");
        assert_eq!(metadata_rows[0].imdb_id.as_deref(), Some("tt7654321"));

        let links = node_files::Entity::find()
            .filter(node_files::Column::NodeId.eq("movie-root"))
            .all(&pool)
            .await?;
        assert_eq!(links.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn materialize_touched_root_replaces_stale_series_shape() -> anyhow::Result<()> {
        let pool = setup_test_db().await?;
        insert_library(&pool).await?;
        insert_file(&pool, "new-file", "show/s01e02.mkv", 100, 30).await?;

        nodes::Entity::insert_many([
            nodes::ActiveModel {
                id: Set("root".to_owned()),
                library_id: Set("lib".to_owned()),
                root_id: Set("root".to_owned()),
                parent_id: Set(None),
                kind: Set(nodes::NodeKind::Series),
                name: Set("Show".to_owned()),
                order: Set(0),
                season_number: Set(None),
                episode_number: Set(None),
                match_candidates_json: Set(None),
                last_added_at: Set(1),
                created_at: Set(1),
                updated_at: Set(1),
            },
            nodes::ActiveModel {
                id: Set("episode-1".to_owned()),
                library_id: Set("lib".to_owned()),
                root_id: Set("root".to_owned()),
                parent_id: Set(Some("root".to_owned())),
                kind: Set(nodes::NodeKind::Episode),
                name: Set("Episode 1".to_owned()),
                order: Set(1),
                season_number: Set(None),
                episode_number: Set(Some(1)),
                match_candidates_json: Set(None),
                last_added_at: Set(1),
                created_at: Set(1),
                updated_at: Set(1),
            },
        ])
        .exec(&pool)
        .await?;

        let plan = RootMaterializationPlan {
            root_id: "root".to_owned(),
            wanted_nodes: HashMap::from([
                (
                    "root".to_owned(),
                    WantedNode {
                        id: "root".to_owned(),
                        root_id: "root".to_owned(),
                        parent_id: None,
                        kind: nodes::NodeKind::Series,
                        name: "Show".to_owned(),
                        season_number: None,
                        episode_number: None,
                        imdb_id: None,
                        tmdb_id: None,
                        last_added_at: 30,
                        attached_file_ids: Vec::new(),
                    },
                ),
                (
                    "season-1".to_owned(),
                    WantedNode {
                        id: "season-1".to_owned(),
                        root_id: "root".to_owned(),
                        parent_id: Some("root".to_owned()),
                        kind: nodes::NodeKind::Season,
                        name: "Season 1".to_owned(),
                        season_number: Some(1),
                        episode_number: None,
                        imdb_id: None,
                        tmdb_id: None,
                        last_added_at: 30,
                        attached_file_ids: Vec::new(),
                    },
                ),
                (
                    "episode-2".to_owned(),
                    WantedNode {
                        id: "episode-2".to_owned(),
                        root_id: "root".to_owned(),
                        parent_id: Some("season-1".to_owned()),
                        kind: nodes::NodeKind::Episode,
                        name: "Episode 2".to_owned(),
                        season_number: Some(1),
                        episode_number: Some(2),
                        imdb_id: None,
                        tmdb_id: None,
                        last_added_at: 30,
                        attached_file_ids: vec!["new-file".to_owned()],
                    },
                ),
            ]),
        };

        materialize_touched_root(&pool, "lib", &plan).await?;

        let season = nodes::Entity::find_by_id("season-1")
            .one(&pool)
            .await?
            .expect("season missing");
        assert_eq!(season.parent_id.as_deref(), Some("root"));

        let old_episode = nodes::Entity::find_by_id("episode-1").one(&pool).await?;
        assert!(old_episode.is_none());

        Ok(())
    }
}
