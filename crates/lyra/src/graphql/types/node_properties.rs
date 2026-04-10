use crate::entities::{
    assets,
    file_assets::{self, FileAssetRole},
    file_probe, files, node_files, nodes,
};
use crate::graphql::dataloaders::{
    node_counts::NodeCountsLoader,
    node_metadata::{NodeMetadataLoader, PreferredNodeMetadata},
};
use crate::graphql::properties::{Asset, NodeProperties};
use async_graphql::dataloader::DataLoader;
use async_graphql::{ComplexObject, Context};
use chrono::{DateTime, Datelike, Utc};
use lyra_probe::ProbeData;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
    RelationTrait,
};

#[ComplexObject]
impl NodeProperties {
    pub async fn background_image(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<Asset>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        find_asset(pool, self.background_asset_id.clone()).await
    }

    pub async fn poster_image(&self, ctx: &Context<'_>) -> Result<Option<Asset>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        if self.kind != nodes::NodeKind::Episode
            && let Some(asset_id) = self.poster_asset_id.clone()
        {
            return find_asset(pool, Some(asset_id)).await;
        }

        if let Some(asset_id) = self.poster_fallback_asset_id(pool).await? {
            return find_asset(pool, Some(asset_id)).await;
        }

        if let Some(asset_id) = self.thumbnail_asset_id.clone() {
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
        if let Some(asset_id) = self.thumbnail_asset_id.clone() {
            return find_asset(pool, Some(asset_id)).await;
        }

        let asset_id = self.file_thumbnail_asset_id(pool).await?;
        find_asset(pool, asset_id).await
    }

    pub async fn display_detail(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<String>, sea_orm::DbErr> {
        match self.kind {
            nodes::NodeKind::Series => {
                let loader = ctx.data_unchecked::<DataLoader<NodeCountsLoader>>();
                let counts = loader
                    .load_one(self.node_id.clone())
                    .await
                    .map_err(sea_orm::DbErr::Custom)?
                    .unwrap_or_default();
                Ok(format_count_detail(counts.season_count, "season"))
            }
            nodes::NodeKind::Season => {
                let loader = ctx.data_unchecked::<DataLoader<NodeCountsLoader>>();
                let counts = loader
                    .load_one(self.node_id.clone())
                    .await
                    .map_err(sea_orm::DbErr::Custom)?
                    .unwrap_or_default();
                Ok(format_count_detail(counts.episode_count, "episode"))
            }
            nodes::NodeKind::Episode => {
                let loader = ctx.data_unchecked::<DataLoader<NodeMetadataLoader>>();
                Ok(loader
                    .load_one(self.root_id.clone())
                    .await
                    .map_err(sea_orm::DbErr::Custom)?
                    .map(|metadata| metadata.display_name().to_owned()))
            }
            nodes::NodeKind::Movie => Ok(self.release_year()),
        }
    }
}

impl NodeProperties {
    pub async fn from_node(
        pool: &DatabaseConnection,
        node: &nodes::Model,
        metadata: Option<PreferredNodeMetadata>,
    ) -> Result<Self, sea_orm::DbErr> {
        let default_file = Self::primary_file_for_node(pool, &node.id).await?;
        let probe = if let Some(file) = &default_file {
            file_probe::Entity::find_by_id(file.id.clone())
                .one(pool)
                .await?
        } else {
            None
        };
        let probe_data = probe.as_ref().and_then(|probe| probe.get_probe().ok());
        let probe_summary = probe_data.as_ref().map(summarize_probe);

        let duration_seconds = probe_summary
            .as_ref()
            .and_then(|probe| probe.duration_seconds)
            .filter(|seconds| *seconds > 0);

        let runtime_minutes = duration_seconds.map(minutes_from_seconds_ceil);
        let metadata = metadata.and_then(|metadata| metadata.metadata);
        let display_name = metadata
            .as_ref()
            .map(|metadata| metadata.name.clone())
            .unwrap_or_else(|| node.name.clone());

        Ok(match metadata {
            Some(metadata) => Self {
                display_name: display_name.clone(),
                description: metadata.description,
                rating: metadata.score_normalized.map(|score| score as f64 / 10.0),
                season_number: node.season_number,
                episode_number: node.episode_number,
                runtime_minutes,
                duration_seconds,
                width: probe_summary
                    .as_ref()
                    .and_then(|probe| probe.width)
                    .or(default_file.as_ref().and_then(|file| file.width)),
                height: probe_summary
                    .as_ref()
                    .and_then(|probe| probe.height)
                    .or(default_file.as_ref().and_then(|file| file.height)),
                video_codec: probe_summary
                    .as_ref()
                    .and_then(|probe| probe.video_codec.clone()),
                audio_codec: probe_summary
                    .as_ref()
                    .and_then(|probe| probe.audio_codec.clone()),
                fps: probe_summary.as_ref().and_then(|probe| probe.fps),
                video_bitrate: probe_summary.as_ref().and_then(|probe| probe.video_bitrate),
                audio_bitrate: probe_summary.as_ref().and_then(|probe| probe.audio_bitrate),
                audio_channels: probe_summary
                    .as_ref()
                    .and_then(|probe| probe.audio_channels),
                has_subtitles: probe_summary.as_ref().map(|probe| probe.has_subtitles),
                file_size_bytes: default_file.as_ref().map(|file| file.size_bytes),
                first_aired: metadata.first_aired,
                last_aired: metadata.last_aired,
                created_at: Some(metadata.created_at),
                updated_at: Some(metadata.updated_at),
                background_asset_id: metadata.background_asset_id,
                poster_asset_id: metadata.poster_asset_id,
                thumbnail_asset_id: metadata.thumbnail_asset_id,
                node_id: node.id.clone(),
                root_id: node.root_id.clone(),
                parent_id: node.parent_id.clone(),
                kind: node.kind,
            },
            None => Self {
                display_name,
                description: None,
                rating: None,
                season_number: node.season_number,
                episode_number: node.episode_number,
                runtime_minutes,
                duration_seconds,
                width: probe_summary
                    .as_ref()
                    .and_then(|probe| probe.width)
                    .or(default_file.as_ref().and_then(|file| file.width)),
                height: probe_summary
                    .as_ref()
                    .and_then(|probe| probe.height)
                    .or(default_file.as_ref().and_then(|file| file.height)),
                video_codec: probe_summary
                    .as_ref()
                    .and_then(|probe| probe.video_codec.clone()),
                audio_codec: probe_summary
                    .as_ref()
                    .and_then(|probe| probe.audio_codec.clone()),
                fps: probe_summary.as_ref().and_then(|probe| probe.fps),
                video_bitrate: probe_summary.as_ref().and_then(|probe| probe.video_bitrate),
                audio_bitrate: probe_summary.as_ref().and_then(|probe| probe.audio_bitrate),
                audio_channels: probe_summary
                    .as_ref()
                    .and_then(|probe| probe.audio_channels),
                has_subtitles: probe_summary.as_ref().map(|probe| probe.has_subtitles),
                file_size_bytes: default_file.as_ref().map(|file| file.size_bytes),
                first_aired: None,
                last_aired: None,
                created_at: None,
                updated_at: None,
                background_asset_id: None,
                poster_asset_id: None,
                thumbnail_asset_id: None,
                node_id: node.id.clone(),
                root_id: node.root_id.clone(),
                parent_id: node.parent_id.clone(),
                kind: node.kind,
            },
        })
    }

    pub async fn primary_file_for_node(
        pool: &DatabaseConnection,
        node_id: &str,
    ) -> Result<Option<files::Model>, sea_orm::DbErr> {
        node_files::Entity::find()
            .join(
                sea_orm::JoinType::InnerJoin,
                node_files::Relation::Files.def(),
            )
            .filter(node_files::Column::NodeId.eq(node_id))
            .filter(files::Column::UnavailableAt.is_null())
            .order_by_asc(node_files::Column::Order)
            .order_by_asc(node_files::Column::FileId)
            .select_only()
            .column_as(files::Column::Id, "id")
            .column_as(files::Column::LibraryId, "library_id")
            .column_as(files::Column::RelativePath, "relative_path")
            .column_as(files::Column::SizeBytes, "size_bytes")
            .column_as(files::Column::Height, "height")
            .column_as(files::Column::Width, "width")
            .column_as(files::Column::EditionName, "edition_name")
            .column_as(files::Column::AudioFingerprint, "audio_fingerprint")
            .column_as(files::Column::SegmentsJson, "segments_json")
            .column_as(files::Column::KeyframesJson, "keyframes_json")
            .column_as(
                files::Column::SubtitlesExtractedAt,
                "subtitles_extracted_at",
            )
            .column_as(files::Column::UnavailableAt, "unavailable_at")
            .column_as(files::Column::ScannedAt, "scanned_at")
            .column_as(files::Column::DiscoveredAt, "discovered_at")
            .into_model::<files::Model>()
            .one(pool)
            .await
    }

    async fn file_thumbnail_asset_id(
        &self,
        pool: &DatabaseConnection,
    ) -> Result<Option<String>, sea_orm::DbErr> {
        let links = node_files::Entity::find()
            .join(
                sea_orm::JoinType::InnerJoin,
                node_files::Relation::Files.def(),
            )
            .filter(node_files::Column::NodeId.eq(self.node_id.clone()))
            .filter(files::Column::UnavailableAt.is_null())
            .order_by_asc(node_files::Column::Order)
            .order_by_asc(node_files::Column::FileId)
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

    // Rank metadata per ancestor by the existing preference order, then return the nearest
    // ancestor whose preferred row includes a poster.
    async fn poster_fallback_asset_id(
        &self,
        pool: &DatabaseConnection,
    ) -> Result<Option<String>, sea_orm::DbErr> {
        if !matches!(
            self.kind,
            nodes::NodeKind::Season | nodes::NodeKind::Episode
        ) {
            return Ok(None);
        }

        Ok(sqlx::query_scalar!(
            r#"
            WITH ranked_ancestor_metadata AS (
                SELECT
                    nc.ancestor_id,
                    nc.depth,
                    nm.poster_asset_id,
                    ROW_NUMBER() OVER (
                        PARTITION BY nc.ancestor_id
                        ORDER BY nm.source DESC, nm.updated_at DESC
                    ) AS metadata_rank
                FROM node_closure nc
                INNER JOIN node_metadata nm ON nm.node_id = nc.ancestor_id
                WHERE nc.descendant_id = ?
                AND nc.depth > 0
            )
            SELECT poster_asset_id AS "poster_asset_id?: String"
            FROM ranked_ancestor_metadata
            WHERE metadata_rank = 1
            AND poster_asset_id IS NOT NULL
            ORDER BY depth ASC
            LIMIT 1
            "#,
            self.node_id,
        )
        .fetch_optional(pool.get_sqlite_connection_pool())
        .await
        .map_err(|error| sea_orm::DbErr::Custom(error.to_string()))?
        .flatten())
    }

    fn release_year(&self) -> Option<String> {
        year_from_unix_timestamp(self.first_aired.or(self.last_aired)?).map(|year| year.to_string())
    }
}

#[derive(Clone, Debug)]
struct ProbeSummary {
    duration_seconds: Option<i64>,
    width: Option<i64>,
    height: Option<i64>,
    video_codec: Option<String>,
    audio_codec: Option<String>,
    fps: Option<f64>,
    video_bitrate: Option<i64>,
    audio_bitrate: Option<i64>,
    audio_channels: Option<i64>,
    has_subtitles: bool,
}

fn minutes_from_seconds_ceil(seconds: i64) -> i64 {
    (seconds + 59) / 60
}

fn format_count_detail(count: i64, singular: &str) -> Option<String> {
    if count <= 0 {
        return None;
    }

    let suffix = if count == 1 {
        singular.to_owned()
    } else {
        format!("{singular}s")
    };
    Some(format!("{count} {suffix}"))
}

fn year_from_unix_timestamp(timestamp: i64) -> Option<i32> {
    DateTime::<Utc>::from_timestamp(timestamp, 0).map(|date| date.year())
}

async fn find_asset(
    pool: &DatabaseConnection,
    asset_id: Option<String>,
) -> Result<Option<Asset>, sea_orm::DbErr> {
    let Some(asset_id) = asset_id else {
        return Ok(None);
    };

    let model = assets::Entity::find_by_id(asset_id).one(pool).await?;
    Ok(model.map(Into::into))
}

fn summarize_probe(probe: &ProbeData) -> ProbeSummary {
    let video = probe.get_video_stream();
    let audio = probe.get_audio_stream();

    ProbeSummary {
        duration_seconds: probe
            .duration_secs
            .map(|value| value.max(0.0).floor() as i64),
        width: video.and_then(|stream| stream.width()).map(i64::from),
        height: video.and_then(|stream| stream.height()).map(i64::from),
        video_codec: video.map(|stream| stream.codec.to_string()),
        audio_codec: audio.map(|stream| stream.codec.to_string()),
        fps: video.and_then(|stream| stream.frame_rate()).map(f64::from),
        video_bitrate: video
            .and_then(|stream| stream.bit_rate)
            .and_then(|value| i64::try_from(value).ok()),
        audio_bitrate: audio
            .and_then(|stream| stream.bit_rate)
            .and_then(|value| i64::try_from(value).ok()),
        audio_channels: audio.and_then(|stream| stream.channels()).map(i64::from),
        has_subtitles: probe.has_subtitles(),
    }
}
