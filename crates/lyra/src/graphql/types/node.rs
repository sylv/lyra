use crate::entities::{
    collection_items, metadata_source::MetadataSource, node_closure, node_files, node_metadata,
    node_metadata_recommendations, node_metadata_recommendations::RecommendationMediaKind, nodes,
    watch_progress,
};
use crate::graphql::dataloaders::node_counts::{NodeCounts, NodeCountsLoader};
use crate::graphql::dataloaders::node_metadata::NodeMetadataLoader;
use crate::graphql::properties::NodeProperties;
use crate::graphql::query::current_user_id;
use async_graphql::dataloader::DataLoader;
use async_graphql::{ComplexObject, Context};
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
    RelationTrait,
};

async fn previous_or_next_playable(
    pool: &DatabaseConnection,
    current: &nodes::Model,
    forward: bool,
) -> Result<Option<nodes::Model>, sea_orm::DbErr> {
    let query = nodes::Entity::find()
        .filter(nodes::Column::RootId.eq(current.root_id.clone()))
        .filter(
            Condition::any()
                .add(nodes::Column::Kind.eq(nodes::NodeKind::Movie))
                .add(nodes::Column::Kind.eq(nodes::NodeKind::Episode)),
        )
        .filter(nodes::Column::UnavailableAt.is_null())
        .filter(if forward {
            Condition::any()
                .add(nodes::Column::Order.gt(current.order))
                .add(
                    Condition::all()
                        .add(nodes::Column::Order.eq(current.order))
                        .add(nodes::Column::Id.gt(current.id.clone())),
                )
        } else {
            Condition::any()
                .add(nodes::Column::Order.lt(current.order))
                .add(
                    Condition::all()
                        .add(nodes::Column::Order.eq(current.order))
                        .add(nodes::Column::Id.lt(current.id.clone())),
                )
        });

    let query = if forward {
        query
            .order_by_asc(nodes::Column::Order)
            .order_by_asc(nodes::Column::Id)
    } else {
        query
            .order_by_desc(nodes::Column::Order)
            .order_by_desc(nodes::Column::Id)
    };

    let candidates = query.all(pool).await?;
    let current_file_ids = node_files::Entity::find()
        .filter(node_files::Column::NodeId.eq(current.id.clone()))
        .order_by_asc(node_files::Column::Order)
        .order_by_asc(node_files::Column::FileId)
        .all(pool)
        .await?
        .into_iter()
        .map(|link| link.file_id)
        .collect::<Vec<_>>();

    for candidate in candidates {
        let candidate_file_ids = node_files::Entity::find()
            .filter(node_files::Column::NodeId.eq(candidate.id.clone()))
            .order_by_asc(node_files::Column::Order)
            .order_by_asc(node_files::Column::FileId)
            .all(pool)
            .await?
            .into_iter()
            .map(|link| link.file_id)
            .collect::<Vec<_>>();

        if candidate_file_ids != current_file_ids {
            return Ok(Some(candidate));
        }
    }

    Ok(None)
}

fn is_playable_node(node: &nodes::Model) -> bool {
    matches!(node.kind, nodes::NodeKind::Movie | nodes::NodeKind::Episode)
}

async fn load_node_counts(ctx: &Context<'_>, node_id: &str) -> Result<NodeCounts, sea_orm::DbErr> {
    let loader = ctx.data_unchecked::<DataLoader<NodeCountsLoader>>();
    Ok(loader
        .load_one(node_id.to_owned())
        .await
        .map_err(sea_orm::DbErr::Custom)?
        .unwrap_or_default())
}

#[ComplexObject]
impl nodes::Model {
    pub async fn root(&self, ctx: &Context<'_>) -> Result<Option<nodes::Model>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        nodes::Entity::find_by_id(self.root_id.clone())
            .one(pool)
            .await
    }

    pub async fn parent(&self, ctx: &Context<'_>) -> Result<Option<nodes::Model>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let Some(parent_id) = &self.parent_id else {
            return Ok(None);
        };

        nodes::Entity::find_by_id(parent_id.clone()).one(pool).await
    }

    pub async fn children(&self, ctx: &Context<'_>) -> Result<Vec<nodes::Model>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        nodes::Entity::find()
            .filter(nodes::Column::ParentId.eq(self.id.clone()))
            .order_by_asc(nodes::Column::Order)
            .order_by_asc(nodes::Column::Id)
            .all(pool)
            .await
    }

    pub async fn properties(&self, ctx: &Context<'_>) -> Result<NodeProperties, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let loader = ctx.data_unchecked::<DataLoader<NodeMetadataLoader>>();
        let metadata = loader
            .load_one(self.id.clone())
            .await
            .map_err(sea_orm::DbErr::Custom)?;
        NodeProperties::from_node(pool, self, metadata).await
    }

    pub async fn default_file(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<crate::entities::files::Model>, sea_orm::DbErr> {
        if !is_playable_node(self) {
            return Ok(None);
        }

        let pool = ctx.data_unchecked::<DatabaseConnection>();
        NodeProperties::primary_file_for_node(pool, &self.id).await
    }

    pub async fn watch_progress(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<watch_progress::Model>, async_graphql::Error> {
        let Some(user_id) = current_user_id(ctx) else {
            return Ok(None);
        };

        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let progress = watch_progress::Entity::find()
            .filter(watch_progress::Column::UserId.eq(user_id))
            .filter(watch_progress::Column::NodeId.eq(self.id.clone()))
            .one(pool)
            .await?;
        Ok(progress)
    }

    pub async fn in_watchlist(&self, ctx: &Context<'_>) -> Result<bool, async_graphql::Error> {
        let Some(user_id) = current_user_id(ctx) else {
            return Ok(false);
        };

        let pool = ctx.data_unchecked::<DatabaseConnection>();
        Ok(
            collection_items::Entity::find_by_id((user_id, self.id.clone()))
                .one(pool)
                .await?
                .is_some(),
        )
    }

    pub async fn current_playable(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<nodes::Model>, sea_orm::DbErr> {
        // This is the playable target for a node, distinct from next_playable which is
        // relative navigation from an already-playing item.
        if is_playable_node(self) {
            return Ok(Some(self.clone()));
        }

        let pool = ctx.data_unchecked::<DatabaseConnection>();
        previous_or_next_playable(pool, self, true).await
    }

    pub async fn next_playable(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<nodes::Model>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        previous_or_next_playable(pool, self, true).await
    }

    pub async fn previous_playable(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<nodes::Model>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        previous_or_next_playable(pool, self, false).await
    }

    pub async fn recommended_nodes(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<nodes::Model>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let loader = ctx.data_unchecked::<DataLoader<NodeMetadataLoader>>();
        let Some(metadata) = loader
            .load_one(self.id.clone())
            .await
            .map_err(sea_orm::DbErr::Custom)?
            .and_then(|loaded| loaded.metadata)
        else {
            return Ok(Vec::new());
        };

        let rows = node_metadata_recommendations::Entity::find()
            .filter(node_metadata_recommendations::Column::NodeMetadataId.eq(metadata.id))
            .order_by_asc(node_metadata_recommendations::Column::Position)
            .all(pool)
            .await?;

        let mut result = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for row in rows {
            let mut query = nodes::Entity::find()
                .join(
                    sea_orm::JoinType::InnerJoin,
                    nodes::Relation::NodeMetadata.def(),
                )
                .filter(nodes::Column::ParentId.is_null())
                .filter(nodes::Column::Id.ne(self.id.clone()))
                .filter(node_metadata::Column::Source.eq(MetadataSource::Remote))
                .filter(node_metadata::Column::ProviderId.eq(row.provider_id.clone()));

            if let Some(tmdb_id) = row.tmdb_id {
                query = query.filter(node_metadata::Column::TmdbId.eq(tmdb_id));
            } else if let Some(imdb_id) = row.imdb_id.clone() {
                query = query.filter(node_metadata::Column::ImdbId.eq(imdb_id));
            } else {
                continue;
            }

            match row.media_kind {
                RecommendationMediaKind::Movie => {
                    query = query.filter(nodes::Column::Kind.eq(nodes::NodeKind::Movie));
                }
                RecommendationMediaKind::Series => {
                    query = query.filter(nodes::Column::Kind.eq(nodes::NodeKind::Series));
                }
            }

            if let Some(node) = query.order_by_asc(nodes::Column::Id).one(pool).await?
                && seen.insert(node.id.clone())
            {
                result.push(node);
            }
        }

        Ok(result)
    }

    pub async fn unplayed_count(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<i64>, async_graphql::Error> {
        if !matches!(self.kind, nodes::NodeKind::Series | nodes::NodeKind::Season) {
            return Ok(None);
        }

        let Some(user_id) = current_user_id(ctx) else {
            return Ok(None);
        };

        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let descendant_ids: Vec<String> = node_closure::Entity::find()
            .filter(node_closure::Column::AncestorId.eq(self.id.clone()))
            .select_only()
            .column(node_closure::Column::DescendantId)
            .into_tuple::<String>()
            .all(pool)
            .await?;

        if descendant_ids.is_empty() {
            return Ok(Some(0));
        }

        let playable_ids: Vec<String> = nodes::Entity::find()
            .filter(nodes::Column::Id.is_in(descendant_ids))
            .filter(
                Condition::any()
                    .add(nodes::Column::Kind.eq(nodes::NodeKind::Movie))
                    .add(nodes::Column::Kind.eq(nodes::NodeKind::Episode)),
            )
            .select_only()
            .column(nodes::Column::Id)
            .into_tuple::<String>()
            .all(pool)
            .await?;

        if playable_ids.is_empty() {
            return Ok(Some(0));
        }

        let watched_ids: Vec<String> = watch_progress::Entity::find()
            .filter(watch_progress::Column::UserId.eq(user_id))
            .filter(watch_progress::Column::NodeId.is_in(playable_ids.clone()))
            .filter(
                watch_progress::Column::ProgressPercent
                    .gt(watch_progress::completed_progress_threshold()),
            )
            .select_only()
            .column(watch_progress::Column::NodeId)
            .into_tuple::<String>()
            .all(pool)
            .await?;

        Ok(Some(
            playable_ids.len().saturating_sub(watched_ids.len()) as i64
        ))
    }

    pub async fn season_count(&self, ctx: &Context<'_>) -> Result<i64, sea_orm::DbErr> {
        Ok(load_node_counts(ctx, &self.id).await?.season_count)
    }

    pub async fn episode_count(&self, ctx: &Context<'_>) -> Result<i64, sea_orm::DbErr> {
        Ok(load_node_counts(ctx, &self.id).await?.episode_count)
    }
}
