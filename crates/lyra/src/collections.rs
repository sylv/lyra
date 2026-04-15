use crate::entities::collections::{
    self, CollectionKind, CollectionResolverKind, CollectionVisibility,
};
use crate::graphql::query::{NodeFilter, OrderBy, OrderDirection};
use crate::ids;
use chrono::{Duration, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    Set,
};

const CONTINUE_WATCHING_CARD_NAME: &str = "continue-watching";
const RECENTLY_RELEASED_CARD_NAME: &str = "recently-released";
const RECENTLY_ADDED_CARD_NAME: &str = "recently-added";

fn continue_watching_id() -> String {
    ids::generate_prefixed_hashid("hs", [CONTINUE_WATCHING_CARD_NAME])
}

pub fn recently_released_id() -> String {
    ids::generate_prefixed_hashid("hs", [RECENTLY_RELEASED_CARD_NAME])
}

pub fn recently_added_id() -> String {
    ids::generate_prefixed_hashid("hs", [RECENTLY_ADDED_CARD_NAME])
}

fn continue_watching_filter() -> NodeFilter {
    NodeFilter {
        order_by: Some(OrderBy::WatchProgressUpdatedAt),
        order_direction: Some(OrderDirection::Desc),
        continue_watching: Some(true),
        ..Default::default()
    }
}

pub fn recently_released_filter() -> NodeFilter {
    NodeFilter {
        kinds: Some(vec![
            crate::entities::nodes::NodeKind::Movie,
            crate::entities::nodes::NodeKind::Episode,
        ]),
        order_by: Some(OrderBy::ReleasedAt),
        order_direction: Some(OrderDirection::Desc),
        released_after: Some((Utc::now() - Duration::days(31 * 6)).timestamp().max(0)),
        ..Default::default()
    }
}

pub fn recently_added_filter() -> NodeFilter {
    NodeFilter {
        kinds: Some(vec![
            crate::entities::nodes::NodeKind::Movie,
            crate::entities::nodes::NodeKind::Series,
        ]),
        order_by: Some(OrderBy::LastAddedAt),
        order_direction: Some(OrderDirection::Desc),
        ..Default::default()
    }
}

fn synthetic_collection(
    id: String,
    name: &str,
    description: &str,
    kind: CollectionKind,
    filter: Option<NodeFilter>,
) -> collections::Model {
    collections::Model {
        id,
        name: name.to_string(),
        description: Some(description.to_string()),
        created_by_id: None,
        visibility: CollectionVisibility::Public,
        resolver_kind: CollectionResolverKind::Filter,
        kind: Some(kind.as_db()),
        filter_json: filter
            .map(|value| serde_json::to_vec(&value).expect("system filter is serializable")),
        show_on_home: true,
        home_position: 0,
        pinned: false,
        pinned_position: 0,
        created_at: 0,
        updated_at: 0,
    }
}

pub fn recently_released_collection() -> collections::Model {
    synthetic_collection(
        recently_released_id(),
        "Recently Released",
        "Recent movies and episodes from the last six months",
        CollectionKind::RecentlyReleased,
        Some(recently_released_filter()),
    )
}

pub fn recently_added_collection() -> collections::Model {
    synthetic_collection(
        recently_added_id(),
        "Recently Added",
        "Series and movies added most recently",
        CollectionKind::RecentlyAdded,
        Some(recently_added_filter()),
    )
}

pub async fn reconcile_system_collections(pool: &DatabaseConnection) -> anyhow::Result<()> {
    let continue_watching_id = continue_watching_id();
    let filter_json = serde_json::to_vec(&continue_watching_filter())?;
    let Some(existing) = collections::Entity::find()
        .filter(collections::Column::Kind.eq(CollectionKind::ContinueWatching.as_db()))
        .one(pool)
        .await?
    else {
        collections::Entity::insert(collections::ActiveModel {
            id: Set(continue_watching_id),
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

    if existing.id != continue_watching_id {
        collections::Entity::delete_by_id(existing.id.clone())
            .exec(pool)
            .await?;
        collections::Entity::insert(collections::ActiveModel {
            id: Set(continue_watching_id),
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
