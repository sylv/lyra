use crate::{
    config::TARGET_SEGMENT_SECONDS,
    model::{StreamDescriptor, StreamInfo, StreamType},
    playlist,
    profiles::{PlaylistKind, Profile, ProfileContext, SegmentLayout},
};
use anyhow::{Context, Result};
use lyra_ffprobe::{
    FfprobeOutput, StreamType as ProbeStreamType, probe_streams_from_output as parse_probe_streams,
};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    sync::{Arc, atomic::AtomicI64},
};
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct StreamProfileKey {
    pub stream_id: u32,
    pub profile_id: String,
}

impl PartialEq for StreamProfileKey {
    fn eq(&self, other: &Self) -> bool {
        self.stream_id == other.stream_id && self.profile_id == other.profile_id
    }
}

impl Eq for StreamProfileKey {}

impl Hash for StreamProfileKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.stream_id.hash(state);
        self.profile_id.hash(state);
    }
}

pub struct SegmentDirGuard {
    path: PathBuf,
}

impl Drop for SegmentDirGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

pub struct StreamProfileState {
    pub key: StreamProfileKey,
    pub stream: StreamDescriptor,
    pub profile: Arc<dyn Profile>,
    pub playlist: String,
    pub segment_start_pts: Vec<i64>,
    pub timeline_time_base_num: i64,
    pub timeline_time_base_den: i64,
    pub hls_cuts: Arc<String>,
    pub segment_dir: PathBuf,
    pub input: PathBuf,
    pub stream_info: Option<StreamInfo>,
    pub keyframes: Option<Arc<Vec<i64>>>,
    pub ffmpeg: Mutex<FfmpegState>,
    pub ffmpeg_ops: Mutex<()>,
    pub last_generated: Arc<AtomicI64>,
    _segment_guard: SegmentDirGuard,
}

pub struct FfmpegState {
    pub child: Option<tokio::process::Child>,
    pub pid: Option<u32>,
    pub start_segment: i64,
    pub last_requested_segment: i64,
    pub throttled: bool,
    pub throttle_task: Option<tokio::task::JoinHandle<()>>,
}

impl Default for FfmpegState {
    fn default() -> Self {
        Self {
            child: None,
            pid: None,
            start_segment: 0,
            last_requested_segment: 0,
            throttled: false,
            throttle_task: None,
        }
    }
}

#[derive(Clone)]
struct SegmentTimeline {
    start_pts: Vec<i64>,
    total_duration_pts: i64,
    time_base_num: i64,
    time_base_den: i64,
    hls_cuts: Arc<String>,
}

/// resolve a language tag to the canonical english display name.
/// handles both iso 639-1 ("en") and iso 639-3 ("eng").
pub fn language_to_display_name(tag: &str) -> Option<&'static str> {
    isolang::Language::from_639_3(tag)
        .or_else(|| isolang::Language::from_639_1(tag))
        .map(|l| l.to_name())
}

pub fn build_track_display_name(
    language: Option<&str>,
    title: Option<&str>,
    fallback: &str,
    is_forced: bool,
    is_sdh: bool,
    is_commentary: bool,
) -> String {
    let base = language
        .and_then(language_to_display_name)
        .map(str::to_string)
        .or_else(|| title.map(str::to_string))
        .unwrap_or_else(|| fallback.to_string());

    match (is_forced, is_sdh, is_commentary) {
        (true, _, _) => format!("{base} (Forced)"),
        (_, true, _) => format!("{base} (SDH)"),
        (_, _, true) => format!("{base} (Commentary)"),
        _ => base,
    }
}

pub fn prepare_segments_root_at(root: &Path) -> Result<PathBuf> {
    let root = root.to_path_buf();
    std::fs::create_dir_all(&root)
        .with_context(|| format!("failed to create segments root {}", root.display()))?;
    for entry in std::fs::read_dir(&root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let _ = std::fs::remove_dir_all(&path);
        } else if path.is_file() {
            let _ = std::fs::remove_file(&path);
        }
    }
    Ok(root)
}

pub fn create_process_segment_dir(root: &Path) -> Result<PathBuf> {
    let id = Uuid::new_v4().to_string();
    let dir = root.join(id);
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create process segment dir {}", dir.display()))?;
    Ok(dir)
}

pub fn streams_from_probe_output(
    ffprobe_output: &FfprobeOutput,
) -> Result<(Vec<StreamDescriptor>, Option<StreamInfo>, f64)> {
    let parsed = parse_probe_streams(ffprobe_output)?;

    let duration_seconds = parsed
        .duration_seconds
        .context("missing duration from ffprobe")?;

    let mut streams: Vec<StreamDescriptor> = Vec::new();
    let mut primary_video_info: Option<StreamInfo> = None;
    let mut seen_primary_video = false;

    for stream in parsed.streams {
        let stream_index = stream.index;
        let stream_type = match stream.stream_type {
            ProbeStreamType::Video => StreamType::Video,
            ProbeStreamType::Audio => StreamType::Audio,
            ProbeStreamType::Subtitle => StreamType::Subtitle,
            ProbeStreamType::Other(_) => continue,
        };

        let codec_name = match stream.codec_name {
            Some(value) => value,
            None => {
                tracing::warn!(stream_index, "stream missing codec_name, skipping");
                continue;
            }
        };

        let is_primary_video = stream_type == StreamType::Video && !seen_primary_video;
        if is_primary_video {
            seen_primary_video = true;
            let time_base = stream
                .time_base
                .context("missing time_base for primary video")?;
            primary_video_info = Some(StreamInfo {
                time_base_num: time_base.num,
                time_base_den: time_base.den,
                duration_seconds,
            });
        }

        let fallback = match stream_type {
            StreamType::Audio => format!(
                "Audio {}",
                streams
                    .iter()
                    .filter(|s: &&StreamDescriptor| s.stream_type == StreamType::Audio)
                    .count()
                    + 1
            ),
            StreamType::Subtitle => format!(
                "Subtitle {}",
                streams
                    .iter()
                    .filter(|s: &&StreamDescriptor| s.stream_type == StreamType::Subtitle)
                    .count()
                    + 1
            ),
            StreamType::Video => format!(
                "Video {}",
                streams
                    .iter()
                    .filter(|s: &&StreamDescriptor| s.stream_type == StreamType::Video)
                    .count()
                    + 1
            ),
        };
        let display_name = build_track_display_name(
            stream.language.as_deref(),
            stream.title.as_deref(),
            &fallback,
            stream.is_forced,
            stream.is_hearing_impaired,
            stream.is_commentary,
        );

        streams.push(StreamDescriptor {
            stream_id: stream_index,
            stream_index,
            stream_type,
            codec_name,
            bit_rate: stream.bit_rate,
            frame_rate: parse_stream_frame_rate(stream.avg_frame_rate.as_deref())
                .or_else(|| parse_stream_frame_rate(stream.r_frame_rate.as_deref())),
            width: stream.width,
            height: stream.height,
            channels: stream.channels,
            language: stream.language,
            is_primary_video,
            is_forced: stream.is_forced,
            is_sdh: stream.is_hearing_impaired,
            is_commentary: stream.is_commentary,
            display_name,
        });
    }

    Ok((streams, primary_video_info, duration_seconds))
}

pub fn build_stream_profiles(
    input: &PathBuf,
    process_dir: &Path,
    streams: &[StreamDescriptor],
    primary_video_info: Option<&StreamInfo>,
    keyframes: Option<Arc<Vec<i64>>>,
    duration_seconds: f64,
    profiles: &[Arc<dyn Profile>],
) -> Result<HashMap<StreamProfileKey, Arc<StreamProfileState>>> {
    let mut map = HashMap::new();
    let keyframe_timeline = match (primary_video_info, keyframes.as_ref()) {
        (Some(info), Some(video_keyframes)) => {
            let total_duration_pts = playlist::seconds_to_pts(
                info.duration_seconds,
                info.time_base_num,
                info.time_base_den,
            );
            let desired_segment_length_pts = playlist::seconds_to_pts(
                TARGET_SEGMENT_SECONDS,
                info.time_base_num,
                info.time_base_den,
            );
            let start_pts = compute_segment_starts_from_keyframes_pts(
                video_keyframes,
                total_duration_pts,
                desired_segment_length_pts,
            )?;
            let hls_cuts = build_hls_cuts_arg(&start_pts, info.time_base_num, info.time_base_den);
            Some(SegmentTimeline {
                start_pts,
                total_duration_pts,
                time_base_num: info.time_base_num,
                time_base_den: info.time_base_den,
                hls_cuts: Arc::new(hls_cuts),
            })
        }
        _ => None,
    };
    let fixed_timeline = {
        let time_base_num = 1;
        let time_base_den = 1_000_000;
        let total_duration_pts =
            playlist::seconds_to_pts(duration_seconds, time_base_num, time_base_den);
        let desired_segment_length_pts =
            playlist::seconds_to_pts(TARGET_SEGMENT_SECONDS, time_base_num, time_base_den);
        let start_pts =
            compute_segment_starts_fixed_pts(total_duration_pts, desired_segment_length_pts);
        let hls_cuts = build_hls_cuts_arg(&start_pts, time_base_num, time_base_den);
        SegmentTimeline {
            start_pts,
            total_duration_pts,
            time_base_num,
            time_base_den,
            hls_cuts: Arc::new(hls_cuts),
        }
    };
    let single_timeline = {
        let time_base_num = 1;
        let time_base_den = 1_000_000;
        let total_duration_pts =
            playlist::seconds_to_pts(duration_seconds, time_base_num, time_base_den);
        SegmentTimeline {
            start_pts: vec![0],
            total_duration_pts,
            time_base_num,
            time_base_den,
            hls_cuts: Arc::new(String::new()),
        }
    };

    for stream in streams {
        for profile in profiles {
            let ctx = ProfileContext {
                input: input.clone(),
                stream: stream.clone(),
                stream_info: if stream.is_primary_video {
                    primary_video_info.cloned()
                } else {
                    None
                },
                keyframes: if stream.is_primary_video {
                    keyframes.clone()
                } else {
                    None
                },
            };

            if !profile.supports_stream(&ctx) {
                continue;
            }

            let segment_dir = process_dir
                .join(stream.stream_id.to_string())
                .join(profile.id_name());
            std::fs::create_dir_all(&segment_dir).with_context(|| {
                format!("failed to create segment dir {}", segment_dir.display())
            })?;

            let endpoint_prefix = format!(
                "/stream/{}/{}/segment/",
                stream.stream_id,
                profile.id_name()
            );

            let timeline = match profile.segment_layout() {
                SegmentLayout::Keyframe => {
                    let Some(timeline) = keyframe_timeline.as_ref() else {
                        tracing::warn!(
                            stream_id = stream.stream_id,
                            profile = profile.id_name(),
                            "keyframe timeline unavailable; disabling keyframe-dependent profile"
                        );
                        continue;
                    };
                    timeline
                }
                SegmentLayout::Fixed => &fixed_timeline,
                SegmentLayout::Single => &single_timeline,
            };

            let (
                playlist,
                segment_start_pts,
                timeline_time_base_num,
                timeline_time_base_den,
                hls_cuts,
            ) = build_stream_profile_playlist(timeline, &endpoint_prefix, profile.playlist_kind())?;

            let key = StreamProfileKey {
                stream_id: stream.stream_id,
                profile_id: profile.id_name().to_string(),
            };

            let state = StreamProfileState {
                key: key.clone(),
                stream: stream.clone(),
                profile: profile.clone(),
                playlist,
                segment_start_pts,
                timeline_time_base_num,
                timeline_time_base_den,
                hls_cuts,
                segment_dir: segment_dir.clone(),
                input: input.clone(),
                stream_info: ctx.stream_info,
                keyframes: ctx.keyframes,
                ffmpeg: Mutex::new(FfmpegState::default()),
                ffmpeg_ops: Mutex::new(()),
                last_generated: Arc::new(AtomicI64::new(-1)),
                _segment_guard: SegmentDirGuard { path: segment_dir },
            };

            map.insert(key, Arc::new(state));
        }
    }

    Ok(map)
}

fn build_stream_profile_playlist(
    timeline: &SegmentTimeline,
    endpoint_prefix: &str,
    playlist_kind: PlaylistKind,
) -> Result<(String, Vec<i64>, i64, i64, Arc<String>)> {
    let playlist = match playlist_kind {
        PlaylistKind::Fmp4 => playlist::create_fmp4_hls_playlist_from_segment_starts_pts(
            &timeline.start_pts,
            timeline.total_duration_pts,
            timeline.time_base_num,
            timeline.time_base_den,
            endpoint_prefix,
            "",
        ),
        PlaylistKind::WebVtt => playlist::create_webvtt_hls_playlist_from_segment_starts_pts(
            &timeline.start_pts,
            timeline.total_duration_pts,
            timeline.time_base_num,
            timeline.time_base_den,
            endpoint_prefix,
            "",
        ),
    }
    .map_err(|err| anyhow::anyhow!(err))?;
    Ok((
        playlist,
        timeline.start_pts.clone(),
        timeline.time_base_num,
        timeline.time_base_den,
        Arc::clone(&timeline.hls_cuts),
    ))
}

pub fn build_master_playlist(
    stream_profiles: &HashMap<StreamProfileKey, Arc<StreamProfileState>>,
    streams: &[StreamDescriptor],
) -> Result<String> {
    let mut playlist = String::new();
    playlist.push_str("#EXTM3U\n");
    playlist.push_str("#EXT-X-VERSION:7\n");

    let mut audio_renditions: Vec<&StreamDescriptor> = streams
        .iter()
        .filter(|stream| stream.stream_type == StreamType::Audio)
        .collect();
    audio_renditions.sort_by_key(|stream| stream.stream_id);

    let mut has_audio = false;
    for stream in &audio_renditions {
        let key = StreamProfileKey {
            stream_id: stream.stream_id,
            profile_id: "audio_aac".to_string(),
        };
        if !stream_profiles.contains_key(&key) {
            continue;
        }
        has_audio = true;
        let name = stream.display_name.clone();
        let language = stream.language.as_deref().unwrap_or("und");
        let uri = format!("/stream/{}/{}/index.m3u8", stream.stream_id, "audio_aac");
        let mut media_attrs = vec![
            "TYPE=AUDIO".to_string(),
            "GROUP-ID=\"audio\"".to_string(),
            format!("NAME=\"{}\"", name),
            "DEFAULT=NO".to_string(),
            "AUTOSELECT=YES".to_string(),
            format!("LANGUAGE=\"{}\"", language),
            format!("URI=\"{}\"", uri),
        ];
        if let Some(channels) = stream.channels {
            media_attrs.push(format!("CHANNELS=\"{}\"", channels));
        }
        playlist.push_str(&format!("#EXT-X-MEDIA:{}\n", media_attrs.join(",")));
    }

    let mut subtitle_renditions: Vec<&StreamDescriptor> = streams
        .iter()
        .filter(|stream| stream.stream_type == StreamType::Subtitle)
        .collect();
    subtitle_renditions.sort_by_key(|stream| stream.stream_id);

    let mut has_subtitles = false;
    for stream in &subtitle_renditions {
        let key = StreamProfileKey {
            stream_id: stream.stream_id,
            profile_id: "subtitle_webvtt".to_string(),
        };
        if !stream_profiles.contains_key(&key) {
            continue;
        }
        has_subtitles = true;
        let name = stream.display_name.clone();
        let uri = format!(
            "/stream/{}/{}/index.m3u8",
            stream.stream_id, "subtitle_webvtt"
        );
        let forced_attr = if stream.is_forced {
            "FORCED=YES"
        } else {
            "FORCED=NO"
        };
        let mut media_attrs = vec![
            "TYPE=SUBTITLES".to_string(),
            "GROUP-ID=\"subs\"".to_string(),
            format!("NAME=\"{}\"", name),
            "DEFAULT=NO".to_string(),
            "AUTOSELECT=YES".to_string(),
            forced_attr.to_string(),
            format!("URI=\"{}\"", uri),
        ];
        if let Some(language) = stream.language.as_deref() {
            media_attrs.push(format!("LANGUAGE=\"{}\"", language));
        }
        playlist.push_str(&format!("#EXT-X-MEDIA:{}\n", media_attrs.join(",")));
    }

    let primary_video = streams
        .iter()
        .find(|stream| stream.is_primary_video && stream.stream_type == StreamType::Video)
        .context("missing primary video stream")?;

    let mut video_profiles: Vec<&StreamProfileState> = stream_profiles
        .values()
        .filter(|state| {
            state.stream.stream_id == primary_video.stream_id
                && state.profile.stream_type() == StreamType::Video
        })
        .map(|state| state.as_ref())
        .collect();

    video_profiles.sort_by(|a, b| {
        let a_type = a.profile.profile_type();
        let b_type = b.profile.profile_type();
        a_type
            .cmp(&b_type)
            .then_with(|| a.profile.id_name().cmp(b.profile.id_name()))
    });

    for profile in video_profiles {
        let uri = format!(
            "/stream/{}/{}/index.m3u8",
            profile.stream.stream_id,
            profile.profile.id_name()
        );
        let mut attrs: Vec<String> = Vec::new();

        if let Some(bandwidth) = estimate_video_profile_bandwidth(profile) {
            attrs.push(format!("BANDWIDTH={bandwidth}"));
            attrs.push(format!("AVERAGE-BANDWIDTH={bandwidth}"));
        }

        if let Some(frame_rate) = profile.stream.frame_rate {
            attrs.push(format!("FRAME-RATE={:.3}", frame_rate));
        }

        if let (Some(width), Some(height)) = (profile.stream.width, profile.stream.height) {
            attrs.push(format!("RESOLUTION={}x{}", width, height));
        }

        if has_audio {
            attrs.push("AUDIO=\"audio\"".to_string());
        }
        if has_subtitles {
            attrs.push("SUBTITLES=\"subs\"".to_string());
        }

        playlist.push_str(&format!("#EXT-X-STREAM-INF:{}\n", attrs.join(",")));
        playlist.push_str(&format!("{}\n", uri));
    }

    Ok(playlist)
}

fn estimate_video_profile_bandwidth(profile: &StreamProfileState) -> Option<u64> {
    let source_bitrate = profile.stream.bit_rate?;

    match profile.profile.id_name() {
        "video_copy" => Some(source_bitrate),
        "video_h264" => {
            if profile.stream.codec_name.eq_ignore_ascii_case("h264") {
                Some(scale_bitrate(source_bitrate, 3, 2))
            } else {
                Some(scale_bitrate(source_bitrate, 2, 1))
            }
        }
        _ => None,
    }
}

fn scale_bitrate(value: u64, numerator: u64, denominator: u64) -> u64 {
    value.saturating_mul(numerator) / denominator
}

fn parse_stream_frame_rate(value: Option<&str>) -> Option<f64> {
    let raw = value?;
    if raw.is_empty() || raw == "0/0" {
        return None;
    }
    if let Some((num, den)) = raw.split_once('/') {
        let num = num.parse::<f64>().ok()?;
        let den = den.parse::<f64>().ok()?;
        if den <= 0.0 {
            return None;
        }
        let rate = num / den;
        if rate > 0.0 && rate.is_finite() {
            return Some(rate);
        }
        return None;
    }
    let rate = raw.parse::<f64>().ok()?;
    if rate > 0.0 && rate.is_finite() {
        Some(rate)
    } else {
        None
    }
}

fn compute_segment_starts_from_keyframes_pts(
    keyframes_pts: &[i64],
    total_duration_pts: i64,
    desired_segment_length_pts: i64,
) -> Result<Vec<i64>> {
    let segments_pts = playlist::compute_segments_from_keyframes_pts(
        keyframes_pts,
        total_duration_pts,
        desired_segment_length_pts,
    )
    .map_err(|err| anyhow::anyhow!(err))?;

    let mut start_pts = Vec::with_capacity(segments_pts.len());
    let mut cursor = 0i64;
    for len in segments_pts {
        start_pts.push(cursor);
        cursor += len;
    }

    Ok(start_pts)
}

fn compute_segment_starts_fixed_pts(
    total_duration_pts: i64,
    desired_segment_length_pts: i64,
) -> Vec<i64> {
    let mut starts = Vec::new();
    let mut cursor = 0i64;
    while cursor < total_duration_pts {
        starts.push(cursor);
        cursor += desired_segment_length_pts;
    }
    starts
}

fn pts_to_av_time(pts: i64, time_base_num: i64, time_base_den: i64) -> i64 {
    let num = pts as i128 * time_base_num as i128 * 1_000_000i128;
    let den = time_base_den as i128;
    (num / den) as i64
}

fn build_hls_cuts_arg(start_pts: &[i64], time_base_num: i64, time_base_den: i64) -> String {
    let mut cuts = String::new();
    for (i, &start) in start_pts.iter().enumerate() {
        if i > 0 {
            cuts.push(',');
        }
        cuts.push_str(&pts_to_av_time(start, time_base_num, time_base_den).to_string());
    }
    cuts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profiles::{AudioAacProfile, SubtitleWebVttProfile, VideoCopyProfile};
    use std::sync::atomic::AtomicI64;

    fn sample_stream(
        stream_id: u32,
        stream_type: StreamType,
        language: Option<&str>,
        is_primary_video: bool,
    ) -> StreamDescriptor {
        let display_name = language
            .and_then(language_to_display_name)
            .map(str::to_string)
            .unwrap_or_else(|| match stream_type {
                StreamType::Audio => "Audio 1".to_string(),
                StreamType::Subtitle => "Subtitle 1".to_string(),
                StreamType::Video => "Video 1".to_string(),
            });
        StreamDescriptor {
            stream_id,
            stream_index: stream_id,
            stream_type,
            codec_name: match stream_type {
                StreamType::Video => "h264".to_string(),
                StreamType::Audio => "aac".to_string(),
                StreamType::Subtitle => "subrip".to_string(),
            },
            bit_rate: Some(1_000_000),
            frame_rate: Some(24.0),
            width: Some(1920),
            height: Some(1080),
            channels: Some(2),
            language: language.map(str::to_string),
            is_primary_video,
            is_forced: false,
            is_sdh: false,
            is_commentary: false,
            display_name,
        }
    }

    fn sample_state(
        stream: StreamDescriptor,
        profile: Arc<dyn Profile>,
    ) -> Arc<StreamProfileState> {
        Arc::new(StreamProfileState {
            key: StreamProfileKey {
                stream_id: stream.stream_id,
                profile_id: profile.id_name().to_string(),
            },
            stream,
            profile,
            playlist: String::new(),
            segment_start_pts: vec![0],
            timeline_time_base_num: 1,
            timeline_time_base_den: 1_000_000,
            hls_cuts: Arc::new(String::new()),
            segment_dir: PathBuf::new(),
            input: PathBuf::new(),
            stream_info: None,
            keyframes: None,
            ffmpeg: Mutex::new(FfmpegState::default()),
            ffmpeg_ops: Mutex::new(()),
            last_generated: Arc::new(AtomicI64::new(-1)),
            _segment_guard: SegmentDirGuard {
                path: PathBuf::new(),
            },
        })
    }

    #[test]
    fn master_playlist_includes_subtitle_group() {
        let video_stream = sample_stream(0, StreamType::Video, None, true);
        let audio_stream = sample_stream(1, StreamType::Audio, Some("en"), false);
        let subtitle_stream = sample_stream(2, StreamType::Subtitle, Some("en"), false);
        let streams = vec![
            video_stream.clone(),
            audio_stream.clone(),
            subtitle_stream.clone(),
        ];

        let mut profiles = HashMap::new();
        profiles.insert(
            StreamProfileKey {
                stream_id: video_stream.stream_id,
                profile_id: "video_copy".to_string(),
            },
            sample_state(video_stream, Arc::new(VideoCopyProfile)),
        );
        profiles.insert(
            StreamProfileKey {
                stream_id: audio_stream.stream_id,
                profile_id: "audio_aac".to_string(),
            },
            sample_state(audio_stream, Arc::new(AudioAacProfile)),
        );
        profiles.insert(
            StreamProfileKey {
                stream_id: subtitle_stream.stream_id,
                profile_id: "subtitle_webvtt".to_string(),
            },
            sample_state(subtitle_stream, Arc::new(SubtitleWebVttProfile)),
        );

        let playlist = build_master_playlist(&profiles, &streams).expect("playlist should build");

        assert!(playlist.contains("TYPE=SUBTITLES"));
        assert!(playlist.contains("GROUP-ID=\"subs\""));
        assert!(playlist.contains("URI=\"/stream/2/subtitle_webvtt/index.m3u8\""));
        assert!(playlist.contains("SUBTITLES=\"subs\""));
    }

    #[test]
    fn build_stream_profiles_creates_single_segment_subtitle_playlist() {
        let root = std::env::temp_dir().join(format!("lyra-packager-test-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&root).expect("temp root should exist");

        let stream = sample_stream(7, StreamType::Subtitle, Some("en"), false);
        let profiles = vec![Arc::new(SubtitleWebVttProfile) as Arc<dyn Profile>];
        let states = build_stream_profiles(
            &PathBuf::from("/tmp/input.mkv"),
            &root,
            std::slice::from_ref(&stream),
            None,
            None,
            120.0,
            &profiles,
        )
        .expect("subtitle profiles should build");

        let key = StreamProfileKey {
            stream_id: stream.stream_id,
            profile_id: "subtitle_webvtt".to_string(),
        };
        let state = states.get(&key).expect("subtitle state should exist");

        assert_eq!(state.segment_start_pts, vec![0]);
        assert!(state.playlist.contains("#EXTINF:120.000000,"));
        assert!(
            state
                .playlist
                .contains("/stream/7/subtitle_webvtt/segment/0.vtt?startPts=0")
        );

        let _ = std::fs::remove_dir_all(&root);
    }
}
