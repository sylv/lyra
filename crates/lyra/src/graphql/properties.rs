use crate::entities::{
    assets,
    file_assets::{self, FileAssetRole},
    file_probe, files, item_files, item_metadata, root_metadata, season_metadata,
};
use crate::segment_markers::StoredFileSegmentKind;
use async_graphql::{ComplexObject, Context, Enum, SimpleObject};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
    RelationTrait,
};
use std::collections::HashMap;

#[derive(Clone, Debug, SimpleObject)]
pub struct Asset {
    pub id: i64,
    pub source_url: Option<String>,
    pub hash_sha256: Option<String>,
    pub size_bytes: Option<i64>,
    pub mime_type: Option<String>,
    pub height: Option<i64>,
    pub width: Option<i64>,
    pub thumbhash: Option<String>,
    pub created_at: i64,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct TimelinePreviewSheet {
    pub position_ms: i64,
    pub end_ms: i64,
    pub sheet_interval_ms: i64,
    pub sheet_gap_size: i64,
    pub asset: Asset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Enum)]
pub enum FileSegmentKind {
    Intro,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct FileSegment {
    pub kind: FileSegmentKind,
    pub start_ms: i64,
    pub end_ms: i64,
}

#[derive(Clone, Debug, SimpleObject)]
#[graphql(complex)]
pub struct RootNodeProperties {
    pub description: Option<String>,
    pub rating: Option<f64>,
    pub runtime_minutes: Option<i64>,
    pub released_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    #[graphql(skip)]
    pub background_asset_id: Option<i64>,
    #[graphql(skip)]
    pub poster_asset_id: Option<i64>,
    #[graphql(skip)]
    pub thumbnail_asset_id: Option<i64>,
}

#[derive(Clone, Debug, SimpleObject)]
#[graphql(complex)]
pub struct SeasonNodeProperties {
    pub description: Option<String>,
    pub rating: Option<f64>,
    pub season_number: Option<i64>,
    pub runtime_minutes: Option<i64>,
    pub released_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    #[graphql(skip)]
    pub background_asset_id: Option<i64>,
    #[graphql(skip)]
    pub poster_asset_id: Option<i64>,
    #[graphql(skip)]
    pub thumbnail_asset_id: Option<i64>,
}

#[derive(Clone, Debug, SimpleObject)]
#[graphql(complex)]
pub struct ItemNodeProperties {
    pub description: Option<String>,
    pub rating: Option<f64>,
    pub season_number: Option<i64>,
    pub episode_number: Option<i64>,
    pub runtime_minutes: Option<i64>,
    pub duration_seconds: Option<i64>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub fps: Option<f64>,
    pub video_bitrate: Option<i64>,
    pub audio_bitrate: Option<i64>,
    pub audio_channels: Option<i64>,
    pub has_subtitles: Option<bool>,
    pub file_size_bytes: Option<i64>,
    pub released_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    #[graphql(skip)]
    pub background_asset_id: Option<i64>,
    #[graphql(skip)]
    pub poster_asset_id: Option<i64>,
    #[graphql(skip)]
    pub thumbnail_asset_id: Option<i64>,
    #[graphql(skip)]
    pub item_id: String,
}

#[ComplexObject]
impl RootNodeProperties {
    pub async fn background_image(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<Asset>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        find_asset(pool, self.background_asset_id).await
    }

    pub async fn poster_image(&self, ctx: &Context<'_>) -> Result<Option<Asset>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        find_asset(pool, self.poster_asset_id.or(self.thumbnail_asset_id)).await
    }

    pub async fn thumbnail_image(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<Asset>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        find_asset(pool, self.thumbnail_asset_id).await
    }
}

#[ComplexObject]
impl SeasonNodeProperties {
    pub async fn background_image(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<Asset>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        find_asset(pool, self.background_asset_id).await
    }

    pub async fn poster_image(&self, ctx: &Context<'_>) -> Result<Option<Asset>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        find_asset(pool, self.poster_asset_id.or(self.thumbnail_asset_id)).await
    }

    pub async fn thumbnail_image(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<Asset>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        find_asset(pool, self.thumbnail_asset_id).await
    }
}

#[ComplexObject]
impl ItemNodeProperties {
    pub async fn background_image(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<Asset>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        find_asset(pool, self.background_asset_id).await
    }

    pub async fn poster_image(&self, ctx: &Context<'_>) -> Result<Option<Asset>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        if let Some(asset_id) = self.poster_asset_id.or(self.thumbnail_asset_id) {
            return find_asset(pool, Some(asset_id)).await;
        }

        let asset_id = self.file_thumbnail_asset_id(pool).await?;
        find_asset(pool, asset_id).await
    }

    pub async fn thumbnail_image(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<Asset>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        if let Some(asset_id) = self.thumbnail_asset_id {
            return find_asset(pool, Some(asset_id)).await;
        }

        let asset_id = self.file_thumbnail_asset_id(pool).await?;
        find_asset(pool, asset_id).await
    }
}

#[ComplexObject]
impl files::Model {
    pub async fn timeline_preview(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<TimelinePreviewSheet>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();

        let rows = file_assets::Entity::find()
            .filter(file_assets::Column::FileId.eq(self.id))
            .filter(file_assets::Column::Role.eq(FileAssetRole::TimelinePreviewSheet))
            .order_by_asc(file_assets::Column::PositionMs)
            .order_by_asc(file_assets::Column::AssetId)
            .all(pool)
            .await?;

        if rows.is_empty() {
            return Ok(Vec::new());
        }

        let asset_ids = rows.iter().map(|row| row.asset_id).collect::<Vec<_>>();
        let asset_models = assets::Entity::find()
            .filter(assets::Column::Id.is_in(asset_ids))
            .all(pool)
            .await?;
        let assets_by_id = asset_models
            .into_iter()
            .map(|asset| (asset.id, asset))
            .collect::<HashMap<_, _>>();

        let mut sheets = Vec::new();
        for row in rows {
            let Some(position_ms) = row.position_ms else {
                continue;
            };
            let Some(end_ms) = row.end_ms else {
                continue;
            };
            let Some(sheet_interval_ms) = row.sheet_interval else {
                continue;
            };
            let Some(sheet_gap_size) = row.sheet_gap_size else {
                continue;
            };
            if end_ms <= position_ms || sheet_interval_ms <= 0 || sheet_gap_size < 0 {
                continue;
            }

            let Some(asset) = assets_by_id.get(&row.asset_id) else {
                continue;
            };

            sheets.push(TimelinePreviewSheet {
                position_ms,
                end_ms,
                sheet_interval_ms,
                sheet_gap_size,
                asset: Asset::from_model(asset.clone()),
            });
        }

        Ok(sheets)
    }

    pub async fn segments(&self, _ctx: &Context<'_>) -> Result<Vec<FileSegment>, sea_orm::DbErr> {
        if self.segments_json.is_empty() {
            return Ok(Vec::new());
        }

        let decoded = match self.decode_segments() {
            Ok(segments) => segments,
            Err(error) => {
                tracing::warn!(
                    file_id = self.id,
                    error = ?error,
                    "failed to decode file segments"
                );
                return Ok(Vec::new());
            }
        };

        Ok(decoded
            .into_iter()
            .filter_map(|segment| {
                if segment.end_ms <= segment.start_ms {
                    return None;
                }

                let kind = match segment.kind {
                    StoredFileSegmentKind::Intro => FileSegmentKind::Intro,
                };

                Some(FileSegment {
                    kind,
                    start_ms: segment.start_ms,
                    end_ms: segment.end_ms,
                })
            })
            .collect())
    }
}

impl RootNodeProperties {
    pub(crate) fn from_metadata(metadata: Option<root_metadata::Model>) -> Self {
        let Some(metadata) = metadata else {
            return Self {
                description: None,
                rating: None,
                runtime_minutes: None,
                released_at: None,
                ended_at: None,
                created_at: None,
                updated_at: None,
                background_asset_id: None,
                poster_asset_id: None,
                thumbnail_asset_id: None,
            };
        };

        Self {
            description: metadata.description,
            rating: metadata.score_normalized.map(|score| score as f64 / 10.0),
            runtime_minutes: None,
            released_at: metadata.released_at,
            ended_at: metadata.ended_at,
            created_at: Some(metadata.created_at),
            updated_at: Some(metadata.updated_at),
            background_asset_id: metadata.background_asset_id,
            poster_asset_id: metadata.poster_asset_id,
            thumbnail_asset_id: metadata.thumbnail_asset_id,
        }
    }
}

impl SeasonNodeProperties {
    pub(crate) fn from_metadata(
        metadata: Option<season_metadata::Model>,
        season_number: Option<i64>,
    ) -> Self {
        let Some(metadata) = metadata else {
            return Self {
                description: None,
                rating: None,
                season_number,
                runtime_minutes: None,
                released_at: None,
                ended_at: None,
                created_at: None,
                updated_at: None,
                background_asset_id: None,
                poster_asset_id: None,
                thumbnail_asset_id: None,
            };
        };

        Self {
            description: metadata.description,
            rating: metadata.score_normalized.map(|score| score as f64 / 10.0),
            season_number,
            runtime_minutes: None,
            released_at: metadata.released_at,
            ended_at: metadata.ended_at,
            created_at: Some(metadata.created_at),
            updated_at: Some(metadata.updated_at),
            background_asset_id: metadata.background_asset_id,
            poster_asset_id: metadata.poster_asset_id,
            thumbnail_asset_id: metadata.thumbnail_asset_id,
        }
    }
}

impl ItemNodeProperties {
    pub(crate) fn from_metadata(
        metadata: Option<item_metadata::Model>,
        item_id: String,
        season_number: Option<i64>,
        episode_number: Option<i64>,
        default_file: Option<files::Model>,
        probe: Option<file_probe::Model>,
    ) -> Self {
        let duration_seconds = probe
            .as_ref()
            .and_then(|probe| probe.duration_s)
            .filter(|seconds| *seconds > 0);
        let runtime_from_probe = duration_seconds.map(minutes_from_seconds_ceil);
        let runtime_from_metadata: Option<i64> = None;

        match metadata {
            Some(metadata) => Self {
                description: metadata.description,
                rating: metadata.score_normalized.map(|score| score as f64 / 10.0),
                season_number,
                episode_number,
                runtime_minutes: runtime_from_probe.or(runtime_from_metadata),
                duration_seconds,
                width: probe
                    .as_ref()
                    .and_then(|probe| probe.width)
                    .or(default_file.as_ref().and_then(|file| file.width)),
                height: probe
                    .as_ref()
                    .and_then(|probe| probe.height)
                    .or(default_file.as_ref().and_then(|file| file.height)),
                video_codec: probe.as_ref().and_then(|probe| probe.video_codec.clone()),
                audio_codec: probe.as_ref().and_then(|probe| probe.audio_codec.clone()),
                fps: probe.as_ref().and_then(|probe| probe.fps),
                video_bitrate: probe.as_ref().and_then(|probe| probe.video_bitrate),
                audio_bitrate: probe.as_ref().and_then(|probe| probe.audio_bitrate),
                audio_channels: probe.as_ref().and_then(|probe| probe.audio_channels),
                has_subtitles: probe.as_ref().map(|probe| probe.has_subtitles),
                file_size_bytes: default_file.as_ref().map(|file| file.size_bytes),
                released_at: metadata.released_at,
                ended_at: metadata.ended_at,
                created_at: Some(metadata.created_at),
                updated_at: Some(metadata.updated_at),
                background_asset_id: metadata.background_asset_id,
                poster_asset_id: metadata.poster_asset_id,
                thumbnail_asset_id: metadata.thumbnail_asset_id,
                item_id,
            },
            None => Self {
                description: None,
                rating: None,
                season_number,
                episode_number,
                runtime_minutes: runtime_from_probe.or(runtime_from_metadata),
                duration_seconds,
                width: probe
                    .as_ref()
                    .and_then(|probe| probe.width)
                    .or(default_file.as_ref().and_then(|file| file.width)),
                height: probe
                    .as_ref()
                    .and_then(|probe| probe.height)
                    .or(default_file.as_ref().and_then(|file| file.height)),
                video_codec: probe.as_ref().and_then(|probe| probe.video_codec.clone()),
                audio_codec: probe.as_ref().and_then(|probe| probe.audio_codec.clone()),
                fps: probe.as_ref().and_then(|probe| probe.fps),
                video_bitrate: probe.as_ref().and_then(|probe| probe.video_bitrate),
                audio_bitrate: probe.as_ref().and_then(|probe| probe.audio_bitrate),
                audio_channels: probe.as_ref().and_then(|probe| probe.audio_channels),
                has_subtitles: probe.as_ref().map(|probe| probe.has_subtitles),
                file_size_bytes: default_file.as_ref().map(|file| file.size_bytes),
                released_at: None,
                ended_at: None,
                created_at: None,
                updated_at: None,
                background_asset_id: None,
                poster_asset_id: None,
                thumbnail_asset_id: None,
                item_id,
            },
        }
    }

    async fn file_thumbnail_asset_id(
        &self,
        pool: &DatabaseConnection,
    ) -> Result<Option<i64>, sea_orm::DbErr> {
        let links = item_files::Entity::find()
            .join(
                sea_orm::JoinType::InnerJoin,
                item_files::Relation::Files.def(),
            )
            .filter(item_files::Column::ItemId.eq(self.item_id.clone()))
            .filter(files::Column::UnavailableAt.is_null())
            .order_by_asc(item_files::Column::Order)
            .order_by_asc(item_files::Column::FileId)
            .all(pool)
            .await?;

        for link in links {
            let thumbnail = file_assets::Entity::find()
                .filter(file_assets::Column::FileId.eq(link.file_id))
                .filter(file_assets::Column::Role.eq(FileAssetRole::Thumbnail))
                .order_by_desc(file_assets::Column::AssetId)
                .one(pool)
                .await?;

            if let Some(thumbnail) = thumbnail {
                return Ok(Some(thumbnail.asset_id));
            }
        }

        Ok(None)
    }
}

fn minutes_from_seconds_ceil(seconds: i64) -> i64 {
    (seconds + 59) / 60
}

impl Asset {
    pub(crate) fn from_model(model: assets::Model) -> Self {
        Self {
            id: model.id,
            source_url: model.source_url,
            hash_sha256: model.hash_sha256,
            size_bytes: model.size_bytes,
            mime_type: model.mime_type,
            height: model.height,
            width: model.width,
            thumbhash: model.thumbhash.map(hex::encode),
            created_at: model.created_at,
        }
    }
}

async fn find_asset(
    pool: &DatabaseConnection,
    asset_id: Option<i64>,
) -> Result<Option<Asset>, sea_orm::DbErr> {
    let Some(asset_id) = asset_id else {
        return Ok(None);
    };

    let model = assets::Entity::find_by_id(asset_id).one(pool).await?;
    Ok(model.map(Asset::from_model))
}
