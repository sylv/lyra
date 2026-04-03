use crate::auth::{RequestAuth, accessible_library_ids};
use crate::entities::{collection_items, collections, nodes, users};
use crate::graphql::query::{
    build_node_query_for_viewer, collection_editable_by_user, current_user_id, paginate_node_query,
};
use async_graphql::{
    ComplexObject, Context,
    connection::{self, EmptyFields},
};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, JoinType, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, RelationTrait,
};

pub async fn collection_item_count(
    ctx: &Context<'_>,
    collection: &collections::Model,
) -> Result<i64, async_graphql::Error> {
    let pool = ctx.data::<DatabaseConnection>()?;
    let auth = ctx.data::<RequestAuth>()?;
    let visible_library_ids = accessible_library_ids(pool, auth)
        .await
        .map_err(async_graphql::Error::from)?;
    let user_id = auth.get_user_or_err()?.id.clone();

    match collection.resolver_kind {
        collections::CollectionResolverKind::Filter => {
            let filter = collection
                .filter_json
                .as_deref()
                .map(serde_json::from_slice)
                .transpose()?
                .unwrap_or_default();
            let qb = build_node_query_for_viewer(
                pool,
                visible_library_ids.as_deref(),
                &user_id,
                &filter,
            )
            .await?;
            Ok(qb.count(pool).await? as i64)
        }
        collections::CollectionResolverKind::Manual => {
            let mut query = nodes::Entity::find()
                .join(JoinType::InnerJoin, nodes::Relation::CollectionItems.def())
                .filter(collection_items::Column::CollectionId.eq(collection.id.clone()))
                .filter(nodes::Column::UnavailableAt.is_null());

            if let Some(visible_library_ids) = visible_library_ids {
                if visible_library_ids.is_empty() {
                    return Ok(0);
                }
                query = query.filter(nodes::Column::LibraryId.is_in(visible_library_ids.to_vec()));
            }

            Ok(query.count(pool).await? as i64)
        }
    }
}

#[ComplexObject]
impl collections::Model {
    pub async fn kind(&self) -> Option<collections::CollectionKind> {
        self.kind.and_then(collections::CollectionKind::from_db)
    }

    pub async fn created_by(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<users::Model>, sea_orm::DbErr> {
        let Some(created_by_id) = &self.created_by_id else {
            return Ok(None);
        };

        let pool = ctx.data_unchecked::<DatabaseConnection>();
        users::Entity::find_by_id(created_by_id.clone())
            .one(pool)
            .await
    }

    pub async fn can_edit(&self, ctx: &Context<'_>) -> Result<bool, async_graphql::Error> {
        let auth = ctx.data::<RequestAuth>()?;
        let user = auth.get_user_or_err()?;
        Ok(collection_editable_by_user(
            self,
            &user.id,
            auth.has_permission(users::UserPerms::ADMIN),
        ))
    }

    pub async fn can_delete(&self, ctx: &Context<'_>) -> Result<bool, async_graphql::Error> {
        self.can_edit(ctx).await
    }

    pub async fn item_count(&self, ctx: &Context<'_>) -> Result<i64, async_graphql::Error> {
        collection_item_count(ctx, self).await
    }

    pub async fn node_list(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        first: Option<i32>,
    ) -> Result<
        connection::Connection<u64, nodes::Model, EmptyFields, EmptyFields>,
        async_graphql::Error,
    > {
        let pool = ctx.data::<DatabaseConnection>()?;
        let auth = ctx.data::<RequestAuth>()?;
        let visible_library_ids = accessible_library_ids(pool, auth)
            .await
            .map_err(async_graphql::Error::from)?;
        let user_id =
            current_user_id(ctx).ok_or_else(|| async_graphql::Error::new("Unauthenticated"))?;

        match self.resolver_kind {
            collections::CollectionResolverKind::Filter => {
                let filter = self
                    .filter_json
                    .as_deref()
                    .map(serde_json::from_slice)
                    .transpose()?
                    .unwrap_or_default();
                let qb = build_node_query_for_viewer(
                    pool,
                    visible_library_ids.as_deref(),
                    &user_id,
                    &filter,
                )
                .await?;
                paginate_node_query(pool, qb, after, first).await
            }
            collections::CollectionResolverKind::Manual => {
                connection::query(
                    after,
                    None,
                    first,
                    None,
                    |after, _before, first, _last| async move {
                        let mut query = nodes::Entity::find()
                            .join(JoinType::InnerJoin, nodes::Relation::CollectionItems.def())
                            .filter(collection_items::Column::CollectionId.eq(self.id.clone()))
                            .filter(nodes::Column::UnavailableAt.is_null())
                            .order_by_asc(collection_items::Column::Position)
                            .order_by_asc(nodes::Column::Id);

                        if let Some(visible_library_ids) = visible_library_ids.as_ref() {
                            if visible_library_ids.is_empty() {
                                return Ok::<_, async_graphql::Error>(connection::Connection::new(
                                    false, false,
                                ));
                            }
                            query = query.filter(
                                nodes::Column::LibraryId.is_in(visible_library_ids.clone()),
                            );
                        }

                        let count = query.clone().count(pool).await?;
                        let limit = first.unwrap_or(50) as u64;
                        let offset = after.map(|cursor| cursor + 1).unwrap_or(0);
                        let records: Vec<nodes::Model> = query
                            .limit(Some(limit))
                            .offset(Some(offset))
                            .all(pool)
                            .await?;

                        let has_previous_page = offset > 0;
                        let has_next_page = offset + limit < count;
                        let mut connection =
                            connection::Connection::new(has_previous_page, has_next_page);
                        connection.edges.extend(records.into_iter().enumerate().map(
                            |(index, node)| connection::Edge::new(offset + index as u64, node),
                        ));

                        Ok::<_, async_graphql::Error>(connection)
                    },
                )
                .await
            }
        }
    }
}
