use crate::{
    ffmpeg::{FfmpegManager, WaitForSegmentError},
    playlist::create_hls_cuts,
    profiles::{Profile, ProfileArgsPosition, ProfileContext, audio_profile, video_profile},
    types::{Compatibility, SessionOptions, SessionSpec},
};
use anyhow::Context;
use lyra_probe::{Stream, VideoKeyframes};
use std::{
    collections::HashSet,
    ffi::OsString,
    path::PathBuf,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

macro_rules! ffarg {
    ($args:ident, $arg:expr) => {{
        $args.push(::std::ffi::OsString::from($arg));
    }};
    ($args:ident, $arg:expr, $value:expr) => {{
        $args.push(::std::ffi::OsString::from($arg));
        $args.push(::std::ffi::OsString::from($value));
    }};
}

const TARGET_SEGMENT_DURATION: Duration = Duration::from_secs(6);

struct ActiveGeneration {
    ffmpeg: FfmpegManager,
}

#[derive(Debug, Clone, Copy)]
struct CompletedRange {
    start_segment: usize,
    end_exclusive: usize,
}

impl CompletedRange {
    fn contains(&self, segment_index: usize) -> bool {
        segment_index >= self.start_segment && segment_index < self.end_exclusive
    }
}

struct SessionState {
    current: Option<ActiveGeneration>,
    completed_ranges: Vec<CompletedRange>,
    shutdown: bool,
}

pub struct Session {
    id: String,
    spec: SessionSpec,
    work_dir: PathBuf,
    keyframes: Option<VideoKeyframes>,
    video_stream: Stream,
    audio_stream: Option<Stream>,
    video_profile: &'static dyn Profile,
    audio_profile: Option<&'static dyn Profile>,
    compatibility: Compatibility,
    state: Mutex<SessionState>,
    player_ids: Mutex<HashSet<String>>,
    last_used: std::sync::Mutex<Instant>,
}

impl Session {
    pub fn new(id: String, work_dir: PathBuf, options: SessionOptions) -> anyhow::Result<Self> {
        let video_stream = options
            .probe
            .video_stream(options.spec.video.stream_index)
            .cloned()
            .with_context(|| {
                format!(
                    "video stream {} not found in probe data",
                    options.spec.video.stream_index
                )
            })?;
        let video_profile = video_profile(&options.spec.video.profile_id).with_context(|| {
            format!(
                "unknown video profile {}",
                options.spec.video.profile_id.as_str()
            )
        })?;
        let compatibility = video_profile
            .compatible_with(&video_stream)
            .with_context(|| {
                format!(
                    "video profile {} is incompatible with stream {}",
                    video_profile.id(),
                    video_stream.index
                )
            })?;

        if compatibility == Compatibility::KeyframeAligned {
            let keyframes = options
                .keyframes
                .as_ref()
                .context("keyframe-aligned sessions require keyframes")?;
            anyhow::ensure!(
                keyframes.video_stream_index == video_stream.index,
                "keyframes are for video stream {}, not {}",
                keyframes.video_stream_index,
                video_stream.index
            );
        }

        let (audio_stream, audio_profile) = match &options.spec.audio {
            Some(selection) => {
                let audio_stream = options
                    .probe
                    .stream(selection.stream_index)
                    .cloned()
                    .with_context(|| {
                        format!(
                            "audio stream {} not found in probe data",
                            selection.stream_index
                        )
                    })?;
                let audio_profile = audio_profile(&selection.profile_id).with_context(|| {
                    format!("unknown audio profile {}", selection.profile_id.as_str())
                })?;
                anyhow::ensure!(
                    audio_profile.compatible_with(&audio_stream).is_some(),
                    "audio profile {} is incompatible with stream {}",
                    audio_profile.id(),
                    audio_stream.index
                );
                (Some(audio_stream), Some(audio_profile))
            }
            None => (None, None),
        };

        Ok(Self {
            id,
            spec: options.spec,
            work_dir,
            keyframes: options.keyframes,
            video_stream,
            audio_stream,
            video_profile,
            audio_profile,
            compatibility,
            state: Mutex::new(SessionState {
                current: None,
                completed_ranges: Vec::new(),
                shutdown: false,
            }),
            player_ids: Mutex::new(HashSet::new()),
            last_used: std::sync::Mutex::new(Instant::now()),
        })
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub fn spec(&self) -> &SessionSpec {
        &self.spec
    }

    pub fn work_dir(&self) -> &std::path::Path {
        self.work_dir.as_path()
    }

    pub fn touch(&self) {
        *self.last_used.lock().expect("last_used mutex poisoned") = Instant::now();
    }

    pub fn is_idle_for(&self, duration: Duration) -> bool {
        self.last_used
            .lock()
            .expect("last_used mutex poisoned")
            .elapsed()
            >= duration
    }

    pub async fn get_init_segment(&self) -> anyhow::Result<PathBuf> {
        self.touch();
        let segment_path = self.get_segment(0).await?;
        let init_path = segment_path.with_file_name("init.mp4");
        debug_assert!(init_path.exists());
        Ok(init_path)
    }

    // The session has to arbitrate between archived ranges and the live ffmpeg process so
    // seeking backwards does not corrupt the live process's request accounting.
    pub async fn get_segment(&self, segment_index: usize) -> anyhow::Result<PathBuf> {
        self.touch();
        let mut state = self.state.lock().await;
        anyhow::ensure!(!state.shutdown, "session has been shut down");

        loop {
            if let Some(current) = state.current.as_mut() {
                if segment_index >= current.ffmpeg.start_segment() {
                    match current.ffmpeg.wait_for_segment(segment_index).await {
                        Ok(()) => {
                            let path = self.segment_path(segment_index);
                            debug_assert!(path.exists());
                            return Ok(path);
                        }
                        Err(WaitForSegmentError::OutOfRange) => {
                            self.archive_current_generation(&mut state).await?;
                            self.create_ffmpeg(&mut state, segment_index).await?;
                            continue;
                        }
                    }
                }
            }

            if state
                .completed_ranges
                .iter()
                .any(|range| range.contains(segment_index))
            {
                let path = self.segment_path(segment_index);
                debug_assert!(path.exists());
                return Ok(path);
            }

            if state.current.is_some() {
                self.archive_current_generation(&mut state).await?;
            }
            self.create_ffmpeg(&mut state, segment_index).await?;
        }
    }

    pub async fn shutdown(&self) -> anyhow::Result<()> {
        self.touch();
        let mut state = self.state.lock().await;
        if state.shutdown {
            return Ok(());
        }

        state.shutdown = true;
        if let Some(mut current) = state.current.take() {
            current.ffmpeg.kill().await;
        }
        drop(state);

        match tokio::fs::remove_dir_all(&self.work_dir).await {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    pub(crate) async fn add_player(&self, player_id: String) {
        self.touch();
        self.player_ids.lock().await.insert(player_id);
    }

    pub(crate) async fn remove_player(&self, player_id: &str) -> bool {
        self.touch();
        let mut player_ids = self.player_ids.lock().await;
        player_ids.remove(player_id);
        player_ids.is_empty()
    }

    async fn archive_current_generation(&self, state: &mut SessionState) -> anyhow::Result<()> {
        let Some(mut current) = state.current.take() else {
            return Ok(());
        };

        let completed = current.ffmpeg.completed_range();
        if completed.end > completed.start {
            register_completed_range(
                &mut state.completed_ranges,
                CompletedRange {
                    start_segment: completed.start,
                    end_exclusive: completed.end,
                },
            );
        }

        current.ffmpeg.kill().await;
        Ok(())
    }

    async fn create_ffmpeg(
        &self,
        state: &mut SessionState,
        start_segment: usize,
    ) -> anyhow::Result<()> {
        let args = self.get_ffmpeg_args(start_segment)?;
        let ffmpeg = FfmpegManager::new(args, start_segment, self.work_dir.clone())?;
        state.current = Some(ActiveGeneration { ffmpeg });
        Ok(())
    }

    fn get_ffmpeg_args(&self, start_segment: usize) -> anyhow::Result<Vec<OsString>> {
        let mut args = Vec::new();
        let video_context_before_input = ProfileContext {
            stream: &self.video_stream,
            keyframes: self.keyframes.as_ref(),
            segment_index: start_segment,
            target_segment_duration: TARGET_SEGMENT_DURATION,
            compatibility: self.compatibility,
            position: ProfileArgsPosition::BeforeInput,
        };
        self.video_profile
            .append_args(&mut args, &video_context_before_input)?;

        ffarg!(args, "-i", self.spec.file_path.clone().into_os_string());
        ffarg!(args, "-map", format!("0:{}", self.video_stream.index));
        let video_context = ProfileContext {
            stream: &self.video_stream,
            keyframes: self.keyframes.as_ref(),
            segment_index: start_segment,
            target_segment_duration: TARGET_SEGMENT_DURATION,
            compatibility: self.compatibility,
            position: ProfileArgsPosition::AfterInput,
        };
        self.video_profile.append_args(&mut args, &video_context)?;

        if let (Some(audio_stream), Some(audio_profile)) = (&self.audio_stream, self.audio_profile)
        {
            ffarg!(args, "-map", format!("0:{}", audio_stream.index));
            let audio_context = ProfileContext {
                stream: audio_stream,
                keyframes: None,
                segment_index: start_segment,
                target_segment_duration: TARGET_SEGMENT_DURATION,
                compatibility: Compatibility::Fixed,
                position: ProfileArgsPosition::AfterInput,
            };
            audio_profile.append_args(&mut args, &audio_context)?;
        }

        ffarg!(args, "-copyts");
        ffarg!(args, "-avoid_negative_ts", "make_non_negative");
        ffarg!(args, "-f", "hls");
        ffarg!(
            args,
            "-hls_time",
            TARGET_SEGMENT_DURATION.as_secs().to_string()
        );
        if let Some(hls_cuts) = self.hls_cuts() {
            ffarg!(args, "-hls_cuts", hls_cuts);
        }
        ffarg!(args, "-start_number", start_segment.to_string());
        ffarg!(args, "-hls_flags", "temp_file");
        ffarg!(args, "-hls_segment_type", "fmp4");
        ffarg!(args, "-hls_segment_filename", "seg%d.m4s");
        ffarg!(args, "-hls_fmp4_init_filename", "init.mp4");
        ffarg!(args, "-hls_segment_options", "movflags=+frag_discont");
        ffarg!(args, "-hls_list_size", "0");
        ffarg!(args, "-y");
        ffarg!(args, "pipe:1");
        Ok(args)
    }

    fn hls_cuts(&self) -> Option<String> {
        match self.compatibility {
            Compatibility::KeyframeAligned => create_hls_cuts(
                self.keyframes
                    .as_ref()
                    .expect("keyframe-aligned sessions always have keyframes"),
                TARGET_SEGMENT_DURATION,
            ),
            Compatibility::Fixed => None,
        }
    }

    fn segment_path(&self, segment_index: usize) -> PathBuf {
        self.work_dir.join(format!("seg{segment_index}.m4s"))
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if let Ok(mut state) = self.state.try_lock() {
            if let Some(current) = state.current.as_mut() {
                current.ffmpeg.start_kill();
            }
        }
        let _ = std::fs::remove_dir_all(&self.work_dir);
    }
}

fn register_completed_range(
    completed_ranges: &mut Vec<CompletedRange>,
    mut new_range: CompletedRange,
) {
    if new_range.end_exclusive <= new_range.start_segment {
        return;
    }

    let mut merged = Vec::with_capacity(completed_ranges.len() + 1);
    let mut inserted = false;

    for range in completed_ranges.iter().copied() {
        if range.end_exclusive < new_range.start_segment {
            merged.push(range);
            continue;
        }

        if new_range.end_exclusive < range.start_segment {
            if !inserted {
                merged.push(new_range);
                inserted = true;
            }
            merged.push(range);
            continue;
        }

        new_range.start_segment = new_range.start_segment.min(range.start_segment);
        new_range.end_exclusive = new_range.end_exclusive.max(range.end_exclusive);
    }

    if !inserted {
        merged.push(new_range);
    }

    *completed_ranges = merged;
}

#[cfg(test)]
mod tests {
    use super::{CompletedRange, register_completed_range};

    #[test]
    fn completed_ranges_merge_overlaps_and_adjacency() {
        let mut ranges = vec![CompletedRange {
            start_segment: 0,
            end_exclusive: 3,
        }];

        register_completed_range(
            &mut ranges,
            CompletedRange {
                start_segment: 3,
                end_exclusive: 5,
            },
        );
        register_completed_range(
            &mut ranges,
            CompletedRange {
                start_segment: 8,
                end_exclusive: 10,
            },
        );
        register_completed_range(
            &mut ranges,
            CompletedRange {
                start_segment: 4,
                end_exclusive: 9,
            },
        );

        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start_segment, 0);
        assert_eq!(ranges[0].end_exclusive, 10);
    }
}
