use super::properties::NodeProperties;
use crate::{
    RequestAuth,
    entities::{
        files,
        metadata::{self},
        node_metadata,
        nodes::{self, NodeKind},
        watch_progress,
    },
};
use async_graphql::{ComplexObject, Context};
use sea_orm::entity::prelude::*;
use sea_orm::{
    ColumnTrait, EntityTrait, JoinType, QueryFilter, QueryOrder, QuerySelect, RelationTrait,
};

#[ComplexObject]
impl nodes::Model {
    pub async fn properties(&self, ctx: &Context<'_>) -> Result<NodeProperties, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let metadata = metadata::Entity::find()
            .join(JoinType::InnerJoin, metadata::Relation::NodeMetadata.def())
            .filter(node_metadata::Column::NodeId.eq(self.id.clone()))
            .filter(node_metadata::Column::IsPrimary.eq(true))
            .one(pool)
            .await?;

        Ok(NodeProperties::from_metadata(metadata, self.file_id))
    }

    pub async fn first_linked_at(&self, ctx: &Context<'_>) -> Result<Option<i64>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        if let Some(file_id) = self.file_id {
            let file = files::Entity::find_by_id(file_id).one(pool).await?;
            return Ok(file.map(|file| file.discovered_at));
        }

        Ok(None)
    }

    /// Gets the default file connection for this node, including child connections.
    pub async fn default_connection(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<files::Model>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();

        let target = match self.kind {
            NodeKind::Movie | NodeKind::Episode => {
                if let Some(file_id) = self.file_id {
                    files::Entity::find_by_id(file_id)
                        .filter(files::Column::UnavailableAt.is_null())
                        .one(pool)
                        .await?
                } else {
                    None
                }
            }
            NodeKind::Season => {
                let episode = nodes::Entity::find()
                    .filter(nodes::Column::ParentId.eq(self.id.clone()))
                    .filter(nodes::Column::Kind.eq(NodeKind::Episode))
                    .filter(nodes::Column::FileId.is_not_null())
                    .join(JoinType::LeftJoin, nodes::Relation::NodeMetadata.def())
                    .join(JoinType::LeftJoin, node_metadata::Relation::Metadata.def())
                    .filter(node_metadata::Column::IsPrimary.eq(true))
                    .order_by_asc(metadata::Column::EpisodeNumber)
                    .order_by_asc(nodes::Column::Id)
                    .one(pool)
                    .await?;

                if let Some(episode) = episode {
                    if let Some(file_id) = episode.file_id {
                        files::Entity::find_by_id(file_id)
                            .filter(files::Column::UnavailableAt.is_null())
                            .one(pool)
                            .await?
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            NodeKind::Series => {
                let episode = nodes::Entity::find()
                    .filter(nodes::Column::RootId.eq(self.id.clone()))
                    .filter(nodes::Column::Kind.eq(NodeKind::Episode))
                    .filter(nodes::Column::FileId.is_not_null())
                    .join(JoinType::LeftJoin, nodes::Relation::NodeMetadata.def())
                    .join(JoinType::LeftJoin, node_metadata::Relation::Metadata.def())
                    .filter(node_metadata::Column::IsPrimary.eq(true))
                    .order_by_asc(metadata::Column::SeasonNumber)
                    .order_by_asc(metadata::Column::EpisodeNumber)
                    .order_by_asc(nodes::Column::Id)
                    .one(pool)
                    .await?;

                if let Some(episode) = episode {
                    if let Some(file_id) = episode.file_id {
                        files::Entity::find_by_id(file_id)
                            .filter(files::Column::UnavailableAt.is_null())
                            .one(pool)
                            .await?
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        };

        Ok(target)
    }

    pub async fn seasons(&self, ctx: &Context<'_>) -> Result<Vec<i64>, sea_orm::DbErr> {
        if self.kind != NodeKind::Series {
            return Ok(vec![]);
        }

        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let result: Vec<i64> = metadata::Entity::find()
            .join(JoinType::InnerJoin, metadata::Relation::NodeMetadata.def())
            .join(JoinType::InnerJoin, node_metadata::Relation::Nodes.def())
            .filter(nodes::Column::ParentId.eq(self.id.clone()))
            .filter(nodes::Column::Kind.eq(NodeKind::Season))
            .filter(node_metadata::Column::IsPrimary.eq(true))
            .select_only()
            .column(metadata::Column::SeasonNumber)
            .distinct()
            .into_tuple()
            .all(pool)
            .await?;

        Ok(result)
    }

    pub async fn parent(&self, ctx: &Context<'_>) -> Result<Option<nodes::Model>, sea_orm::DbErr> {
        let Some(parent_id) = &self.parent_id else {
            return Ok(None);
        };

        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let parent = nodes::Entity::find_by_id(parent_id.clone())
            .one(pool)
            .await?;
        Ok(parent)
    }

    pub async fn watch_progress(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<watch_progress::Model>, async_graphql::Error> {
        let auth = ctx.data::<RequestAuth>()?;
        let user = auth.get_user_or_err()?;
        if self.kind != NodeKind::Episode && self.kind != NodeKind::Movie {
            return Ok(None);
        }

        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let progress = watch_progress::Entity::find()
            .filter(watch_progress::Column::UserId.eq(user.id.clone()))
            .filter(watch_progress::Column::NodeId.eq(self.id.clone()))
            .one(pool)
            .await?;

        Ok(progress)
    }
}
