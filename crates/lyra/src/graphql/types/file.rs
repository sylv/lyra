use crate::auth::RequestAuth;
use crate::entities::{
    assets, file_assets::{self, FileAssetRole}, file_probe, file_subtitles, files, users,
};
use crate::graphql::properties::{
    AudioRenditionOption, AudioTrackOption, FileProbe, FileSegment, FileSegmentKind,
    PlaybackOptions, SubtitleRendition, SubtitleRenditionType, SubtitleTrack, TimelinePreviewSheet,
    TrackDispositionPreference, VideoRenditionOption,
};
use crate::hls;
use crate::segment_markers::StoredFileSegmentKind;
use crate::subtitles::{
    disposition_names, subtitle_kind_from_stream,
};
use crate::subtitles::language::{
    SubtitleSelectionCandidate, SubtitleTrackVariant, language_match_strength, select_subtitle_track,
};
use async_graphql::{ComplexObject, Context};
use lyra_packager::{Compatibility, audio_profile, video_profile};
use lyra_probe::{Stream, StreamKind};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use std::collections::HashMap;

#[ComplexObject]
impl files::Model {
    pub async fn probe(&self, ctx: &Context<'_>) -> Result<Option<FileProbe>, sea_orm::DbErr> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let probe = file_probe::Entity::find_by_id(self.id.clone()).one(pool).await?;
        Ok(probe
            .as_ref()
            .and_then(|probe| probe.get_probe().ok())
            .map(|probe| summarize_probe(&probe)))
    }

    pub async fn playback_options(
        &self,
        ctx: &Context<'_>,
    ) -> Result<PlaybackOptions, async_graphql::Error> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let auth = ctx.data::<RequestAuth>()?;
        let user = auth.get_user();
        let (probe_data, keyframes) = hls::load_probe_data_for_playback_options(pool, &self.id)
            .await
            .map_err(|error| async_graphql::Error::new(error.to_string()))?;

        let primary_video = probe_data
            .get_video_stream()
            .ok_or_else(|| async_graphql::Error::new("File has no playable video stream"))?;
        let video_renditions = derive_video_renditions(primary_video, keyframes.as_ref());

        let mut audio_streams: Vec<_> = probe_data
            .streams
            .iter()
            .filter(|stream| stream.kind() == StreamKind::Audio)
            .collect();
        audio_streams.sort_by_key(|stream| stream.index);
        let recommended_audio_manifest_index =
            user.and_then(|user| compute_recommended_audio_track_index(&audio_streams, user));

        let audio_tracks = audio_streams
            .iter()
            .map(|stream| {
                let recommended = recommended_audio_manifest_index
                    .and_then(|index| audio_streams.get(index))
                    .is_some_and(|candidate| candidate.index == stream.index);
                AudioTrackOption {
                    stream_index: stream.index as i32,
                    display_name: stream
                        .display_name
                        .clone()
                        .unwrap_or_else(|| format!("Audio {}", stream.index + 1)),
                    language: stream.language_bcp47.clone(),
                    recommended,
                    renditions: derive_audio_renditions(stream),
                }
            })
            .collect::<Vec<_>>();

        Ok(PlaybackOptions {
            video_renditions,
            audio_tracks,
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

        let asset_ids = rows.iter().map(|row| row.asset_id.clone()).collect::<Vec<_>>();
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

    pub async fn subtitles(
        &self,
        ctx: &Context<'_>,
        language_hints: Option<Vec<String>>,
    ) -> Result<Vec<SubtitleTrack>, async_graphql::Error> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let auth = ctx.data::<RequestAuth>()?;
        let user = auth.get_user();

        let probe_row = file_probe::Entity::find_by_id(self.id.clone())
            .one(pool)
            .await
            .map_err(|error| async_graphql::Error::new(error.to_string()))?;
        let Some(probe_row) = probe_row else {
            return Ok(Vec::new());
        };
        let probe_data = probe_row
            .get_probe()
            .map_err(|error| async_graphql::Error::new(error.to_string()))?;

        let subtitle_rows = load_current_subtitle_rows(pool, self).await?;
        let mut rows_by_stream_index: HashMap<i64, Vec<file_subtitles::Model>> = HashMap::new();
        for row in subtitle_rows {
            rows_by_stream_index.entry(row.stream_index).or_default().push(row);
        }

        let mut audio_streams: Vec<_> = probe_data
            .streams
            .iter()
            .filter(|stream| stream.kind() == StreamKind::Audio)
            .collect();
        audio_streams.sort_by_key(|stream| stream.index);
        let recommended_audio_index = user
            .and_then(|user| compute_recommended_audio_track_index(&audio_streams, user))
            .unwrap_or(0);
        let active_audio_language = audio_streams
            .get(recommended_audio_index)
            .and_then(|stream| stream.language_bcp47.as_deref());

        let mut built_tracks = Vec::new();
        for stream in probe_data
            .streams
            .iter()
            .filter(|stream| stream.kind() == StreamKind::Subtitle)
        {
            let Some(track) = build_logical_subtitle_track(
                &self.id,
                stream,
                rows_by_stream_index.get(&i64::from(stream.index)),
            ) else {
                continue;
            };
            built_tracks.push(track);
        }

        built_tracks.sort_by_key(|track| track.track.stream_index);

        let preferred_subtitle_languages: Vec<String> = user
            .map(|user| {
                serde_json::from_str::<Vec<String>>(&user.preferred_subtitle_languages)
                    .unwrap_or_default()
            })
            .unwrap_or_default();
        let selected_track_id = user.and_then(|user| {
            select_subtitle_track(
                &built_tracks
                    .iter()
                    .map(|track| track.candidate.clone())
                    .collect::<Vec<_>>(),
                user.subtitle_mode,
                &preferred_subtitle_languages,
                &language_hints.unwrap_or_default(),
                user.subtitle_variant_preference,
                active_audio_language,
            )
        });

        Ok(built_tracks
            .into_iter()
            .map(|mut built| {
                built.track.autoselect = selected_track_id.as_deref() == Some(built.track.id.as_str());
                built.track
            })
            .collect())
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
    track: SubtitleTrack,
    candidate: SubtitleSelectionCandidate,
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

fn build_logical_subtitle_track(
    file_id: &str,
    stream: &Stream,
    rows: Option<&Vec<file_subtitles::Model>>,
) -> Option<BuiltSubtitleTrack> {
    let kind = subtitle_kind_from_stream(stream)?;
    let display_name = stream
        .display_name
        .clone()
        .unwrap_or_else(|| format!("Subtitle {}", stream.index + 1));
    let flags = disposition_names(i64::from(stream.disposition.bits()))
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    let variant = track_variant(stream);
    let mut renditions = Vec::new();

    match kind {
        file_subtitles::SubtitleKind::Vtt => renditions.push(SubtitleRendition {
            id: "direct".to_string(),
            codec_name: "WebVTT".to_string(),
            r#type: SubtitleRenditionType::Direct,
            display_info: "WebVTT".to_string(),
            on_demand: rows.is_none_or(|rows| {
                !rows.iter().any(|row| {
                    row.derived_from_subtitle_id.is_none()
                        && row.source == file_subtitles::SubtitleSource::Extracted
                        && row.kind == file_subtitles::SubtitleKind::Vtt
                })
            }),
        }),
        file_subtitles::SubtitleKind::Srt
        | file_subtitles::SubtitleKind::Ass
        | file_subtitles::SubtitleKind::MovText
        | file_subtitles::SubtitleKind::Text
        | file_subtitles::SubtitleKind::Ttml => renditions.push(SubtitleRendition {
            id: "converted".to_string(),
            codec_name: "WebVTT".to_string(),
            r#type: SubtitleRenditionType::Converted,
            display_info: format!("Converted from {}", subtitle_kind_label(kind)),
            on_demand: rows.is_none_or(|rows| {
                !rows.iter().any(|row| {
                    row.source == file_subtitles::SubtitleSource::Converted
                        && row.kind == file_subtitles::SubtitleKind::Vtt
                })
            }),
        }),
        file_subtitles::SubtitleKind::Pgs | file_subtitles::SubtitleKind::VobSub => {
            renditions.push(SubtitleRendition {
                id: "ocr".to_string(),
                codec_name: "WebVTT".to_string(),
                r#type: SubtitleRenditionType::Ocr,
                display_info: format!("Converted from {} using OCR", subtitle_kind_label(kind)),
                on_demand: rows.is_none_or(|rows| {
                    !rows.iter().any(|row| {
                        row.source == file_subtitles::SubtitleSource::Ocr
                            && row.kind == file_subtitles::SubtitleKind::Vtt
                    })
                }),
            });
        }
    }

    if rows.is_some_and(|rows| {
        rows.iter().any(|row| {
            row.source == file_subtitles::SubtitleSource::Generated
                && row.kind == file_subtitles::SubtitleKind::Vtt
        })
    }) {
        renditions.push(SubtitleRendition {
            id: "generated".to_string(),
            codec_name: "WebVTT".to_string(),
            r#type: SubtitleRenditionType::Generated,
            display_info: "Generated WebVTT".to_string(),
            on_demand: false,
        });
    }

    Some(BuiltSubtitleTrack {
        track: SubtitleTrack {
            id: logical_subtitle_track_id(file_id, stream.index),
            stream_index: stream.index as i32,
            display_name,
            language_bcp47: stream.language_bcp47.clone(),
            flags,
            autoselect: false,
            renditions,
        },
        candidate: SubtitleSelectionCandidate {
            id: logical_subtitle_track_id(file_id, stream.index),
            language_bcp47: stream.language_bcp47.clone(),
            variant,
        },
    })
}

pub(crate) fn logical_subtitle_track_id(file_id: &str, stream_index: u32) -> String {
    format!("{file_id}:{stream_index}")
}

pub(crate) fn parse_logical_subtitle_track_id(track_id: &str) -> Option<(&str, u32)> {
    let (file_id, stream_index) = track_id.rsplit_once(':')?;
    let stream_index = stream_index.parse().ok()?;
    Some((file_id, stream_index))
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
) -> Vec<VideoRenditionOption> {
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
                if let Some(codec_tag) = lyra_probe::video_codec_tag(stream) {
                    renditions.push(VideoRenditionOption {
                        rendition_id: "original".to_string(),
                        display_name: stream.codec.to_string().to_uppercase(),
                        display_info: lyra_probe::video_display_info(stream),
                        codec_tag,
                        on_demand: false,
                    });
                }
            }
            "h264" => renditions.push(VideoRenditionOption {
                rendition_id: "h264".to_string(),
                display_name: "H.264".to_string(),
                display_info: "Converted for compatibility".to_string(),
                codec_tag: lyra_probe::TRANSCODED_H264_VIDEO_CODEC_TAG.to_string(),
                on_demand: true,
            }),
            _ => {}
        }
    }

    renditions
}

fn derive_audio_renditions(stream: &Stream) -> Vec<AudioRenditionOption> {
    let mut renditions = Vec::new();

    for profile_id in ["aac"] {
        let Some(profile) = audio_profile(profile_id) else {
            continue;
        };
        if profile.compatible_with(stream).is_none() {
            continue;
        }

        match profile.id() {
            "aac" => renditions.push(AudioRenditionOption {
                rendition_id: "aac".to_string(),
                codec_name: "AAC".to_string(),
                bitrate: Some(160_000),
                channels: stream.channels().map(i32::from),
                sample_rate: match &stream.details {
                    lyra_probe::StreamDetails::Audio { sample_rate, .. } => {
                        sample_rate.map(|value| value as i32)
                    }
                    _ => None,
                },
                codec_tag: lyra_probe::audio_codec_tag(&lyra_probe::Codec::AudioAac)
                    .expect("AAC codec tag must exist")
                    .to_string(),
                on_demand: true,
            }),
            _ => {}
        }
    }

    renditions
}

fn compute_recommended_audio_track_index(
    audio_streams: &[&Stream],
    user: &users::Model,
) -> Option<usize> {
    if audio_streams.is_empty() {
        return None;
    }

    let Some(pref_lang) = user.preferred_audio_language.as_deref() else {
        return Some(0);
    };
    let pref_disp = user
        .preferred_audio_disposition
        .as_deref()
        .and_then(TrackDispositionPreference::from_str);

    let matching = audio_streams
        .iter()
        .enumerate()
        .filter_map(|(index, stream)| {
            let strength = stream
                .language_bcp47
                .as_deref()
                .and_then(|lang| language_match_strength(lang, pref_lang))?;
            Some((index, *stream, strength))
        })
        .collect::<Vec<_>>();

    if matching.is_empty() {
        return Some(0);
    }

    let mut best: Option<((i32, i32), usize)> = None;
    for (index, stream, strength) in matching {
        let disposition_rank = match pref_disp {
            Some(TrackDispositionPreference::Commentary) if stream.is_commentary() => 3,
            Some(TrackDispositionPreference::Sdh)
                if stream.is_hearing_impaired() && !stream.is_commentary() => 3,
            Some(TrackDispositionPreference::Normal)
                if !stream.is_hearing_impaired() && !stream.is_commentary() => 3,
            Some(_) => 0,
            None if !stream.is_hearing_impaired() && !stream.is_commentary() => 3,
            None if stream.is_hearing_impaired() && !stream.is_commentary() => 2,
            None if stream.is_commentary() => 1,
            None => 0,
        };
        let score = (strength as i32, disposition_rank);
        if best.as_ref().is_none_or(|(best_score, _)| score > *best_score) {
            best = Some((score, index));
        }
    }

    Some(best.map(|(_, index)| index).unwrap_or(0))
}
