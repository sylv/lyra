use crate::{
    config::BuildOptions,
    ffmpeg::{
        ensure_ffmpeg_for_init, ensure_ffmpeg_for_segment, parse_segment_index, wait_for_file,
    },
    profiles::{AudioAacProfile, Profile, VideoCopyProfile, VideoH264Profile},
    state::{
        build_master_playlist, build_stream_profiles, create_process_segment_dir,
        prepare_segments_root_at, streams_from_probe_output,
    },
};
use anyhow::{Context, Result, bail};
use lyra_ffprobe::{
    FfprobeOutput, paths::get_ffprobe_path, probe_keyframes_pts_blocking, probe_output_blocking,
};
use std::{
    collections::HashMap,
    path::{Path as FsPath, PathBuf},
    sync::Arc,
    time::Duration,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SessionKey {
    pub stream_id: u32,
    pub profile_id: String,
}

#[derive(Clone)]
pub struct Session {
    inner: Arc<crate::state::StreamProfileState>,
}

pub struct Package {
    master_playlist: String,
    sessions: HashMap<SessionKey, Session>,
}

pub fn get_profiles() -> Vec<Arc<dyn Profile>> {
    vec![
        Arc::new(VideoCopyProfile),
        Arc::new(VideoH264Profile),
        Arc::new(AudioAacProfile),
    ]
}

pub fn canonicalize_input_path(path: impl AsRef<FsPath>) -> Result<PathBuf> {
    let path = path.as_ref();
    if !path.exists() {
        bail!("input file does not exist: {}", path.display());
    }
    let canonical = std::fs::canonicalize(path)
        .with_context(|| format!("failed to canonicalize input path {}", path.display()))?;
    Ok(canonical)
}

pub fn parse_input_path_from_args<I>(mut args: I, usage_program: &str) -> Result<PathBuf>
where
    I: Iterator<Item = String>,
{
    let input = args
        .next()
        .with_context(|| format!("usage: {usage_program} <input-file>"))?;
    if args.next().is_some() {
        bail!("only a single input file is supported");
    }
    canonicalize_input_path(input)
}

pub fn parse_single_input_path_arg() -> Result<PathBuf> {
    let mut args = std::env::args();
    let program = args.next().unwrap_or_else(|| "lyra-packager".to_string());
    parse_input_path_from_args(args, &program)
}

pub fn build_package(
    input: &FsPath,
    profiles: &[Arc<dyn Profile>],
    options: &BuildOptions,
    ffprobe_output: &FfprobeOutput,
    keyframes_pts: &[i64],
) -> Result<Package> {
    let input = canonicalize_input_path(input)?;

    let segments_root = prepare_segments_root_at(&options.transcode_cache_dir)?;
    let process_dir = create_process_segment_dir(&segments_root)?;

    let (streams, primary_video_info, duration_seconds) =
        streams_from_probe_output(ffprobe_output)?;
    let keyframes = if primary_video_info.is_some() && !keyframes_pts.is_empty() {
        Some(Arc::new(keyframes_pts.to_vec()))
    } else {
        if primary_video_info.is_some() {
            tracing::warn!(
                input = %input.display(),
                "keyframe data is empty; keyframe-dependent profiles will be disabled"
            );
        }
        None
    };

    let stream_profiles = build_stream_profiles(
        &input,
        &process_dir,
        &streams,
        primary_video_info.as_ref(),
        keyframes,
        duration_seconds,
        profiles,
    )?;

    let master_playlist = build_master_playlist(&stream_profiles, &streams)?;
    let mut sessions = HashMap::new();
    for (key, value) in stream_profiles {
        sessions.insert(
            SessionKey {
                stream_id: key.stream_id,
                profile_id: key.profile_id,
            },
            Session { inner: value },
        );
    }

    Ok(Package {
        master_playlist,
        sessions,
    })
}

pub fn build_package_with_defaults(input: &FsPath, options: &BuildOptions) -> Result<Package> {
    let input = canonicalize_input_path(input)?;
    let profiles = get_profiles();
    let ffprobe_bin = PathBuf::from(get_ffprobe_path()?);
    let ffprobe_output = probe_output_blocking(&ffprobe_bin, &input)?;
    let keyframes = probe_keyframes_pts_blocking(&ffprobe_bin, &input)?;
    build_package(&input, &profiles, options, &ffprobe_output, &keyframes)
}

impl Package {
    pub fn master_playlist(&self) -> &str {
        &self.master_playlist
    }

    pub fn sessions(&self) -> &HashMap<SessionKey, Session> {
        &self.sessions
    }

    pub fn get_session(&self, stream_id: u32, profile_id: &str) -> Option<&Session> {
        let key = SessionKey {
            stream_id,
            profile_id: profile_id.to_string(),
        };
        self.sessions.get(&key)
    }
}

impl Session {
    pub fn stream_id(&self) -> u32 {
        self.inner.stream.stream_id
    }

    pub fn profile_id(&self) -> &str {
        &self.inner.key.profile_id
    }

    pub fn key(&self) -> SessionKey {
        SessionKey {
            stream_id: self.stream_id(),
            profile_id: self.profile_id().to_string(),
        }
    }

    pub fn playlist(&self) -> &str {
        &self.inner.playlist
    }

    pub fn segment_count(&self) -> usize {
        self.inner.segment_start_pts.len()
    }

    pub fn has_segment(&self, segment_index: i64) -> bool {
        segment_index >= 0 && (segment_index as usize) < self.inner.segment_start_pts.len()
    }

    pub fn segment_path(&self, name: &str) -> PathBuf {
        self.inner.segment_dir.join(name)
    }

    pub fn parse_segment_name(name: &str) -> Option<i64> {
        parse_segment_index(name)
    }

    pub async fn ensure_init(&self) -> Result<()> {
        ensure_ffmpeg_for_init(&self.inner).await
    }

    pub async fn ensure_segment(
        &self,
        segment_index: i64,
        requested_start_pts: Option<i64>,
    ) -> Result<()> {
        if !self.has_segment(segment_index) {
            bail!("segment {segment_index} out of range");
        }
        ensure_ffmpeg_for_segment(&self.inner, segment_index, requested_start_pts).await
    }

    pub async fn wait_for_segment_file(&self, name: &str, timeout: Duration) -> Result<PathBuf> {
        let path = self.segment_path(name);
        wait_for_file(&path, timeout).await?;
        Ok(path)
    }
}
