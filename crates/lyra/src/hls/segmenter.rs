use crate::hls::{
    TARGET_DURATION,
    profiles::{ProfileContext, TranscodingProfile},
};
use anyhow::Result;
use easy_ffprobe::Stream;
use nix::{sys::signal::Signal::SIGSTOP, unistd::Pid};
use notify::{EventKind, RecursiveMode, Watcher};
use std::{
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::{
    fs::File,
    sync::{Mutex, Notify},
    task::JoinHandle,
};

const JUMP_SIZE: usize = 5;
const BUFFER_SIZE: usize = 2;

pub struct Segmenter {
    ffmpeg_path: String,
    ffmpeg_handle: Arc<Mutex<Option<FfmpegHandle>>>,
    profile: Arc<Box<dyn TranscodingProfile + Send + Sync>>,
    input_path: PathBuf,
    stream: Stream,
    segment_dir: PathBuf,
    stream_idx: usize,
}

impl Segmenter {
    pub fn new(
        ffmpeg_path: String,
        segment_dir: PathBuf,
        profile: Arc<Box<dyn TranscodingProfile + Send + Sync>>,
        input_path: PathBuf,
        stream: Stream,
        stream_idx: usize,
    ) -> Self {
        std::fs::create_dir_all(&segment_dir).unwrap();
        Self {
            ffmpeg_path,
            profile,
            input_path,
            stream,
            segment_dir,
            stream_idx,
            ffmpeg_handle: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn get_segment(&self, segment_id: usize) -> Result<File> {
        let segment_path = self.segment_dir.join(format!("{}.ts", segment_id));
        if segment_path.exists() {
            let handle = File::open(segment_path).await?;
            return Ok(handle);
        }

        let mut ffmpeg_lock = self.ffmpeg_handle.lock().await;
        let ffmpeg_lock = if let Some(ffmpeg) = ffmpeg_lock.as_mut() {
            let current_segment = ffmpeg.current_segment.load(Ordering::Relaxed);
            if segment_id > current_segment && segment_id - current_segment <= JUMP_SIZE {
                tracing::debug!(
                    "ffmpeg at {} and requested at {}, using existing ffmpeg",
                    current_segment,
                    segment_id
                );
                ffmpeg
            } else {
                tracing::warn!(
                    "moving ffmpeg from segment {} to {}",
                    current_segment,
                    segment_id
                );
                // if the current ffmpeg segment is within JUMP_SIZE of the segment_id, we just wait for it to be written
                // otherwise, we need to kill ffmpeg and restart it at the new position
                let args = self.get_args_for_position(segment_id);
                *ffmpeg = FfmpegHandle::new(
                    self.segment_dir.clone(),
                    self.ffmpeg_path.clone(),
                    args,
                    segment_id,
                );
                ffmpeg
            }
        } else {
            tracing::debug!("no ffmpeg handle, creating new one");
            let args = self.get_args_for_position(segment_id);
            let ffmpeg = FfmpegHandle::new(
                self.segment_dir.clone(),
                self.ffmpeg_path.clone(),
                args,
                segment_id,
            );
            *ffmpeg_lock = Some(ffmpeg);
            ffmpeg_lock.as_mut().unwrap()
        };

        ffmpeg_lock.resume();
        ffmpeg_lock
            .wait_for_segment(segment_id, &segment_path)
            .await?;

        let handle = File::open(segment_path).await?;
        Ok(handle)
    }

    fn get_args_for_position(&self, segment_id: usize) -> Vec<String> {
        let context = ProfileContext {
            input_path: self.input_path.clone(),
            stream: self.stream.clone(),
            outdir: self.segment_dir.clone(),
            segment_idx: segment_id,
            segment_duration: TARGET_DURATION,
            start_time_offset: (segment_id - 1) as f64 * TARGET_DURATION,
            stream_idx: self.stream_idx,
        };

        self.profile.get_args(&context)
    }
}

struct FfmpegHandle {
    handle: Child,
    current_segment: Arc<AtomicUsize>,
    wanted_segment: Arc<AtomicUsize>,
    is_paused: Arc<AtomicBool>,
    watcher_handle: JoinHandle<()>,
    notifier: Arc<Notify>,
}

impl FfmpegHandle {
    fn new(
        segment_dir: PathBuf,
        ffmpeg_path: String,
        ffmpeg_args: Vec<String>,
        wanted_segment: usize,
    ) -> Self {
        let current_segment = Arc::new(AtomicUsize::new(0));
        let wanted_segment = Arc::new(AtomicUsize::new(wanted_segment));
        let notifier = Arc::new(Notify::new());
        let is_paused = Arc::new(AtomicBool::new(false));

        tracing::info!("starting ffmpeg with args: {:?}", ffmpeg_args);
        let handle = Command::new(ffmpeg_path)
            .args(ffmpeg_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        let handle_pid = Pid::from_raw(handle.id() as i32);
        let watcher_handle = tokio::spawn({
            let current_segment = current_segment.clone();
            let wanted_segment = wanted_segment.clone();
            let notifier = notifier.clone();
            let is_paused = is_paused.clone();
            async move {
                let (tx, mut rx) = tokio::sync::mpsc::channel::<notify::Result<notify::Event>>(100);
                let mut watcher = notify::recommended_watcher(move |event| {
                    let _ = tx.blocking_send(event);
                })
                .expect("failed to create watcher");

                watcher
                    .watch(&segment_dir, RecursiveMode::NonRecursive)
                    .expect("failed to watch segment directory");

                while let Some(event) = rx.recv().await {
                    let Ok(event) = event else {
                        tracing::warn!(event = ?event, "segmenter watch error");
                        continue;
                    };

                    // we need either Modify or Create
                    match event.kind {
                        EventKind::Modify(_) => {
                            tracing::debug!(event = ?event, "segment renamed");
                        }
                        EventKind::Create(_) => {
                            tracing::debug!(event = ?event, "segment created");
                        }
                        _ => {
                            tracing::trace!(event = ?event, "ignoring event");
                            continue;
                        }
                    }

                    let Some(file_name) = event
                        .paths
                        .iter()
                        .find(|p| p.extension().unwrap_or_default() == "ts")
                    else {
                        continue;
                    };

                    if file_name.extension().unwrap_or_default() != "ts" {
                        tracing::trace!(event = ?event, "ignoring non-ts file");
                        continue;
                    }

                    let Ok(segment_id) = file_name
                        .file_stem()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .parse::<usize>()
                    else {
                        panic!("failed to parse segment id from file name: {:?}", file_name);
                    };

                    tracing::info!("segment {} created", segment_id);
                    current_segment.store(segment_id, Ordering::Relaxed);
                    notifier.notify_waiters();

                    tracing::debug!(
                        "loaded segment: {}, wanted_segment: {}",
                        segment_id,
                        wanted_segment.load(Ordering::Relaxed)
                    );

                    let wanted_segment = wanted_segment.load(Ordering::Relaxed);
                    if segment_id > wanted_segment + BUFFER_SIZE {
                        tracing::info!(
                            "pausing ffmpeg because segment {} is past wanted segment {}",
                            segment_id,
                            wanted_segment
                        );
                        nix::sys::signal::kill(handle_pid, SIGSTOP).unwrap();
                        is_paused.store(true, Ordering::Relaxed);
                    }
                }
            }
        });

        Self {
            handle,
            current_segment,
            is_paused,
            watcher_handle,
            notifier,
            wanted_segment,
        }
    }

    async fn wait_for_segment(&self, segment_id: usize, segment_path: &PathBuf) -> Result<()> {
        assert!(!segment_path.exists());
        assert!(!self.is_paused.load(Ordering::Relaxed));

        self.wanted_segment.store(segment_id, Ordering::Relaxed);
        let timeout = Duration::from_secs(10);
        loop {
            tokio::select! {
                _ = tokio::time::sleep(timeout) => {
                    break;
                }
                _ = self.notifier.notified() => {
                    tracing::info!("checking for segment {}", segment_id);
                    if segment_path.exists() {
                        return Ok(());
                    }
                }
            }
        }

        return Err(anyhow::anyhow!(
            "segment {} not found after timeout",
            segment_id
        ));
    }

    fn resume(&self) {
        tracing::debug!("resuming ffmpeg");
        let pid = Pid::from_raw(self.handle.id() as i32);
        nix::sys::signal::kill(pid, nix::sys::signal::Signal::SIGCONT).unwrap();
        self.is_paused.store(false, Ordering::Relaxed);
    }
}

impl Drop for FfmpegHandle {
    fn drop(&mut self) {
        self.handle.kill().unwrap();
        self.watcher_handle.abort();
    }
}
