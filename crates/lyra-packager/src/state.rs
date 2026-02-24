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
use tracing::warn;
use uuid::Uuid;

use crate::{
    config::TARGET_SEGMENT_SECONDS,
    model::{StreamDescriptor, StreamInfo, StreamType},
    playlist,
    profiles::{Profile, ProfileContext, ProfileType, SegmentLayout},
};

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

pub fn prepare_segments_root() -> Result<PathBuf> {
    let root = std::env::current_dir()?.join(".segments");
    prepare_segments_root_at(&root)
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
                warn!(stream_index, "stream missing codec_name, skipping");
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

        streams.push(StreamDescriptor {
            stream_id: stream_index,
            stream_index,
            stream_type,
            codec_name,
            language: stream.language,
            is_primary_video,
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
                        warn!(
                            stream_id = stream.stream_id,
                            profile = profile.id_name(),
                            "keyframe timeline unavailable; disabling keyframe-dependent profile"
                        );
                        continue;
                    };
                    timeline
                }
                SegmentLayout::Fixed => &fixed_timeline,
            };

            let (
                playlist,
                segment_start_pts,
                timeline_time_base_num,
                timeline_time_base_den,
                hls_cuts,
            ) = build_stream_profile_playlist(timeline, &endpoint_prefix)?;

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
) -> Result<(String, Vec<i64>, i64, i64, Arc<String>)> {
    let playlist = playlist::create_fmp4_hls_playlist_from_segment_starts_pts(
        &timeline.start_pts,
        timeline.total_duration_pts,
        timeline.time_base_num,
        timeline.time_base_den,
        endpoint_prefix,
        "",
    )
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
        let name = stream
            .language
            .clone()
            .unwrap_or_else(|| format!("Audio {}", stream.stream_id));
        let language = stream.language.as_deref().unwrap_or("und");
        let uri = format!("/stream/{}/{}/index.m3u8", stream.stream_id, "audio_aac");
        playlist.push_str(&format!(
            "#EXT-X-MEDIA:TYPE=AUDIO,GROUP-ID=\"audio\",NAME=\"{}\",DEFAULT=NO,AUTOSELECT=YES,LANGUAGE=\"{}\",URI=\"{}\"\n",
            name, language, uri
        ));
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
        let audio_attr = if has_audio { ",AUDIO=\"audio\"" } else { "" };
        let bandwidth = match profile.profile.profile_type() {
            ProfileType::Copy => 8_000_000,
            ProfileType::Transcode => 4_000_000,
        };
        playlist.push_str(&format!(
            "#EXT-X-STREAM-INF:BANDWIDTH={}{}\n",
            bandwidth, audio_attr
        ));
        playlist.push_str(&format!("{}\n", uri));
    }

    Ok(playlist)
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
