use anyhow::{Context, Result, bail};
use tracing::warn;
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::AtomicI64,
    },
};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    config::TARGET_SEGMENT_SECONDS,
    keyframes,
    model::{StreamDescriptor, StreamInfo, StreamType},
    playlist,
    profiles::{Profile, ProfileContext, ProfileType},
};

pub struct AppState {
    pub master_playlist: String,
    pub stream_profiles: HashMap<StreamProfileKey, Arc<StreamProfileState>>,
}

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
    pub segment_start_pts: Option<Vec<i64>>,
    pub segment_start_seconds: Vec<f64>,
    pub segment_dir: PathBuf,
    pub input: PathBuf,
    pub stream_info: Option<StreamInfo>,
    pub keyframes: Option<Arc<Vec<f64>>>,
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

#[derive(serde::Deserialize)]
struct FfprobeOutput {
    streams: Vec<FfprobeStream>,
    format: Option<FfprobeFormat>,
}

#[derive(serde::Deserialize)]
struct FfprobeStream {
    index: Option<u32>,
    codec_type: Option<String>,
    codec_name: Option<String>,
    time_base: Option<String>,
    tags: Option<HashMap<String, String>>,
}

#[derive(serde::Deserialize)]
struct FfprobeFormat {
    duration: Option<String>,
}

pub fn prepare_segments_root() -> Result<PathBuf> {
    let root = std::env::current_dir()?.join(".segments");
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

pub fn probe_streams(
    input: &Path,
) -> Result<(Vec<StreamDescriptor>, Option<StreamInfo>, f64)> {
    let output = std::process::Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_streams",
            "-show_format",
            "-of",
            "json",
        ])
        .arg(input)
        .output()
        .context("failed to run ffprobe")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("ffprobe failed: {stderr}");
    }

    let parsed: FfprobeOutput =
        serde_json::from_slice(&output.stdout).context("failed to parse ffprobe JSON")?;

    let duration = parsed
        .format
        .as_ref()
        .and_then(|format| format.duration.as_ref())
        .context("missing duration from ffprobe")?;
    let duration_seconds: f64 = duration
        .parse()
        .context("failed to parse duration from ffprobe")?;

    let mut streams: Vec<StreamDescriptor> = Vec::new();
    let mut primary_video_info: Option<StreamInfo> = None;
    let mut seen_primary_video = false;

    for stream in parsed.streams {
        let stream_index = stream.index.context("missing stream index")?;
        let codec_type = match stream.codec_type.as_deref() {
            Some(value) => value,
            None => continue,
        };
        let stream_type = match codec_type {
            "video" => StreamType::Video,
            "audio" => StreamType::Audio,
            "subtitle" => StreamType::Subtitle,
            _ => continue,
        };
        let codec_name = match stream.codec_name {
            Some(value) => value,
            None => {
                warn!(stream_index, codec_type, "stream missing codec_name, skipping");
                continue;
            }
        };

        let is_primary_video = stream_type == StreamType::Video && !seen_primary_video;
        if is_primary_video {
            seen_primary_video = true;
            let time_base = stream
                .time_base
                .as_ref()
                .context("missing time_base for primary video")?;
            let (time_base_num, time_base_den) = parse_time_base(time_base)?;
            primary_video_info = Some(StreamInfo {
                time_base_num,
                time_base_den,
                duration_seconds,
            });
        }

        let language = stream
            .tags
            .as_ref()
            .and_then(|tags| tags.get("language"))
            .cloned();

        streams.push(StreamDescriptor {
            stream_id: stream_index,
            stream_index,
            stream_type,
            codec_name,
            language,
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
    keyframes: Option<Arc<Vec<f64>>>,
    duration_seconds: f64,
    profiles: &[Arc<dyn Profile>],
) -> Result<HashMap<StreamProfileKey, Arc<StreamProfileState>>> {
    let mut map = HashMap::new();

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
                keyframes: if stream.is_primary_video { keyframes.clone() } else { None },
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

            let (playlist, segment_start_pts, segment_start_seconds) = match profile.profile_type() {
                ProfileType::Copy => {
                    let info = ctx
                        .stream_info
                        .as_ref()
                        .context("missing stream info for copy profile")?;
                    let keyframes = ctx
                        .keyframes
                        .as_ref()
                        .context("missing keyframes for copy profile")?;
                    let playlist = playlist::create_fmp4_hls_playlist_from_keyframes_seconds(
                        keyframes,
                        info.duration_seconds,
                        TARGET_SEGMENT_SECONDS,
                        info.time_base_num,
                        info.time_base_den,
                        &endpoint_prefix,
                        "",
                    )
                    .map_err(|err| anyhow::anyhow!(err))?;

                    let (start_pts, start_seconds) = compute_segment_starts_from_keyframes(
                        keyframes,
                        info.duration_seconds,
                        info.time_base_num,
                        info.time_base_den,
                        TARGET_SEGMENT_SECONDS,
                    )?;

                    (playlist, Some(start_pts), start_seconds)
                }
                ProfileType::Transcode => {
                    let playlist = playlist::create_fmp4_hls_playlist_fixed_seconds(
                        duration_seconds,
                        TARGET_SEGMENT_SECONDS,
                        &endpoint_prefix,
                        "",
                    )
                    .map_err(|err| anyhow::anyhow!(err))?;
                    let start_seconds =
                        compute_segment_starts_fixed(duration_seconds, TARGET_SEGMENT_SECONDS);
                    (playlist, None, start_seconds)
                }
            };

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
                segment_start_seconds,
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
        let uri = format!(
            "/stream/{}/{}/index.m3u8",
            stream.stream_id, "audio_aac"
        );
        playlist.push_str(&format!(
            "#EXT-X-MEDIA:TYPE=AUDIO,GROUP-ID=\"audio\",NAME=\"{}\",DEFAULT=NO,AUTOSELECT=YES,LANGUAGE=\"{}\",URI=\"{}\"\n",
            name, language, uri
        ));
    }

    let primary_video = streams
        .iter()
        .find(|stream| stream.is_primary_video && stream.stream_type == StreamType::Video)
        .context("missing primary video stream")?;
    let video_key = StreamProfileKey {
        stream_id: primary_video.stream_id,
        profile_id: "video_copy".to_string(),
    };
    if stream_profiles.contains_key(&video_key) {
        let uri = format!(
            "/stream/{}/{}/index.m3u8",
            primary_video.stream_id, "video_copy"
        );
        let audio_attr = if has_audio { ",AUDIO=\"audio\"" } else { "" };
        playlist.push_str(&format!(
            "#EXT-X-STREAM-INF:BANDWIDTH=8000000{}\n",
            audio_attr
        ));
        playlist.push_str(&format!("{}\n", uri));
    }

    Ok(playlist)
}

fn parse_time_base(value: &str) -> Result<(i64, i64)> {
    let mut parts = value.split('/');
    let num = parts
        .next()
        .context("invalid time_base numerator")?
        .parse::<i64>()
        .context("invalid time_base numerator")?;
    let den = parts
        .next()
        .context("invalid time_base denominator")?
        .parse::<i64>()
        .context("invalid time_base denominator")?;
    if parts.next().is_some() {
        bail!("invalid time_base format: {value}");
    }
    if num <= 0 || den <= 0 {
        bail!("invalid time_base values: {value}");
    }
    Ok((num, den))
}

fn compute_segment_starts_from_keyframes(
    keyframes_seconds: &[f64],
    total_duration_seconds: f64,
    time_base_num: i64,
    time_base_den: i64,
    desired_segment_seconds: f64,
) -> Result<(Vec<i64>, Vec<f64>)> {
    let mut keyframes_pts: Vec<i64> = keyframes_seconds
        .iter()
        .map(|&s| playlist::seconds_to_pts(s, time_base_num, time_base_den))
        .collect();
    keyframes_pts.sort_unstable();
    keyframes_pts.dedup();

    let total_duration_pts =
        playlist::seconds_to_pts(total_duration_seconds, time_base_num, time_base_den);
    let desired_segment_length_pts =
        playlist::seconds_to_pts(desired_segment_seconds, time_base_num, time_base_den);

    let segments_pts = playlist::compute_segments_from_keyframes_pts(
        &keyframes_pts,
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

    let start_seconds = start_pts
        .iter()
        .map(|&pts| (pts as f64) * (time_base_num as f64) / (time_base_den as f64))
        .collect();

    Ok((start_pts, start_seconds))
}

fn compute_segment_starts_fixed(total_duration_seconds: f64, desired_segment_seconds: f64) -> Vec<f64> {
    let mut starts = Vec::new();
    let mut cursor = 0.0f64;
    while cursor < total_duration_seconds {
        starts.push(cursor);
        cursor += desired_segment_seconds;
    }
    starts
}

pub fn load_keyframes_if_needed(input: &Path, has_primary_video: bool) -> Result<Option<Arc<Vec<f64>>>> {
    if has_primary_video {
        Ok(Some(Arc::new(keyframes::load_or_probe_keyframes(input)?)))
    } else {
        Ok(None)
    }
}
