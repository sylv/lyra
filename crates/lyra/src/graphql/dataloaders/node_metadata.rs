use crate::entities::{node_metadata, nodes};
use async_graphql::dataloader::Loader;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct PreferredNodeMetadata {
    pub metadata: Option<node_metadata::Model>,
    pub node_name: String,
}

impl PreferredNodeMetadata {
    pub fn display_name(&self) -> &str {
        self.metadata
            .as_ref()
            .map(|metadata| metadata.name.as_str())
            .unwrap_or(self.node_name.as_str())
    }

    pub fn poster_asset_id(&self) -> Option<&str> {
        self.metadata
            .as_ref()
            .and_then(|metadata| metadata.poster_asset_id.as_deref())
    }
}

#[derive(Clone)]
pub struct NodeMetadataLoader {
    pool: DatabaseConnection,
}

impl NodeMetadataLoader {
    pub fn new(pool: DatabaseConnection) -> Self {
        Self { pool }
    }
}

impl Loader<String> for NodeMetadataLoader {
    type Value = PreferredNodeMetadata;
    type Error = String;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        let nodes = nodes::Entity::find()
            .filter(nodes::Column::Id.is_in(keys.to_vec()))
            .all(&self.pool)
            .await
            .map_err(|error| error.to_string())?;

        let metadata_rows = node_metadata::Entity::find()
            .filter(node_metadata::Column::NodeId.is_in(keys.to_vec()))
            .order_by_asc(node_metadata::Column::NodeId)
            .order_by_desc(node_metadata::Column::Source)
            .order_by_desc(node_metadata::Column::UpdatedAt)
            .all(&self.pool)
            .await
            .map_err(|error| error.to_string())?;

        let mut preferred_by_node_id = HashMap::new();
        for metadata in metadata_rows {
            preferred_by_node_id
                .entry(metadata.node_id.clone())
                .or_insert(metadata);
        }

        Ok(nodes
            .into_iter()
            .map(|node| {
                let metadata = preferred_by_node_id.remove(&node.id);
                (
                    node.id.clone(),
                    PreferredNodeMetadata {
                        metadata,
                        node_name: node.name,
                    },
                )
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{libraries, metadata_source::MetadataSource, node_metadata, nodes};
    use sea_orm::{ActiveValue::Set, Database};

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

    async fn insert_node(pool: &DatabaseConnection, id: &str) -> anyhow::Result<()> {
        nodes::Entity::insert(nodes::ActiveModel {
            id: Set(id.to_owned()),
            library_id: Set("lib".to_owned()),
            root_id: Set(id.to_owned()),
            parent_id: Set(None),
            kind: Set(nodes::NodeKind::Movie),
            name: Set("Movie".to_owned()),
            order: Set(0),
            season_number: Set(None),
            episode_number: Set(None),
            last_added_at: Set(0),
            unavailable_at: Set(None),
            created_at: Set(0),
            updated_at: Set(0),
        })
        .exec(pool)
        .await?;
        Ok(())
    }

    #[tokio::test]
    async fn loader_returns_remote_before_local() -> anyhow::Result<()> {
        let pool = setup_test_db().await?;
        insert_library(&pool).await?;
        insert_node(&pool, "movie").await?;

        node_metadata::Entity::insert_many([
            node_metadata::ActiveModel {
                id: Set("local".to_owned()),
                node_id: Set("movie".to_owned()),
                source: Set(MetadataSource::Local),
                provider_id: Set("local".to_owned()),
                name: Set("Local".to_owned()),
                created_at: Set(1),
                updated_at: Set(1),
                ..Default::default()
            },
            node_metadata::ActiveModel {
                id: Set("remote".to_owned()),
                node_id: Set("movie".to_owned()),
                source: Set(MetadataSource::Remote),
                provider_id: Set("tmdb".to_owned()),
                name: Set("Remote".to_owned()),
                created_at: Set(2),
                updated_at: Set(2),
                ..Default::default()
            },
        ])
        .exec(&pool)
        .await?;

        let loaded: HashMap<String, PreferredNodeMetadata> = NodeMetadataLoader::new(pool)
            .load(&["movie".to_owned()])
            .await
            .map_err(anyhow::Error::msg)?;
        let metadata = loaded
            .get("movie")
            .and_then(|loaded| loaded.metadata.as_ref());

        assert_eq!(metadata.map(|row| row.name.as_str()), Some("Remote"));

        Ok(())
    }
}
