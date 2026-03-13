use super::properties::{ItemNodeProperties, RootNodeProperties, SeasonNodeProperties};
use crate::auth::RequestAuth;
use crate::entities::{
    file_probe, files, item_files, item_metadata, items, root_metadata, roots, season_metadata,
    seasons, watch_progress,
};
use async_graphql::{ComplexObject, Context, Union};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
};
use std::collections::{HashMap, HashSet};

#[derive(Union)]
pub enum RootChild {
    SeasonNode(seasons::Model),
    ItemNode(items::Model),
}

#[ComplexObject]
impl roots::Model {
    pub async fn properties(
        &self,
        ctx: &Context<'_>,
    ) -> Result<RootNodeProperties, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let metadata = root_metadata::Entity::find()
            .filter(root_metadata::Column::RootId.eq(self.id.clone()))
            .order_by_desc(root_metadata::Column::Source)
            .order_by_desc(root_metadata::Column::UpdatedAt)
            .one(pool)
            .await?;

        Ok(RootNodeProperties::from_metadata(metadata))
    }

    pub async fn seasons(&self, ctx: &Context<'_>) -> Result<Vec<seasons::Model>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        seasons::Entity::find()
            .filter(seasons::Column::RootId.eq(self.id.clone()))
            .order_by_asc(seasons::Column::Order)
            .all(pool)
            .await
    }

    pub async fn files(&self, ctx: &Context<'_>) -> Result<Vec<items::Model>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        items::Entity::find()
            .filter(items::Column::RootId.eq(self.id.clone()))
            .order_by_asc(items::Column::Order)
            .all(pool)
            .await
    }

    pub async fn children(&self, ctx: &Context<'_>) -> Result<Vec<RootChild>, sea_orm::DbErr> {
        let seasons = self.seasons(ctx).await?;
        if !seasons.is_empty() {
            return Ok(seasons
                .into_iter()
                .map(RootChild::SeasonNode)
                .collect::<Vec<_>>());
        }

        let items = self.files(ctx).await?;
        Ok(items
            .into_iter()
            .map(RootChild::ItemNode)
            .collect::<Vec<_>>())
    }

    pub async fn playable_item(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<items::Model>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let user_id = current_user_id(ctx);
        let root_items = find_ordered_items_for_root(pool, &self.id).await?;
        find_playable_item_for_ordered_items(pool, root_items, user_id.as_deref()).await
    }

    pub async fn watch_progress(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<watch_progress::Model>, async_graphql::Error> {
        let auth = ctx.data::<RequestAuth>()?;
        let user = auth.get_user_or_err()?;
        let pool = ctx.data_unchecked::<DatabaseConnection>();

        let root_items = items::Entity::find()
            .filter(items::Column::RootId.eq(self.id.clone()))
            .order_by_asc(items::Column::Order)
            .all(pool)
            .await?;

        if root_items.is_empty() {
            return Ok(None);
        }

        let item_ids = root_items
            .iter()
            .map(|item| item.id.clone())
            .collect::<Vec<_>>();

        let progress_rows = watch_progress::Entity::find()
            .filter(watch_progress::Column::UserId.eq(user.id.clone()))
            .filter(watch_progress::Column::ItemId.is_in(item_ids.clone()))
            .all(pool)
            .await?;

        let progress_by_item = progress_rows
            .into_iter()
            .map(|progress| (progress.item_id.clone(), progress))
            .collect::<HashMap<_, _>>();

        Ok(select_watch_progress_for_ordered_items(
            &root_items,
            &progress_by_item,
        ))
    }

    pub async fn unplayed_items(&self, ctx: &Context<'_>) -> Result<i32, async_graphql::Error> {
        let auth = ctx.data::<RequestAuth>()?;
        let user = auth.get_user_or_err()?;
        let pool = ctx.data_unchecked::<DatabaseConnection>();

        let root_items = find_ordered_items_for_root(pool, &self.id).await?;
        count_unplayed_items_for_ordered_items(pool, root_items, &user.id).await
    }

    pub async fn season_count(&self, ctx: &Context<'_>) -> Result<i32, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let count = seasons::Entity::find()
            .filter(seasons::Column::RootId.eq(self.id.clone()))
            .count(pool)
            .await?;
        Ok(saturating_i32_from_u64(count))
    }

    pub async fn episode_count(&self, ctx: &Context<'_>) -> Result<i32, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let count = items::Entity::find()
            .filter(items::Column::RootId.eq(self.id.clone()))
            .filter(items::Column::SeasonId.is_null())
            .count(pool)
            .await?;
        Ok(saturating_i32_from_u64(count))
    }
}

#[ComplexObject]
impl seasons::Model {
    pub async fn properties(
        &self,
        ctx: &Context<'_>,
    ) -> Result<SeasonNodeProperties, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let metadata = season_metadata::Entity::find()
            .filter(season_metadata::Column::SeasonId.eq(self.id.clone()))
            .order_by_desc(season_metadata::Column::Source)
            .order_by_desc(season_metadata::Column::UpdatedAt)
            .one(pool)
            .await?;

        Ok(SeasonNodeProperties::from_metadata(
            metadata,
            Some(self.season_number),
        ))
    }

    pub async fn files(&self, ctx: &Context<'_>) -> Result<Vec<items::Model>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        items::Entity::find()
            .filter(items::Column::SeasonId.eq(self.id.clone()))
            .order_by_asc(items::Column::Order)
            .all(pool)
            .await
    }

    pub async fn playable_item(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<items::Model>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let user_id = current_user_id(ctx);
        let season_items = find_ordered_items_for_season(pool, &self.id).await?;
        find_playable_item_for_ordered_items(pool, season_items, user_id.as_deref()).await
    }

    pub async fn watch_progress(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<watch_progress::Model>, async_graphql::Error> {
        let auth = ctx.data::<RequestAuth>()?;
        let user = auth.get_user_or_err()?;
        let pool = ctx.data_unchecked::<DatabaseConnection>();

        let season_items = self.files(ctx).await?;
        if season_items.is_empty() {
            return Ok(None);
        }

        let item_ids = season_items
            .iter()
            .map(|item| item.id.clone())
            .collect::<Vec<_>>();

        let progress_rows = watch_progress::Entity::find()
            .filter(watch_progress::Column::UserId.eq(user.id.clone()))
            .filter(watch_progress::Column::ItemId.is_in(item_ids.clone()))
            .all(pool)
            .await?;

        let progress_by_item = progress_rows
            .into_iter()
            .map(|progress| (progress.item_id.clone(), progress))
            .collect::<HashMap<_, _>>();

        Ok(select_watch_progress_for_ordered_items(
            &season_items,
            &progress_by_item,
        ))
    }

    pub async fn unplayed_items(&self, ctx: &Context<'_>) -> Result<i32, async_graphql::Error> {
        let auth = ctx.data::<RequestAuth>()?;
        let user = auth.get_user_or_err()?;
        let pool = ctx.data_unchecked::<DatabaseConnection>();

        let season_items = find_ordered_items_for_season(pool, &self.id).await?;
        count_unplayed_items_for_ordered_items(pool, season_items, &user.id).await
    }

    pub async fn episode_count(&self, ctx: &Context<'_>) -> Result<i32, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let count = items::Entity::find()
            .filter(items::Column::SeasonId.eq(self.id.clone()))
            .count(pool)
            .await?;
        Ok(saturating_i32_from_u64(count))
    }
}

#[ComplexObject]
impl items::Model {
    pub async fn properties(
        &self,
        ctx: &Context<'_>,
    ) -> Result<ItemNodeProperties, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();

        let metadata = item_metadata::Entity::find()
            .filter(item_metadata::Column::ItemId.eq(self.id.clone()))
            .order_by_desc(item_metadata::Column::Source)
            .order_by_desc(item_metadata::Column::UpdatedAt)
            .one(pool)
            .await?;

        let season_number = if let Some(season_id) = &self.season_id {
            seasons::Entity::find_by_id(season_id.clone())
                .one(pool)
                .await?
                .map(|season| season.season_number)
        } else {
            None
        };
        let default_file = find_default_file_for_item(pool, self).await?;
        let probe = if let Some(file) = default_file.as_ref() {
            file_probe::Entity::find_by_id(file.id).one(pool).await?
        } else {
            None
        };

        Ok(ItemNodeProperties::from_metadata(
            metadata,
            self.id.clone(),
            season_number,
            self.episode_number,
            default_file,
            probe,
        ))
    }

    pub async fn watch_progress(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<watch_progress::Model>, async_graphql::Error> {
        let auth = ctx.data::<RequestAuth>()?;
        let user = auth.get_user_or_err()?;
        let pool = ctx.data_unchecked::<DatabaseConnection>();

        let progress = watch_progress::Entity::find()
            .filter(watch_progress::Column::UserId.eq(user.id.clone()))
            .filter(watch_progress::Column::ItemId.eq(self.id.clone()))
            .one(pool)
            .await?;

        Ok(progress)
    }

    pub async fn file(&self, ctx: &Context<'_>) -> Result<Option<files::Model>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        find_default_file_for_item(pool, self).await
    }

    pub async fn parent(&self, ctx: &Context<'_>) -> Result<Option<roots::Model>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        roots::Entity::find_by_id(self.root_id.clone())
            .one(pool)
            .await
    }

    pub async fn previous_item(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<items::Model>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        find_adjacent_item_without_shared_files(pool, self, ItemDirection::Previous).await
    }

    pub async fn next_item(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<items::Model>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        find_adjacent_item_without_shared_files(pool, self, ItemDirection::Next).await
    }
}

fn current_user_id(ctx: &Context<'_>) -> Option<String> {
    let auth = ctx.data_opt::<RequestAuth>()?;
    let user = auth.get_user_or_err().ok()?;
    Some(user.id.clone())
}

async fn find_ordered_items_for_root(
    pool: &DatabaseConnection,
    root_id: &str,
) -> Result<Vec<items::Model>, sea_orm::DbErr> {
    items::Entity::find()
        .filter(items::Column::RootId.eq(root_id.to_string()))
        .order_by_asc(items::Column::Order)
        .order_by_asc(items::Column::Id)
        .all(pool)
        .await
}

async fn find_ordered_items_for_season(
    pool: &DatabaseConnection,
    season_id: &str,
) -> Result<Vec<items::Model>, sea_orm::DbErr> {
    items::Entity::find()
        .filter(items::Column::SeasonId.eq(season_id.to_string()))
        .order_by_asc(items::Column::Order)
        .order_by_asc(items::Column::Id)
        .all(pool)
        .await
}

async fn find_playable_item_for_ordered_items(
    pool: &DatabaseConnection,
    ordered_items: Vec<items::Model>,
    user_id: Option<&str>,
) -> Result<Option<items::Model>, sea_orm::DbErr> {
    if ordered_items.is_empty() {
        return Ok(None);
    }

    if let Some(user_id) = user_id {
        let item_ids = ordered_items
            .iter()
            .map(|item| item.id.clone())
            .collect::<Vec<_>>();

        let progress_rows = watch_progress::Entity::find()
            .filter(watch_progress::Column::UserId.eq(user_id))
            .filter(watch_progress::Column::ItemId.is_in(item_ids))
            .all(pool)
            .await?;

        let progress_by_item = progress_rows
            .into_iter()
            .map(|progress| (progress.item_id.clone(), progress))
            .collect::<HashMap<_, _>>();

        // Prefer resuming an actively in-progress item before falling back to
        // deterministic next-up selection.
        let mut resume_candidate: Option<(i64, usize, items::Model)> = None;
        for (index, item) in ordered_items.iter().enumerate() {
            let Some(row) = progress_by_item.get(&item.id) else {
                continue;
            };

            let is_in_progress = watch_progress::is_in_progress(row.progress_percent);
            if !is_in_progress {
                continue;
            }

            match &resume_candidate {
                Some((best_updated_at, best_index, _))
                    if row.updated_at < *best_updated_at
                        || (row.updated_at == *best_updated_at && index >= *best_index) => {}
                _ => {
                    resume_candidate = Some((row.updated_at, index, item.clone()));
                }
            }
        }

        if let Some((_, _, item)) = resume_candidate {
            return Ok(Some(item));
        }

        for item in &ordered_items {
            let row = progress_by_item.get(&item.id);
            if row.is_none() || row.is_some_and(|entry| entry.progress_percent <= 0.0) {
                return Ok(Some(item.clone()));
            }
        }
    }

    Ok(ordered_items.into_iter().next())
}

fn select_watch_progress_for_ordered_items(
    ordered_items: &[items::Model],
    progress_by_item: &HashMap<String, watch_progress::Model>,
) -> Option<watch_progress::Model> {
    let mut resume_candidate: Option<(i64, usize, watch_progress::Model)> = None;
    for (index, item) in ordered_items.iter().enumerate() {
        let Some(progress) = progress_by_item.get(&item.id) else {
            continue;
        };

        let is_in_progress = watch_progress::is_in_progress(progress.progress_percent);
        if !is_in_progress {
            continue;
        }

        match &resume_candidate {
            Some((best_updated_at, best_index, _))
                if progress.updated_at < *best_updated_at
                    || (progress.updated_at == *best_updated_at && index >= *best_index) => {}
            _ => {
                resume_candidate = Some((progress.updated_at, index, progress.clone()));
            }
        }
    }

    if let Some((_, _, progress)) = resume_candidate {
        return Some(progress);
    }

    for item in ordered_items {
        if let Some(progress) = progress_by_item.get(&item.id) {
            return Some(progress.clone());
        }
    }

    None
}

async fn count_unplayed_items_for_ordered_items(
    pool: &DatabaseConnection,
    ordered_items: Vec<items::Model>,
    user_id: &str,
) -> Result<i32, async_graphql::Error> {
    if ordered_items.is_empty() {
        return Ok(0);
    }

    let item_ids = ordered_items
        .iter()
        .map(|item| item.id.clone())
        .collect::<Vec<_>>();

    let watched_count = watch_progress::Entity::find()
        .filter(watch_progress::Column::UserId.eq(user_id))
        .filter(watch_progress::Column::ItemId.is_in(item_ids))
        .all(pool)
        .await?
        .into_iter()
        .filter(|progress| watch_progress::is_completed_progress(progress.progress_percent))
        .count();

    let unplayed_count = ordered_items.len().saturating_sub(watched_count);
    Ok(i32::try_from(unplayed_count).unwrap_or(i32::MAX))
}

fn saturating_i32_from_u64(value: u64) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

#[derive(Clone, Copy)]
enum ItemDirection {
    Previous,
    Next,
}

async fn find_adjacent_item_without_shared_files(
    pool: &DatabaseConnection,
    item: &items::Model,
    direction: ItemDirection,
) -> Result<Option<items::Model>, sea_orm::DbErr> {
    let ordered_items = find_ordered_items_for_root(pool, &item.root_id).await?;
    if ordered_items.is_empty() {
        return Ok(None);
    }

    let Some(current_index) = ordered_items
        .iter()
        .position(|candidate| candidate.id == item.id)
    else {
        return Ok(None);
    };

    let item_ids = ordered_items
        .iter()
        .map(|candidate| candidate.id.clone())
        .collect::<Vec<_>>();

    let links = item_files::Entity::find()
        .filter(item_files::Column::ItemId.is_in(item_ids))
        .all(pool)
        .await?;

    let mut file_ids_by_item: HashMap<String, HashSet<i64>> = HashMap::new();
    for link in links {
        file_ids_by_item
            .entry(link.item_id)
            .or_default()
            .insert(link.file_id);
    }

    let current_item_file_ids = file_ids_by_item.get(&item.id).cloned().unwrap_or_default();

    let candidate = match direction {
        ItemDirection::Previous => ordered_items[..current_index]
            .iter()
            .rev()
            .find(|candidate| {
                is_navigation_candidate(candidate, &file_ids_by_item, &current_item_file_ids)
            }),
        ItemDirection::Next => ordered_items
            .iter()
            .skip(current_index + 1)
            .find(|candidate| {
                is_navigation_candidate(candidate, &file_ids_by_item, &current_item_file_ids)
            }),
    };

    Ok(candidate.cloned())
}

fn is_navigation_candidate(
    candidate: &items::Model,
    file_ids_by_item: &HashMap<String, HashSet<i64>>,
    current_item_file_ids: &HashSet<i64>,
) -> bool {
    let Some(candidate_file_ids) = file_ids_by_item.get(&candidate.id) else {
        return false;
    };

    if candidate_file_ids.is_empty() {
        return false;
    }

    !candidate_file_ids
        .iter()
        .any(|file_id| current_item_file_ids.contains(file_id))
}

async fn find_default_file_for_item(
    pool: &DatabaseConnection,
    item: &items::Model,
) -> Result<Option<files::Model>, sea_orm::DbErr> {
    if let Some(primary_file_id) = item.primary_file_id {
        let primary = files::Entity::find_by_id(primary_file_id)
            .filter(files::Column::UnavailableAt.is_null())
            .one(pool)
            .await?;

        if primary.is_some() {
            return Ok(primary);
        }
    }

    let links = item_files::Entity::find()
        .filter(item_files::Column::ItemId.eq(item.id.clone()))
        .order_by_asc(item_files::Column::Order)
        .order_by_asc(item_files::Column::FileId)
        .all(pool)
        .await?;

    for link in links {
        let file = files::Entity::find_by_id(link.file_id)
            .filter(files::Column::UnavailableAt.is_null())
            .one(pool)
            .await?;
        if file.is_some() {
            return Ok(file);
        }
    }

    Ok(None)
}
