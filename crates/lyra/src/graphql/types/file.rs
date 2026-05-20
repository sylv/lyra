use crate::assets::sign_asset_url;
use crate::auth::RequestAuth;
use crate::entities::{
    assets,
    file_assets::{self, FileAssetRole},
    file_probe, file_subtitles, files, node_files, nodes, users, watch_progress,
};
use crate::graphql::properties::{
    FileProbe, FileSegment, FileSegmentKind, Playback, PlaybackAudioCodec, PlaybackAudioProfileId,
    PlaybackAudioRendition, PlaybackAudioTrack, PlaybackSubtitleCodec, PlaybackSubtitleKind,
    PlaybackSubtitleRendition, PlaybackSubtitleTrack, PlaybackVideoCodec, PlaybackVideoProfileId,
    PlaybackVideoRendition, PlaybackVideoTrack, TimelinePreviewSheet, TrackDispositionPreference,
};
use crate::graphql::query::current_user_id;
use crate::hls;
use crate::jobs;
use crate::segment_markers::StoredFileSegmentKind;
use crate::subtitles::job_extract::FileSubtitleExtractJob;
use crate::subtitles::job_process::FileSubtitleProcessJob;
use crate::subtitles::language::{
    SubtitleSelectionCandidate, SubtitleTrackVariant, language_match_strength,
    select_subtitle_track,
};
use crate::subtitles::subtitle_kind_from_stream;
use async_graphql::{ComplexObject, Context, SimpleObject};
use lyra_packager::{Compatibility, audio_profile, video_profile};
use lyra_probe::{Codec, HDRFormat, Stream, StreamDetails, StreamKind};
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
    RelationTrait,
};
use std::collections::HashMap;

const ON_DEMAND_SUBTITLE_JOB_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(120);

#[derive(Clone, Debug, SimpleObject)]
pub struct ResumeHint {
    pub id: String,
    pub start_ms: i64,
    pub updated_at: i64,
}

#[ComplexObject]
impl files::Model {
    pub async fn probe(&self, ctx: &Context<'_>) -> Result<Option<FileProbe>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let probe = file_probe::Entity::find_by_id(self.id.clone())
            .one(pool)
            .await?;
        Ok(probe
            .as_ref()
            .and_then(|probe| probe.get_probe().ok())
            .map(|probe| summarize_probe(&probe)))
    }

    pub async fn resume_hint(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<ResumeHint>, async_graphql::Error> {
        let Some(user_id) = current_user_id(ctx) else {
            return Ok(None);
        };

        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let probe_row = file_probe::Entity::find_by_id(self.id.clone())
            .one(pool)
            .await?;
        let Some(probe_row) = probe_row else {
            // TODO: Queue or await probing here so freshly discovered files can still resume.
            return Ok(None);
        };
        let duration_secs = probe_row
            .get_probe()
            .map_err(|error| async_graphql::Error::new(error.to_string()))?
            .duration_secs
            .filter(|duration_secs| duration_secs.is_finite() && *duration_secs > 0.0);
        let Some(duration_secs) = duration_secs else {
            return Ok(None);
        };

        let linked_node_ids = node_files::Entity::find()
            .join(
                sea_orm::JoinType::InnerJoin,
                node_files::Relation::Nodes.def(),
            )
            .filter(node_files::Column::FileId.eq(self.id.clone()))
            .order_by_asc(nodes::Column::Order)
            .order_by_asc(nodes::Column::Id)
            .select_only()
            .column(node_files::Column::NodeId)
            .into_tuple::<String>()
            .all(pool)
            .await?;
        let node_order_by_id = linked_node_ids
            .iter()
            .enumerate()
            .map(|(index, node_id)| (node_id.clone(), index))
            .collect::<HashMap<_, _>>();

        let mut rows = watch_progress::Entity::find()
            .filter(watch_progress::Column::UserId.eq(user_id))
            .filter(
                Condition::any()
                    .add(watch_progress::Column::FileId.eq(self.id.clone()))
                    .add(watch_progress::Column::NodeId.is_in(linked_node_ids)),
            )
            .all(pool)
            .await?;

        rows.sort_by_key(|row| {
            (
                node_order_by_id
                    .get(&row.node_id)
                    .copied()
                    .unwrap_or(usize::MAX),
                row.node_id.clone(),
                row.updated_at,
            )
        });

        let Some(row) = rows.into_iter().next() else {
            return Ok(None);
        };
        let Some(progress_percent) = watch_progress::resume_progress(row.progress_percent) else {
            return Ok(None);
        };

        Ok(Some(ResumeHint {
            id: row.id,
            start_ms: (duration_secs * f64::from(progress_percent) * 1000.0).round() as i64,
            updated_at: row.updated_at,
        }))
    }

    pub async fn playback(
        &self,
        ctx: &Context<'_>,
        language_hint: Option<String>,
    ) -> Result<Playback, async_graphql::Error> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let auth = ctx.data::<RequestAuth>()?;
        let user = auth.get_user();
        let (probe_data, keyframes) = hls::load_probe_data_for_playback_options(pool, &self.id)
            .await
            .map_err(|error| async_graphql::Error::new(error.to_string()))?;

        let default_video_stream_index = probe_data.get_video_stream().map(|stream| stream.index);
        let mut video_streams = probe_data
            .streams
            .iter()
            .filter(|stream| stream.kind() == StreamKind::Video)
            .collect::<Vec<_>>();
        video_streams.sort_by_key(|stream| {
            (
                default_video_stream_index != Some(stream.index),
                stream.index,
            )
        });
        let video = video_streams
            .into_iter()
            .enumerate()
            .filter_map(|(position, stream)| {
                let renditions = derive_video_renditions(
                    stream,
                    keyframes
                        .as_ref()
                        .filter(|keyframes| keyframes.video_stream_index == stream.index),
                );
                if renditions.is_empty() {
                    return None;
                }

                Some(PlaybackVideoTrack {
                    source_track_id: source_track_id(stream.index),
                    display_name: stream
                        .display_name
                        .clone()
                        .unwrap_or_else(|| format!("Track {}", position + 1)),
                    autoselect: default_video_stream_index == Some(stream.index),
                    renditions,
                })
            })
            .collect::<Vec<_>>();
        if video.is_empty() {
            return Err(async_graphql::Error::new(
                "File has no playable video stream",
            ));
        }

        let (audio, active_audio_language) =
            build_audio_tracks(&probe_data, user, language_hint.as_deref());
        let subtitles = load_playback_subtitle_tracks(
            pool,
            self,
            user,
            &probe_data,
            language_hint.as_deref(),
            active_audio_language.as_deref(),
        )
        .await?;

        Ok(Playback {
            hls_url_template: hls::sign_playback_url_template(&self.id)
                .map_err(|error| async_graphql::Error::new(error.to_string()))?,
            video,
            audio,
            subtitles,
        })
    }

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
                asset: asset.clone().into(),
            });
        }

        Ok(sheets)
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

#[derive(Clone)]
struct BuiltSubtitleTrack {
    track: PlaybackSubtitleTrack,
    candidate: SubtitleSelectionCandidate,
    stream_index: u32,
}

#[derive(Clone)]
struct BuiltAudioTrack {
    track: PlaybackAudioTrack,
    score: (i32, i32, i32, i32, i32, i32),
    stream_index: u32,
}

fn build_audio_tracks(
    probe_data: &lyra_probe::ProbeData,
    user: Option<&users::Model>,
    language_hint: Option<&str>,
) -> (Vec<PlaybackAudioTrack>, Option<String>) {
    let mut audio_streams: Vec<_> = probe_data
        .streams
        .iter()
        .filter(|stream| stream.kind() == StreamKind::Audio)
        .collect();
    audio_streams.sort_by_key(|stream| stream.index);

    let selected_index = compute_recommended_audio_track_index(&audio_streams, user, language_hint);
    let selected_stream_index = selected_index
        .and_then(|index| audio_streams.get(index))
        .map(|stream| stream.index);
    let active_audio_language = selected_index
        .and_then(|index| audio_streams.get(index))
        .and_then(|stream| stream.language_bcp47.clone());

    let mut built = audio_streams
        .iter()
        .enumerate()
        .map(|(position, stream)| BuiltAudioTrack {
            track: PlaybackAudioTrack {
                source_track_id: source_track_id(stream.index),
                display_name: stream
                    .display_name
                    .clone()
                    .unwrap_or_else(|| format!("Audio {}", position + 1)),
                language_bcp47: stream.language_bcp47.clone(),
                autoselect: selected_stream_index == Some(stream.index),
                renditions: derive_audio_renditions(stream),
            },
            score: audio_track_sort_score(stream, user, language_hint),
            stream_index: stream.index,
        })
        .filter(|track| !track.track.renditions.is_empty())
        .collect::<Vec<_>>();

    built.sort_by(|left, right| {
        right
            .track
            .autoselect
            .cmp(&left.track.autoselect)
            .then_with(|| right.score.cmp(&left.score))
            .then_with(|| left.stream_index.cmp(&right.stream_index))
    });

    (
        built.into_iter().map(|track| track.track).collect(),
        active_audio_language,
    )
}

async fn load_playback_subtitle_tracks(
    pool: &DatabaseConnection,
    file: &files::Model,
    user: Option<&users::Model>,
    probe_data: &lyra_probe::ProbeData,
    language_hint: Option<&str>,
    active_audio_language: Option<&str>,
) -> Result<Vec<PlaybackSubtitleTrack>, async_graphql::Error> {
    let subtitle_rows = ensure_current_subtitle_rows(pool, file, probe_data).await?;
    let mut rows_by_stream_index: HashMap<i64, Vec<file_subtitles::Model>> = HashMap::new();
    for row in subtitle_rows {
        rows_by_stream_index
            .entry(row.stream_index)
            .or_default()
            .push(row);
    }

    let mut built_tracks = Vec::new();
    for stream in probe_data
        .streams
        .iter()
        .filter(|stream| stream.kind() == StreamKind::Subtitle)
    {
        let Some(track) = build_playback_subtitle_track(
            pool,
            stream,
            rows_by_stream_index.get(&i64::from(stream.index)),
        )
        .await?
        else {
            continue;
        };
        built_tracks.push(track);
    }

    let preferred_subtitle_languages: Vec<String> = user
        .map(|user| {
            serde_json::from_str::<Vec<String>>(&user.preferred_subtitle_languages)
                .unwrap_or_default()
        })
        .unwrap_or_default();
    let language_hints = language_hint
        .map(|hint| vec![hint.to_string()])
        .unwrap_or_default();
    let selected_track_id = user.and_then(|user| {
        select_subtitle_track(
            &built_tracks
                .iter()
                .map(|track| track.candidate.clone())
                .collect::<Vec<_>>(),
            user.subtitle_mode,
            &preferred_subtitle_languages,
            &language_hints,
            user.subtitle_variant_preference,
            active_audio_language,
        )
    });

    built_tracks.sort_by(|left, right| {
        (selected_track_id.as_deref() != Some(left.candidate.id.as_str()))
            .cmp(&(selected_track_id.as_deref() != Some(right.candidate.id.as_str())))
            .then_with(|| left.stream_index.cmp(&right.stream_index))
    });

    Ok(built_tracks
        .into_iter()
        .map(|mut built| {
            built.track.autoselect =
                selected_track_id.as_deref() == Some(built.candidate.id.as_str());
            built.track
        })
        .collect())
}

async fn ensure_current_subtitle_rows(
    pool: &DatabaseConnection,
    file: &files::Model,
    probe_data: &lyra_probe::ProbeData,
) -> Result<Vec<file_subtitles::Model>, async_graphql::Error> {
    let mut current_file = file.clone();
    let mut rows = load_current_subtitle_rows(pool, &current_file).await?;
    if subtitle_streams_missing_source_rows(probe_data, &rows) {
        current_file = files::Entity::find_by_id(file.id.clone())
            .one(pool)
            .await
            .map_err(|error| async_graphql::Error::new(error.to_string()))?
            .ok_or_else(|| async_graphql::Error::new("File not found"))?;
        jobs::try_run_job(
            pool,
            &FileSubtitleExtractJob,
            current_file.clone(),
            ON_DEMAND_SUBTITLE_JOB_TIMEOUT,
        )
        .await
        .map_err(|error| async_graphql::Error::new(error.to_string()))?;
        current_file = files::Entity::find_by_id(file.id.clone())
            .one(pool)
            .await
            .map_err(|error| async_graphql::Error::new(error.to_string()))?
            .ok_or_else(|| async_graphql::Error::new("File not found"))?;
        rows = load_current_subtitle_rows(pool, &current_file).await?;
    }

    Ok(rows)
}

fn subtitle_streams_missing_source_rows(
    probe_data: &lyra_probe::ProbeData,
    rows: &[file_subtitles::Model],
) -> bool {
    let mut source_streams = rows
        .iter()
        .filter(|row| row.derived_from_subtitle_id.is_none())
        .map(|row| row.stream_index)
        .collect::<std::collections::HashSet<_>>();

    probe_data
        .streams
        .iter()
        .filter(|stream| stream.kind() == StreamKind::Subtitle)
        .any(|stream| !source_streams.remove(&i64::from(stream.index)))
}

async fn load_current_subtitle_rows(
    pool: &DatabaseConnection,
    file: &files::Model,
) -> Result<Vec<file_subtitles::Model>, async_graphql::Error> {
    let Some(latest_seen_at) = file.subtitles_extracted_at else {
        return Ok(Vec::new());
    };

    file_subtitles::Entity::find()
        .filter(file_subtitles::Column::FileId.eq(file.id.clone()))
        .filter(file_subtitles::Column::LastSeenAt.eq(latest_seen_at))
        .all(pool)
        .await
        .map_err(|error| async_graphql::Error::new(error.to_string()))
}

async fn build_playback_subtitle_track(
    pool: &DatabaseConnection,
    stream: &Stream,
    rows: Option<&Vec<file_subtitles::Model>>,
) -> Result<Option<BuiltSubtitleTrack>, async_graphql::Error> {
    let Some(kind) = subtitle_kind_from_stream(stream) else {
        return Ok(None);
    };
    let Some(source_row) = rows.and_then(|rows| {
        rows.iter()
            .find(|row| row.derived_from_subtitle_id.is_none())
    }) else {
        return Ok(None);
    };
    let display_name = stream
        .display_name
        .clone()
        .unwrap_or_else(|| format!("Subtitle {}", stream.index + 1));
    let variant = track_variant(stream);
    let mut renditions = Vec::new();

    match kind {
        file_subtitles::SubtitleKind::Vtt => {
            renditions.push(build_subtitle_rendition(source_row, "Original".to_string()));
        }
        file_subtitles::SubtitleKind::Srt | file_subtitles::SubtitleKind::Ass => {
            renditions.push(build_subtitle_rendition(
                source_row,
                format!("Original {}", subtitle_kind_label(kind)),
            ));
            let converted = ensure_processed_subtitle(
                pool,
                source_row,
                file_subtitles::SubtitleSource::Converted,
            )
            .await?;
            renditions.push(build_subtitle_rendition(
                &converted,
                format!("Converted from {}", subtitle_kind_label(kind)),
            ));
        }
        file_subtitles::SubtitleKind::MovText
        | file_subtitles::SubtitleKind::Text
        | file_subtitles::SubtitleKind::Ttml => {
            let converted = ensure_processed_subtitle(
                pool,
                source_row,
                file_subtitles::SubtitleSource::Converted,
            )
            .await?;
            renditions.push(build_subtitle_rendition(
                &converted,
                format!("Converted from {}", subtitle_kind_label(kind)),
            ));
        }
        file_subtitles::SubtitleKind::Pgs | file_subtitles::SubtitleKind::VobSub => {
            let ocr =
                ensure_processed_subtitle(pool, source_row, file_subtitles::SubtitleSource::Ocr)
                    .await?;
            renditions.push(build_subtitle_rendition(
                &ocr,
                format!("Converted from {} using OCR", subtitle_kind_label(kind)),
            ));
        }
    }

    if let Some(generated) = rows.and_then(|rows| {
        rows.iter().find(|row| {
            row.source == file_subtitles::SubtitleSource::Generated
                && row.kind == file_subtitles::SubtitleKind::Vtt
        })
    }) {
        renditions.push(build_subtitle_rendition(generated, "Generated".to_string()));
    }

    Ok(Some(BuiltSubtitleTrack {
        track: PlaybackSubtitleTrack {
            source_track_id: source_track_id(stream.index),
            display_name,
            language_bcp47: stream.language_bcp47.clone(),
            kind: subtitle_track_kind(stream),
            autoselect: false,
            renditions,
        },
        candidate: SubtitleSelectionCandidate {
            id: source_track_id(stream.index),
            language_bcp47: stream.language_bcp47.clone(),
            variant,
        },
        stream_index: stream.index,
    }))
}

async fn ensure_processed_subtitle(
    pool: &DatabaseConnection,
    source_row: &file_subtitles::Model,
    desired_source: file_subtitles::SubtitleSource,
) -> Result<file_subtitles::Model, async_graphql::Error> {
    let mut derived = file_subtitles::Entity::find()
        .filter(file_subtitles::Column::DerivedFromSubtitleId.eq(source_row.id.clone()))
        .filter(file_subtitles::Column::Source.eq(desired_source))
        .filter(file_subtitles::Column::Kind.eq(file_subtitles::SubtitleKind::Vtt))
        .one(pool)
        .await
        .map_err(|error| async_graphql::Error::new(error.to_string()))?;
    if derived.is_none() {
        jobs::try_run_job(
            pool,
            &FileSubtitleProcessJob,
            source_row.clone(),
            ON_DEMAND_SUBTITLE_JOB_TIMEOUT,
        )
        .await
        .map_err(|error| async_graphql::Error::new(error.to_string()))?;
        derived = file_subtitles::Entity::find()
            .filter(file_subtitles::Column::DerivedFromSubtitleId.eq(source_row.id.clone()))
            .filter(file_subtitles::Column::Source.eq(desired_source))
            .filter(file_subtitles::Column::Kind.eq(file_subtitles::SubtitleKind::Vtt))
            .one(pool)
            .await
            .map_err(|error| async_graphql::Error::new(error.to_string()))?;
    }

    derived.ok_or_else(|| async_graphql::Error::new("Subtitle rendition not available"))
}

fn build_subtitle_rendition(
    row: &file_subtitles::Model,
    display_info: String,
) -> PlaybackSubtitleRendition {
    PlaybackSubtitleRendition {
        variant_id: (row.kind == file_subtitles::SubtitleKind::Vtt)
            .then(|| subtitle_variant_id(row.stream_index as u32, row.source)),
        signed_url: sign_asset_url(&row.asset_id),
        display_info,
        codec: playback_subtitle_codec(row.kind),
    }
}

pub(crate) fn source_track_id(stream_index: u32) -> String {
    stream_index.to_string()
}

pub(crate) fn parse_source_track_id(track_id: &str) -> Option<u32> {
    track_id.parse().ok()
}

fn track_variant(stream: &Stream) -> SubtitleTrackVariant {
    if stream.is_commentary() {
        SubtitleTrackVariant::Commentary
    } else if stream.is_forced() {
        SubtitleTrackVariant::Forced
    } else if stream.is_hearing_impaired() {
        SubtitleTrackVariant::Sdh
    } else {
        SubtitleTrackVariant::Normal
    }
}

fn subtitle_track_kind(stream: &Stream) -> PlaybackSubtitleKind {
    if stream.is_commentary() {
        PlaybackSubtitleKind::Commentary
    } else if stream.is_forced() {
        PlaybackSubtitleKind::Forced
    } else if stream.is_hearing_impaired() {
        PlaybackSubtitleKind::Captions
    } else {
        PlaybackSubtitleKind::Subtitles
    }
}

fn subtitle_variant_id(stream_index: u32, source: file_subtitles::SubtitleSource) -> String {
    let suffix = match source {
        file_subtitles::SubtitleSource::Extracted => "original",
        file_subtitles::SubtitleSource::Converted => "converted",
        file_subtitles::SubtitleSource::Ocr => "ocr",
        file_subtitles::SubtitleSource::Generated => "generated",
    };
    format!("s{stream_index}-{suffix}")
}

fn playback_subtitle_codec(kind: file_subtitles::SubtitleKind) -> PlaybackSubtitleCodec {
    match kind {
        file_subtitles::SubtitleKind::Vtt => PlaybackSubtitleCodec::Vtt,
        file_subtitles::SubtitleKind::Srt => PlaybackSubtitleCodec::Srt,
        file_subtitles::SubtitleKind::Ass => PlaybackSubtitleCodec::Ass,
        file_subtitles::SubtitleKind::MovText => PlaybackSubtitleCodec::MovText,
        file_subtitles::SubtitleKind::Text => PlaybackSubtitleCodec::Text,
        file_subtitles::SubtitleKind::Ttml => PlaybackSubtitleCodec::Ttml,
        file_subtitles::SubtitleKind::Pgs => PlaybackSubtitleCodec::Pgs,
        file_subtitles::SubtitleKind::VobSub => PlaybackSubtitleCodec::VobSub,
    }
}

fn subtitle_kind_label(kind: file_subtitles::SubtitleKind) -> &'static str {
    match kind {
        file_subtitles::SubtitleKind::Srt => "SRT",
        file_subtitles::SubtitleKind::Vtt => "WebVTT",
        file_subtitles::SubtitleKind::Ass => "ASS",
        file_subtitles::SubtitleKind::MovText => "MovText",
        file_subtitles::SubtitleKind::Text => "Text",
        file_subtitles::SubtitleKind::Ttml => "TTML",
        file_subtitles::SubtitleKind::Pgs => "PGS",
        file_subtitles::SubtitleKind::VobSub => "VobSub",
    }
}

fn summarize_probe(probe: &lyra_probe::ProbeData) -> FileProbe {
    let video = probe.get_video_stream();
    let audio = probe.get_audio_stream();
    let duration_seconds = probe
        .duration_secs
        .map(|value| value.max(0.0).floor() as i64)
        .filter(|seconds| *seconds > 0);

    FileProbe {
        runtime_minutes: duration_seconds.map(minutes_from_seconds_ceil),
        duration_seconds,
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

fn minutes_from_seconds_ceil(seconds: i64) -> i64 {
    (seconds + 59) / 60
}

fn derive_video_renditions(
    stream: &Stream,
    keyframes: Option<&lyra_probe::VideoKeyframes>,
) -> Vec<PlaybackVideoRendition> {
    let mut renditions = Vec::new();

    for profile_id in ["copy", "h264"] {
        let Some(profile) = video_profile(profile_id) else {
            continue;
        };
        let Some(compatibility) = profile.compatible_with(stream) else {
            continue;
        };
        if compatibility == Compatibility::KeyframeAligned && keyframes.is_none() {
            continue;
        }

        match profile.id() {
            "copy" => {
                let Some(codec_tag) = lyra_probe::video_codec_tag(stream) else {
                    continue;
                };
                let Some(codec) = playback_video_codec(stream) else {
                    continue;
                };
                renditions.push(PlaybackVideoRendition {
                    pair_id: hls::video_pair_id(stream.index, profile.id()),
                    profile_id: PlaybackVideoProfileId::Copy,
                    codec,
                    display_info: format_video_display_info(stream, codec, true),
                    codec_tag,
                });
            }
            "h264" => renditions.push(PlaybackVideoRendition {
                pair_id: hls::video_pair_id(stream.index, profile.id()),
                profile_id: PlaybackVideoProfileId::H264,
                codec: PlaybackVideoCodec::H264,
                display_info: format_video_display_info(stream, PlaybackVideoCodec::H264, false),
                codec_tag: lyra_probe::TRANSCODED_H264_VIDEO_CODEC_TAG.to_string(),
            }),
            _ => {}
        }
    }

    renditions
}

fn playback_video_codec(stream: &Stream) -> Option<PlaybackVideoCodec> {
    match stream.codec {
        Codec::VideoH264 => Some(PlaybackVideoCodec::H264),
        Codec::VideoH265 => Some(PlaybackVideoCodec::H265),
        Codec::VideoAv1 => Some(PlaybackVideoCodec::Av1),
        _ => None,
    }
}

fn derive_audio_renditions(stream: &Stream) -> Vec<PlaybackAudioRendition> {
    let mut renditions = Vec::new();

    for profile_id in ["aac"] {
        let Some(profile) = audio_profile(profile_id) else {
            continue;
        };
        if profile.compatible_with(stream).is_none() {
            continue;
        }

        match profile.id() {
            "aac" => renditions.push(PlaybackAudioRendition {
                pair_id: hls::audio_pair_id(stream.index, profile.id()),
                profile_id: PlaybackAudioProfileId::Aac,
                codec: PlaybackAudioCodec::Aac,
                display_info: format_transcoded_audio_display_info(stream),
                codec_tag: lyra_probe::audio_codec_tag(&lyra_probe::Codec::AudioAac)
                    .expect("AAC codec tag must exist")
                    .to_string(),
            }),
            _ => {}
        }
    }

    renditions
}

fn audio_track_sort_score(
    stream: &Stream,
    user: Option<&users::Model>,
    language_hint: Option<&str>,
) -> (i32, i32, i32, i32, i32, i32) {
    let preferred_language_strength = user
        .and_then(|user| user.preferred_audio_language.as_deref())
        .and_then(|preferred_language| {
            stream
                .language_bcp47
                .as_deref()
                .and_then(|language| language_match_strength(language, preferred_language))
        })
        .map(|strength| strength as i32)
        .unwrap_or_default();
    let hint_language_strength = language_hint
        .and_then(|hint| {
            stream
                .language_bcp47
                .as_deref()
                .and_then(|language| language_match_strength(language, hint))
        })
        .map(|strength| strength as i32)
        .unwrap_or_default();
    let preferred_disposition = user
        .and_then(|user| user.preferred_audio_disposition.as_deref())
        .and_then(TrackDispositionPreference::from_str);
    let disposition_rank = match preferred_disposition {
        Some(TrackDispositionPreference::Commentary) if stream.is_commentary() => 3,
        Some(TrackDispositionPreference::Sdh)
            if stream.is_hearing_impaired() && !stream.is_commentary() =>
        {
            3
        }
        Some(TrackDispositionPreference::Normal)
            if !stream.is_hearing_impaired() && !stream.is_commentary() =>
        {
            3
        }
        Some(_) => 0,
        None if !stream.is_hearing_impaired() && !stream.is_commentary() => 3,
        None if stream.is_hearing_impaired() && !stream.is_commentary() => 2,
        None if stream.is_commentary() => 1,
        None => 0,
    };
    let default_rank = stream
        .disposition
        .contains(lyra_probe::StreamDisposition::DEFAULT) as i32;
    let regular_rank = (!stream.is_commentary() && !stream.is_hearing_impaired()) as i32;
    let index_rank = -(stream.index as i32);

    (
        preferred_language_strength,
        hint_language_strength,
        disposition_rank,
        default_rank,
        regular_rank,
        index_rank,
    )
}

fn compute_recommended_audio_track_index(
    audio_streams: &[&Stream],
    user: Option<&users::Model>,
    language_hint: Option<&str>,
) -> Option<usize> {
    audio_streams
        .iter()
        .enumerate()
        .max_by_key(|(_, stream)| audio_track_sort_score(stream, user, language_hint))
        .map(|(index, _)| index)
}

fn format_video_display_info(stream: &Stream, codec: PlaybackVideoCodec, original: bool) -> String {
    let mut parts = vec![if original {
        "Original".to_string()
    } else {
        playback_video_codec_label(codec).to_string()
    }];

    if let (Some(width), Some(height)) = (stream.width(), stream.height()) {
        let _ = width;
        parts.push(video_resolution_label(height));
    }
    parts.push(if original {
        video_dynamic_range_label(stream).to_string()
    } else {
        "SDR".to_string()
    });
    if original {
        if let Some(bit_rate) = stream.bit_rate {
            parts.push(format!(
                "{}Mbps",
                (bit_rate as f64 / 1_000_000.0).round() as i64
            ));
        }
    }

    parts.join(" ")
}

fn video_resolution_label(height: u32) -> String {
    format!("{height}p")
}

fn video_dynamic_range_label(stream: &Stream) -> &'static str {
    match &stream.details {
        StreamDetails::Video {
            hdr_format: Some(HDRFormat::Hdr10),
            ..
        } => "HDR10",
        StreamDetails::Video {
            hdr_format: Some(HDRFormat::Hdr10Plus),
            ..
        } => "HDR10+",
        StreamDetails::Video {
            hdr_format: Some(HDRFormat::DolbyVision),
            ..
        } => "Dolby Vision",
        StreamDetails::Video {
            hdr_format: Some(HDRFormat::Hlg),
            ..
        } => "HLG",
        StreamDetails::Video {
            hdr_format: Some(HDRFormat::Unknown(_)),
            ..
        } => "HDR",
        _ => "SDR",
    }
}

fn playback_video_codec_label(codec: PlaybackVideoCodec) -> &'static str {
    match codec {
        PlaybackVideoCodec::H264 => "H.264",
        PlaybackVideoCodec::H265 => "H.265",
        PlaybackVideoCodec::Av1 => "AV1",
    }
}

fn format_transcoded_audio_display_info(stream: &Stream) -> String {
    let mut parts = vec!["AAC".to_string()];
    if let Some(channels) = stream.channels() {
        parts.push(format!("{}ch", channels.min(2)));
    }
    parts.push("160kbps".to_string());
    parts.join(" ")
}
