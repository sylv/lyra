use crate::entities::{metadata_source::MetadataSource, node_metadata, nodes};
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, JoinType, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait, Select,
    sea_query::{Alias, Expr, Query},
};

pub async fn preferred_node_metadata(
    pool: &DatabaseConnection,
    node_id: &str,
) -> Result<Option<node_metadata::Model>, sea_orm::DbErr> {
    node_metadata::Entity::find()
        .filter(node_metadata::Column::NodeId.eq(node_id))
        .order_by_desc(node_metadata::Column::Source)
        .order_by_desc(node_metadata::Column::UpdatedAt)
        .one(pool)
        .await
}

pub fn join_preferred_node_metadata(mut query: Select<nodes::Entity>) -> Select<nodes::Entity> {
    query = query
        .join(JoinType::LeftJoin, nodes::Relation::NodeMetadata.def())
        .filter(preferred_node_metadata_condition());
    query
}

fn preferred_node_metadata_condition() -> Condition {
    let remote_rows = Alias::new("preferred_remote_rows");

    Condition::any()
        .add(node_metadata::Column::Id.is_null())
        .add(node_metadata::Column::Source.eq(MetadataSource::Remote))
        .add(
            Condition::all()
                .add(node_metadata::Column::Source.eq(MetadataSource::Local))
                .add(
                    Condition::all().not().add(Expr::exists(
                        Query::select()
                            .expr(Expr::val(1))
                            .from_as(node_metadata::Entity, remote_rows.clone())
                            .and_where(
                                Expr::col((remote_rows.clone(), node_metadata::Column::NodeId))
                                    .equals((node_metadata::Entity, node_metadata::Column::NodeId)),
                            )
                            .and_where(
                                Expr::col((remote_rows, node_metadata::Column::Source))
                                    .eq(MetadataSource::Remote),
                            )
                            .to_owned(),
                    )),
                ),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{libraries, node_metadata, nodes};
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
            last_scanned_at: Set(None),
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
            created_at: Set(0),
            updated_at: Set(0),
        })
        .exec(pool)
        .await?;
        Ok(())
    }

    #[tokio::test]
    async fn preferred_metadata_returns_remote_before_local() -> anyhow::Result<()> {
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

        let metadata = preferred_node_metadata(&pool, "movie").await?;
        assert_eq!(
            metadata.as_ref().map(|row| row.name.as_str()),
            Some("Remote")
        );

        Ok(())
    }
}
