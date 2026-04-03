use crate::entities::collections::{
    self, CollectionKind, CollectionResolverKind, CollectionVisibility,
};
use crate::graphql::query::{NodeFilter, OrderBy, OrderDirection};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    Set,
};

const CONTINUE_WATCHING_ID: &str = "system:continue-watching";

fn continue_watching_filter() -> NodeFilter {
    NodeFilter {
        library_id: None,
        root_id: None,
        parent_id: None,
        kinds: None,
        search_term: None,
        availability: None,
        order_by: Some(OrderBy::WatchProgressUpdatedAt),
        order_direction: Some(OrderDirection::Desc),
        watched: None,
        continue_watching: Some(true),
    }
}

pub async fn reconcile_system_collections(pool: &DatabaseConnection) -> anyhow::Result<()> {
    let filter_json = serde_json::to_vec(&continue_watching_filter())?;
    let Some(existing) = collections::Entity::find()
        .filter(collections::Column::Kind.eq(CollectionKind::ContinueWatching.as_db()))
        .one(pool)
        .await?
    else {
        collections::Entity::insert(collections::ActiveModel {
            id: Set(CONTINUE_WATCHING_ID.to_string()),
            name: Set("Continue Watching".to_string()),
            description: Set(Some("Pick up where you left off".to_string())),
            created_by_id: Set(None),
            visibility: Set(CollectionVisibility::Private),
            resolver_kind: Set(CollectionResolverKind::Filter),
            kind: Set(Some(CollectionKind::ContinueWatching.as_db())),
            filter_json: Set(Some(filter_json)),
            show_on_home: Set(true),
            home_position: Set(0),
            pinned: Set(true),
            pinned_position: Set(0),
            ..Default::default()
        })
        .exec(pool)
        .await?;
        return Ok(());
    };

    if existing.id != CONTINUE_WATCHING_ID {
        collections::Entity::delete_by_id(existing.id.clone())
            .exec(pool)
            .await?;
        collections::Entity::insert(collections::ActiveModel {
            id: Set(CONTINUE_WATCHING_ID.to_string()),
            name: Set("Continue Watching".to_string()),
            description: Set(Some("Pick up where you left off".to_string())),
            created_by_id: Set(None),
            visibility: Set(CollectionVisibility::Private),
            resolver_kind: Set(CollectionResolverKind::Filter),
            kind: Set(Some(CollectionKind::ContinueWatching.as_db())),
            filter_json: Set(Some(serde_json::to_vec(&continue_watching_filter())?)),
            show_on_home: Set(true),
            home_position: Set(0),
            pinned: Set(true),
            pinned_position: Set(0),
            ..Default::default()
        })
        .exec(pool)
        .await?;
        return Ok(());
    }

    let mut active = existing.into_active_model();
    active.name = Set("Continue Watching".to_string());
    active.description = Set(Some("Pick up where you left off".to_string()));
    active.created_by_id = Set(None);
    active.visibility = Set(CollectionVisibility::Private);
    active.resolver_kind = Set(CollectionResolverKind::Filter);
    active.kind = Set(Some(CollectionKind::ContinueWatching.as_db()));
    active.filter_json = Set(Some(serde_json::to_vec(&continue_watching_filter())?));
    active.show_on_home = Set(true);
    active.home_position = Set(0);
    active.pinned = Set(true);
    active.pinned_position = Set(0);
    active.update(pool).await?;

    Ok(())
}
