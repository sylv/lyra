use super::properties::{ItemNodeProperties, RootNodeProperties, SeasonNodeProperties};
use crate::auth::RequestAuth;
use crate::entities::{
    files, item_files, item_metadata, items, root_metadata, roots, season_metadata, seasons,
    watch_progress,
};
use async_graphql::{ComplexObject, Context, Union};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
};
use std::collections::HashMap;

const PLAYABLE_PROGRESS_THRESHOLD: f32 = 0.8;

#[derive(Union)]
pub enum RootChild {
    SeasonNode(seasons::Model),
    ItemNode(items::Model),
}

#[ComplexObject]
impl roots::Model {
    pub async fn properties(&self, ctx: &Context<'_>) -> Result<RootNodeProperties, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let metadata = root_metadata::Entity::find()
            .filter(root_metadata::Column::RootId.eq(self.id.clone()))
            .filter(root_metadata::Column::IsPrimary.eq(true))
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
        Ok(items.into_iter().map(RootChild::ItemNode).collect::<Vec<_>>())
    }

    #[graphql(name = "playable_item")]
    pub async fn playable_item(&self, ctx: &Context<'_>) -> Result<Option<items::Model>, sea_orm::DbErr> {
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

        for item in root_items {
            if let Some(progress) = progress_by_item.get(&item.id) {
                return Ok(Some(progress.clone()));
            }
        }

        Ok(None)
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
            .filter(season_metadata::Column::IsPrimary.eq(true))
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

    #[graphql(name = "playable_item")]
    pub async fn playable_item(&self, ctx: &Context<'_>) -> Result<Option<items::Model>, sea_orm::DbErr> {
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

        for item in season_items {
            if let Some(progress) = progress_by_item.get(&item.id) {
                return Ok(Some(progress.clone()));
            }
        }

        Ok(None)
    }
}

#[ComplexObject]
impl items::Model {
    pub async fn properties(&self, ctx: &Context<'_>) -> Result<ItemNodeProperties, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();

        let metadata = item_metadata::Entity::find()
            .filter(item_metadata::Column::ItemId.eq(self.id.clone()))
            .filter(item_metadata::Column::IsPrimary.eq(true))
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

        Ok(ItemNodeProperties::from_metadata(
            metadata,
            self.id.clone(),
            season_number,
            self.episode_number,
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
        roots::Entity::find_by_id(self.root_id.clone()).one(pool).await
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

        for item in &ordered_items {
            let should_pick = match progress_by_item.get(&item.id) {
                Some(row) => row.progress_percent < PLAYABLE_PROGRESS_THRESHOLD,
                None => true,
            };

            if should_pick {
                return Ok(Some(item.clone()));
            }
        }
    }

    Ok(ordered_items.into_iter().next())
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
