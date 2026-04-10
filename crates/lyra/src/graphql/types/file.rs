use crate::auth::RequestAuth;
use crate::entities::{
    assets,
    file_assets::{self, FileAssetRole},
    file_probe, file_subtitles, files, users,
};
use crate::graphql::properties::{
    AudioRenditionOption, AudioTrackOption, FileSegment, FileSegmentKind, PlaybackOptions,
    SubtitleKind, SubtitlePlaybackTrack, SubtitleRenditionOption, SubtitleSource, SubtitleTrack,
    TimelinePreviewSheet, TrackDispositionPreference, VideoRenditionOption,
};
use crate::hls;
use crate::segment_markers::StoredFileSegmentKind;
use crate::subtitles::disposition_names;
use async_graphql::{ComplexObject, Context};
use lyra_packager::{Compatibility, audio_profile, video_profile};
use lyra_probe::{Stream, StreamKind};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use std::collections::HashMap;

#[ComplexObject]
impl files::Model {
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

        let subtitle_rows = self.subtitle_tracks(ctx).await?;
        let recommended_subtitle_id = self.recommended_subtitle_track_id(ctx).await?;
        let subtitle_tracks = subtitle_rows
            .into_iter()
            .map(|track| SubtitlePlaybackTrack {
                subtitle_id: track.id.clone(),
                stream_index: track.stream_index,
                display_name: track.label,
                language: track.language,
                recommended: recommended_subtitle_id.as_deref() == Some(track.id.as_str()),
                renditions: vec![SubtitleRenditionOption {
                    rendition_id: "webvtt".to_string(),
                    codec_name: "WebVTT".to_string(),
                    on_demand: false,
                }],
            })
            .collect();

        Ok(PlaybackOptions {
            video_renditions,
            audio_tracks,
            subtitle_tracks,
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

    pub async fn subtitle_tracks(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<SubtitleTrack>, async_graphql::Error> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();

        let latest_seen_at = self.subtitles_extracted_at;
        let Some(latest_seen_at) = latest_seen_at else {
            return Ok(Vec::new());
        };

        let rows = file_subtitles::Entity::find()
            .filter(file_subtitles::Column::FileId.eq(self.id.clone()))
            .filter(file_subtitles::Column::LastSeenAt.eq(latest_seen_at))
            .all(pool)
            .await
            .map_err(|error| async_graphql::Error::new(error.to_string()))?;

        let assets_by_id = assets::Entity::find()
            .filter(
                assets::Column::Id.is_in(
                    rows.iter()
                        .map(|row| row.asset_id.clone())
                        .collect::<Vec<_>>(),
                ),
            )
            .all(pool)
            .await
            .map_err(|error| async_graphql::Error::new(error.to_string()))?
            .into_iter()
            .map(|asset| (asset.id.clone(), asset))
            .collect::<HashMap<_, _>>();

        let playable_rows = rows
            .iter()
            .filter(|row| row.kind == file_subtitles::SubtitleKind::Vtt)
            .collect::<Vec<_>>();

        let mut tracks = Vec::new();
        for row in playable_rows {
            let Some(asset) = assets_by_id.get(&row.asset_id) else {
                continue;
            };

            tracks.push(SubtitleTrack {
                id: row.id.clone(),
                stream_index: row.stream_index as i32,
                kind: subtitle_kind_from_model(row.kind),
                source: subtitle_source_from_model(row.source),
                label: row
                    .display_name
                    .clone()
                    .unwrap_or_else(|| format!("Subtitle {}", row.stream_index + 1)),
                language: row.language_bcp47.clone(),
                dispositions: disposition_names(row.disposition_bits)
                    .into_iter()
                    .map(str::to_string)
                    .collect(),
                asset: asset.clone().into(),
                derived_from_subtitle_id: row.derived_from_subtitle_id.clone(),
            });
        }

        tracks.sort_by(|a, b| {
            a.stream_index
                .cmp(&b.stream_index)
                .then_with(|| a.label.cmp(&b.label))
        });
        Ok(tracks)
    }

    pub async fn recommended_subtitle_track_id(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<String>, async_graphql::Error> {
        let pool = ctx.data_unchecked::<DatabaseConnection>();
        let auth = ctx.data::<RequestAuth>()?;
        let Some(user) = auth.get_user() else {
            return Ok(None);
        };

        let probe = file_probe::Entity::find_by_id(self.id.clone())
            .one(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        let Some(probe) = probe else {
            return Ok(None);
        };
        let probe_data = match probe.get_probe() {
            Ok(v) => v,
            Err(_) => return Ok(None),
        };

        let mut audio_streams: Vec<_> = probe_data
            .streams
            .iter()
            .filter(|s| s.kind() == StreamKind::Audio)
            .collect();
        audio_streams.sort_by_key(|s| s.index);

        let subtitle_rows = self.subtitle_tracks(ctx).await?;
        Ok(recommend_subtitle_track_id(
            &audio_streams,
            &subtitle_rows,
            user,
        ))
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

fn normalize_lang(tag: &str) -> Option<isolang::Language> {
    isolang::Language::from_639_3(tag).or_else(|| isolang::Language::from_639_1(tag))
}

fn langs_match(a: &str, b: &str) -> bool {
    match (normalize_lang(a), normalize_lang(b)) {
        (Some(la), Some(lb)) => la == lb,
        _ => false,
    }
}

fn compute_recommended_audio_track_index(
    audio_streams: &[&Stream],
    user: &users::Model,
) -> Option<usize> {
    if audio_streams.is_empty() {
        None
    } else if let Some(ref pref_lang) = user.preferred_audio_language {
        let pref_disp = user
            .preferred_audio_disposition
            .as_deref()
            .and_then(TrackDispositionPreference::from_str);

        // find all tracks matching the preferred language
        let matching: Vec<(usize, &&Stream)> = audio_streams
            .iter()
            .enumerate()
            .filter(|(_, s)| {
                s.language_bcp47
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
                TrackDispositionPreference::Commentary => s.is_commentary(),
                TrackDispositionPreference::Sdh => s.is_hearing_impaired() && !s.is_commentary(),
                TrackDispositionPreference::Normal => {
                    !s.is_hearing_impaired() && !s.is_commentary()
                }
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
                .find(|(_, s)| !s.is_hearing_impaired() && !s.is_commentary())
                .or_else(|| {
                    matching
                        .iter()
                        .find(|(_, s)| s.is_hearing_impaired() && !s.is_commentary())
                })
                .or_else(|| matching.iter().find(|(_, s)| s.is_commentary()))
                .or_else(|| matching.first());
            Some(pick.map(|(i, _)| *i).unwrap_or(0))
        }
    } else {
        Some(0usize)
    }
}

fn recommend_subtitle_track_id(
    audio_streams: &[&Stream],
    subtitle_tracks: &[SubtitleTrack],
    user: &users::Model,
) -> Option<String> {
    let active_audio_lang = user.preferred_audio_language.clone().or_else(|| {
        audio_streams
            .first()
            .and_then(|stream| stream.language_bcp47.clone())
    });

    let forced_match = subtitle_tracks.iter().find(|track| {
        let is_forced = track.dispositions.iter().any(|tag| tag == "Forced");
        if !is_forced {
            return false;
        }
        match (&track.language, &active_audio_lang) {
            (Some(sub_lang), Some(audio_lang)) => langs_match(sub_lang, audio_lang),
            _ => false,
        }
    });
    if let Some(track) = forced_match {
        return Some(track.id.clone());
    }

    let non_forced = subtitle_tracks
        .iter()
        .filter(|track| !track.dispositions.iter().any(|tag| tag == "Forced"))
        .collect::<Vec<_>>();
    let pref_lang = user.preferred_subtitle_language.as_deref()?;
    let pref_disp = user
        .preferred_subtitle_disposition
        .as_deref()
        .and_then(TrackDispositionPreference::from_str);

    let matching = non_forced
        .into_iter()
        .filter(|track| {
            track
                .language
                .as_deref()
                .map(|lang| langs_match(lang, pref_lang))
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    if matching.is_empty() {
        return None;
    }

    let best = if let Some(disp) = pref_disp {
        matching
            .iter()
            .find(|track| subtitle_track_matches_preference(track, disp))
    } else {
        matching
            .iter()
            .find(|track| {
                subtitle_track_matches_preference(track, TrackDispositionPreference::Normal)
            })
            .or_else(|| {
                matching.iter().find(|track| {
                    subtitle_track_matches_preference(track, TrackDispositionPreference::Sdh)
                })
            })
            .or_else(|| {
                matching.iter().find(|track| {
                    subtitle_track_matches_preference(track, TrackDispositionPreference::Commentary)
                })
            })
            .or_else(|| matching.first())
    };

    best.map(|track| track.id.clone())
}

fn subtitle_track_matches_preference(
    track: &SubtitleTrack,
    preference: TrackDispositionPreference,
) -> bool {
    let has_sdh = track.dispositions.iter().any(|tag| tag == "SDH");
    let has_commentary = track.dispositions.iter().any(|tag| tag == "Commentary");
    match preference {
        TrackDispositionPreference::Commentary => has_commentary,
        TrackDispositionPreference::Sdh => has_sdh && !has_commentary,
        TrackDispositionPreference::Normal => !has_sdh && !has_commentary,
    }
}

fn subtitle_source_from_model(source: file_subtitles::SubtitleSource) -> SubtitleSource {
    match source {
        file_subtitles::SubtitleSource::Extracted => SubtitleSource::Extracted,
        file_subtitles::SubtitleSource::Converted => SubtitleSource::Converted,
        file_subtitles::SubtitleSource::Ocr => SubtitleSource::Ocr,
        file_subtitles::SubtitleSource::Generated => SubtitleSource::Generated,
    }
}

fn subtitle_kind_from_model(kind: file_subtitles::SubtitleKind) -> SubtitleKind {
    match kind {
        file_subtitles::SubtitleKind::Srt => SubtitleKind::Srt,
        file_subtitles::SubtitleKind::Vtt => SubtitleKind::Vtt,
        file_subtitles::SubtitleKind::Ass => SubtitleKind::Ass,
        file_subtitles::SubtitleKind::MovText => SubtitleKind::MovText,
        file_subtitles::SubtitleKind::Text => SubtitleKind::Text,
        file_subtitles::SubtitleKind::Ttml => SubtitleKind::Ttml,
        file_subtitles::SubtitleKind::Pgs => SubtitleKind::Pgs,
        file_subtitles::SubtitleKind::VobSub => SubtitleKind::VobSub,
    }
}
