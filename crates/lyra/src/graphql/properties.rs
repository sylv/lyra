use crate::segment_markers::StoredFileSegmentKind;
use crate::{
    auth::RequestAuth,
    entities::{
        assets,
        file_assets::{self, FileAssetRole},
        file_probe, files, libraries, library_users, node_files, node_metadata, nodes, users,
    },
    signer::Signer,
};
use async_graphql::{ComplexObject, Context, Enum, SimpleObject};
use lyra_ffprobe::StreamType as ProbeStreamType;
use lyra_packager::state::{build_track_display_name, language_to_display_name};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
    RelationTrait,
};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Enum)]
pub enum TrackType {
    Audio,
    Subtitle,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Enum)]
pub enum TrackDispositionPreference {
    Normal,
    Sdh,
    Commentary,
}

impl TrackDispositionPreference {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Normal" => Some(Self::Normal),
            "Sdh" => Some(Self::Sdh),
            "Commentary" => Some(Self::Commentary),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Sdh => "Sdh",
            Self::Commentary => "Commentary",
        }
    }
}

#[derive(Clone, Debug, SimpleObject)]
pub struct TrackInfo {
    /// original ffprobe stream index
    pub track_index: i32,
    /// 0-based index within type (maps to HLS.js index directly)
    pub manifest_index: i32,
    pub track_type: TrackType,
    pub display_name: String,
    /// iso 639 language code, null if unparseable
    pub language: Option<String>,
    /// null if forced or unparseable
    pub disposition: Option<TrackDispositionPreference>,
    pub is_forced: bool,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct RecommendedTrack {
    pub manifest_index: i32,
    pub track_type: TrackType,
    pub enabled: bool,
}

#[derive(Clone, Debug, SimpleObject)]
#[graphql(complex)]
pub struct Asset {
    pub id: String,
    pub source_url: Option<String>,
    pub hash_sha256: Option<String>,
    pub size_bytes: Option<i64>,
    pub mime_type: Option<String>,
    pub height: Option<i64>,
    pub width: Option<i64>,
    pub thumbhash: Option<String>,
    pub created_at: i64,
}

#[ComplexObject]
impl Asset {
    pub async fn signed_url(&self, ctx: &Context<'_>) -> async_graphql::Result<String> {
        const ASSET_URL_SIGNATURE_SCOPE: &str = "asset_url";
        const ASSET_URL_SIGNATURE_TTL_SECONDS: i64 = 24 * 60 * 60;

        let auth = ctx.data::<RequestAuth>()?;
        let user_id = auth.get_user_or_err()?.id.as_str();
        let signer = ctx.data_unchecked::<Signer>();
        let signature = signer.sign(
            ASSET_URL_SIGNATURE_SCOPE,
            ASSET_URL_SIGNATURE_TTL_SECONDS,
            &[user_id, &self.id],
        );

        Ok(format!("/api/assets/{}/{}", self.id, signature))
    }
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
pub struct NodeProperties {
    pub display_name: String,
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
    pub background_asset_id: Option<String>,
    #[graphql(skip)]
    pub poster_asset_id: Option<String>,
    #[graphql(skip)]
    pub thumbnail_asset_id: Option<String>,
    #[graphql(skip)]
    pub node_id: String,
}

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
        if let Some(asset_id) = self
            .poster_asset_id
            .clone()
            .or(self.thumbnail_asset_id.clone())
        {
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
}

#[ComplexObject]
impl users::Model {
    pub async fn libraries(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<libraries::Model>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();

        let rows = library_users::Entity::find()
            .filter(library_users::Column::UserId.eq(&self.id))
            .find_also_related(libraries::Entity)
            .all(pool)
            .await?;

        Ok(rows
            .into_iter()
            .filter_map(|(_, library)| library)
            .collect())
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
            .filter(file_assets::Column::FileId.eq(&self.id))
            .filter(file_assets::Column::Role.eq(FileAssetRole::TimelinePreviewSheet))
            .order_by_asc(file_assets::Column::PositionMs)
            .order_by_asc(file_assets::Column::AssetId)
            .all(pool)
            .await?;

        if rows.is_empty() {
            return Ok(Vec::new());
        }

        let asset_ids = rows
            .iter()
            .map(|row| row.asset_id.clone())
            .collect::<Vec<_>>();
        let asset_models = assets::Entity::find()
            .filter(assets::Column::Id.is_in(asset_ids))
            .all(pool)
            .await?;
        let assets_by_id = asset_models
            .into_iter()
            .map(|asset| (asset.id.clone(), asset))
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

    pub async fn tracks(&self, ctx: &Context<'_>) -> Result<Vec<TrackInfo>, async_graphql::Error> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();

        let probe = file_probe::Entity::find_by_id(self.id.clone())
            .one(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let Some(probe) = probe else {
            return Ok(Vec::new());
        };

        let ffprobe_output = match probe.decode_ffprobe_output() {
            Ok(v) => v,
            Err(_) => return Ok(Vec::new()),
        };

        let probe_result = match lyra_ffprobe::probe_streams_from_output(&ffprobe_output) {
            Ok(v) => v,
            Err(_) => return Ok(Vec::new()),
        };

        // separate audio and subtitle tracks, sorted by stream index, to match HLS manifest order
        let mut audio_streams: Vec<_> = probe_result
            .streams
            .iter()
            .filter(|s| s.stream_type == ProbeStreamType::Audio)
            .collect();
        audio_streams.sort_by_key(|s| s.index);

        let mut subtitle_streams: Vec<_> = probe_result
            .streams
            .iter()
            .filter(|s| s.stream_type == ProbeStreamType::Subtitle)
            .collect();
        subtitle_streams.sort_by_key(|s| s.index);

        let mut tracks = Vec::new();

        for (manifest_index, stream) in audio_streams.iter().enumerate() {
            let has_parseable_lang = stream
                .language
                .as_deref()
                .and_then(language_to_display_name)
                .is_some();

            let (language, disposition) = if has_parseable_lang {
                let disp = if stream.is_commentary {
                    Some(TrackDispositionPreference::Commentary)
                } else if stream.is_hearing_impaired {
                    Some(TrackDispositionPreference::Sdh)
                } else {
                    Some(TrackDispositionPreference::Normal)
                };
                (stream.language.clone(), disp)
            } else {
                (None, None)
            };

            let fallback = format!("Audio {}", manifest_index + 1);
            let display_name = build_track_display_name(
                stream.language.as_deref(),
                stream.title.as_deref(),
                &fallback,
                stream.is_forced,
                stream.is_hearing_impaired,
                stream.is_commentary,
            );

            tracks.push(TrackInfo {
                track_index: stream.index as i32,
                manifest_index: manifest_index as i32,
                track_type: TrackType::Audio,
                display_name,
                language,
                disposition,
                is_forced: stream.is_forced,
            });
        }

        for (manifest_index, stream) in subtitle_streams.iter().enumerate() {
            let has_parseable_lang = stream
                .language
                .as_deref()
                .and_then(language_to_display_name)
                .is_some();

            // forced tracks are not a user-selectable disposition; disposition is set to null for them
            let (language, disposition) = if has_parseable_lang && !stream.is_forced {
                let disp = if stream.is_commentary {
                    Some(TrackDispositionPreference::Commentary)
                } else if stream.is_hearing_impaired {
                    Some(TrackDispositionPreference::Sdh)
                } else {
                    Some(TrackDispositionPreference::Normal)
                };
                (stream.language.clone(), disp)
            } else if has_parseable_lang && stream.is_forced {
                (stream.language.clone(), None)
            } else {
                (None, None)
            };

            let fallback = format!("Subtitle {}", manifest_index + 1);
            let display_name = build_track_display_name(
                stream.language.as_deref(),
                stream.title.as_deref(),
                &fallback,
                stream.is_forced,
                stream.is_hearing_impaired,
                stream.is_commentary,
            );

            tracks.push(TrackInfo {
                track_index: stream.index as i32,
                manifest_index: manifest_index as i32,
                track_type: TrackType::Subtitle,
                display_name,
                language,
                disposition,
                is_forced: stream.is_forced,
            });
        }

        Ok(tracks)
    }

    pub async fn recommended_tracks(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<RecommendedTrack>, async_graphql::Error> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let auth = ctx.data::<RequestAuth>()?;

        let Some(user) = auth.get_user() else {
            return Ok(Vec::new());
        };

        let probe = file_probe::Entity::find_by_id(self.id.clone())
            .one(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let Some(probe) = probe else {
            return Ok(Vec::new());
        };

        let ffprobe_output = match probe.decode_ffprobe_output() {
            Ok(v) => v,
            Err(_) => return Ok(Vec::new()),
        };

        let probe_result = match lyra_ffprobe::probe_streams_from_output(&ffprobe_output) {
            Ok(v) => v,
            Err(_) => return Ok(Vec::new()),
        };

        let mut audio_streams: Vec<_> = probe_result
            .streams
            .iter()
            .filter(|s| s.stream_type == ProbeStreamType::Audio)
            .collect();
        audio_streams.sort_by_key(|s| s.index);

        let mut subtitle_streams: Vec<_> = probe_result
            .streams
            .iter()
            .filter(|s| s.stream_type == ProbeStreamType::Subtitle)
            .collect();
        subtitle_streams.sort_by_key(|s| s.index);

        let recommendations = compute_recommended_tracks(&audio_streams, &subtitle_streams, user);

        Ok(recommendations)
    }

    pub async fn segments(&self, _ctx: &Context<'_>) -> Result<Vec<FileSegment>, sea_orm::DbErr> {
        if self.segments_json.is_none() {
            return Ok(Vec::new());
        }

        let decoded = match self.decode_segments() {
            Ok(segments) => segments,
            Err(error) => {
                tracing::warn!(file_id = self.id, error = ?error, "failed to decode file segments");
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

impl NodeProperties {
    pub async fn from_node(
        pool: &DatabaseConnection,
        node: &nodes::Model,
        metadata: Option<node_metadata::Model>,
    ) -> Result<Self, sea_orm::DbErr> {
        let default_file = Self::primary_file_for_node(pool, &node.id).await?;
        let probe = if let Some(file) = &default_file {
            file_probe::Entity::find_by_id(file.id.clone())
                .one(pool)
                .await?
        } else {
            None
        };

        let duration_seconds = probe
            .as_ref()
            .and_then(|probe| probe.duration_s)
            .filter(|seconds| *seconds > 0);

        let runtime_minutes = duration_seconds.map(minutes_from_seconds_ceil);

        Ok(match metadata {
            Some(metadata) => Self {
                display_name: metadata.name,
                description: metadata.description,
                rating: metadata.score_normalized.map(|score| score as f64 / 10.0),
                season_number: node.season_number,
                episode_number: node.episode_number,
                runtime_minutes,
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
                has_subtitles: probe.as_ref().map(|probe| probe.has_subtitles != 0),
                file_size_bytes: default_file.as_ref().map(|file| file.size_bytes),
                released_at: metadata.released_at,
                ended_at: metadata.ended_at,
                created_at: Some(metadata.created_at),
                updated_at: Some(metadata.updated_at),
                background_asset_id: metadata.background_asset_id,
                poster_asset_id: metadata.poster_asset_id,
                thumbnail_asset_id: metadata.thumbnail_asset_id,
                node_id: node.id.clone(),
            },
            None => Self {
                display_name: node.name.clone(),
                description: None,
                rating: None,
                season_number: node.season_number,
                episode_number: node.episode_number,
                runtime_minutes,
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
                has_subtitles: probe.as_ref().map(|probe| probe.has_subtitles != 0),
                file_size_bytes: default_file.as_ref().map(|file| file.size_bytes),
                released_at: None,
                ended_at: None,
                created_at: None,
                updated_at: None,
                background_asset_id: None,
                poster_asset_id: None,
                thumbnail_asset_id: None,
                node_id: node.id.clone(),
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
    asset_id: Option<String>,
) -> Result<Option<Asset>, sea_orm::DbErr> {
    let Some(asset_id) = asset_id else {
        return Ok(None);
    };

    let model = assets::Entity::find_by_id(asset_id).one(pool).await?;
    Ok(model.map(Asset::from_model))
}

fn normalize_lang(tag: &str) -> Option<isolang::Language> {
    isolang::Language::from_639_3(tag).or_else(|| isolang::Language::from_639_1(tag))
}

fn langs_match(a: &str, b: &str) -> bool {
    match (normalize_lang(a), normalize_lang(b)) {
        (Some(la), Some(lb)) => la == lb,
        _ => false,
    }
}

fn compute_recommended_tracks(
    audio_streams: &[&lyra_ffprobe::Stream],
    subtitle_streams: &[&lyra_ffprobe::Stream],
    user: &users::Model,
) -> Vec<RecommendedTrack> {
    let mut recommendations = Vec::new();

    // --- audio recommendation ---
    let audio_manifest_index = if audio_streams.is_empty() {
        None
    } else if let Some(ref pref_lang) = user.preferred_audio_language {
        let pref_disp = user
            .preferred_audio_disposition
            .as_deref()
            .and_then(TrackDispositionPreference::from_str);

        // find all tracks matching the preferred language
        let matching: Vec<(usize, &&lyra_ffprobe::Stream)> = audio_streams
            .iter()
            .enumerate()
            .filter(|(_, s)| {
                s.language
                    .as_deref()
                    .map(|lang| langs_match(lang, pref_lang))
                    .unwrap_or(false)
            })
            .collect();

        if matching.is_empty() {
            Some(0usize)
        } else if let Some(disp) = pref_disp {
            // prefer exact disposition match, then fall through ordering
            let exact = matching.iter().find(|(_, s)| match disp {
                TrackDispositionPreference::Commentary => s.is_commentary,
                TrackDispositionPreference::Sdh => s.is_hearing_impaired && !s.is_commentary,
                TrackDispositionPreference::Normal => !s.is_hearing_impaired && !s.is_commentary,
            });
            Some(
                exact
                    .or_else(|| matching.first())
                    .map(|(i, _)| *i)
                    .unwrap_or(0),
            )
        } else {
            // prefer Normal > SDH > Commentary > other
            let pick = matching
                .iter()
                .find(|(_, s)| !s.is_hearing_impaired && !s.is_commentary)
                .or_else(|| {
                    matching
                        .iter()
                        .find(|(_, s)| s.is_hearing_impaired && !s.is_commentary)
                })
                .or_else(|| matching.iter().find(|(_, s)| s.is_commentary))
                .or_else(|| matching.first());
            Some(pick.map(|(i, _)| *i).unwrap_or(0))
        }
    } else {
        Some(0usize)
    };

    if let Some(idx) = audio_manifest_index {
        recommendations.push(RecommendedTrack {
            manifest_index: idx as i32,
            track_type: TrackType::Audio,
            enabled: true,
        });
    }

    // determine active audio language for forced subtitle matching
    let active_audio_lang: Option<String> = user
        .preferred_audio_language
        .clone()
        .or_else(|| audio_streams.first().and_then(|s| s.language.clone()));

    // --- subtitle recommendations ---
    // forced tracks whose language matches active audio are always enabled
    for (manifest_index, stream) in subtitle_streams.iter().enumerate() {
        if !stream.is_forced {
            continue;
        }
        let enabled = match (&stream.language, &active_audio_lang) {
            (Some(sub_lang), Some(audio_lang)) => langs_match(sub_lang, audio_lang),
            _ => false,
        };
        recommendations.push(RecommendedTrack {
            manifest_index: manifest_index as i32,
            track_type: TrackType::Subtitle,
            enabled,
        });
    }

    // non-forced subtitle tracks
    let non_forced: Vec<(usize, &&lyra_ffprobe::Stream)> = subtitle_streams
        .iter()
        .enumerate()
        .filter(|(_, s)| !s.is_forced)
        .collect();

    if let Some(ref pref_lang) = user.preferred_subtitle_language {
        let pref_disp = user
            .preferred_subtitle_disposition
            .as_deref()
            .and_then(TrackDispositionPreference::from_str);

        let matching: Vec<(usize, &&lyra_ffprobe::Stream)> = non_forced
            .iter()
            .filter(|(_, s)| {
                s.language
                    .as_deref()
                    .map(|lang| langs_match(lang, pref_lang))
                    .unwrap_or(false)
            })
            .cloned()
            .collect();

        let best = if matching.is_empty() {
            None
        } else if let Some(disp) = pref_disp {
            let exact = matching.iter().find(|(_, s)| match disp {
                TrackDispositionPreference::Commentary => s.is_commentary,
                TrackDispositionPreference::Sdh => s.is_hearing_impaired && !s.is_commentary,
                TrackDispositionPreference::Normal => !s.is_hearing_impaired && !s.is_commentary,
            });
            exact.or_else(|| matching.first()).map(|(i, _)| *i)
        } else {
            let pick = matching
                .iter()
                .find(|(_, s)| !s.is_hearing_impaired && !s.is_commentary)
                .or_else(|| {
                    matching
                        .iter()
                        .find(|(_, s)| s.is_hearing_impaired && !s.is_commentary)
                })
                .or_else(|| matching.iter().find(|(_, s)| s.is_commentary))
                .or_else(|| matching.first());
            pick.map(|(i, _)| *i)
        };

        for (manifest_index, _) in &non_forced {
            let enabled = best.map(|b| b == *manifest_index).unwrap_or(false);
            recommendations.push(RecommendedTrack {
                manifest_index: *manifest_index as i32,
                track_type: TrackType::Subtitle,
                enabled,
            });
        }
    } else {
        // no subtitle preference → disable all non-forced tracks
        for (manifest_index, _) in &non_forced {
            recommendations.push(RecommendedTrack {
                manifest_index: *manifest_index as i32,
                track_type: TrackType::Subtitle,
                enabled: false,
            });
        }
    }

    recommendations
}
