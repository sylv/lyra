use lyra_probe::get_ffmpeg_path;
use std::{
    ffi::OsString,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize},
    },
};
use tokio::{
    io::AsyncRead,
    io::{AsyncBufReadExt, BufReader},
    sync::Notify,
};

pub(crate) const BUFFER_AHEAD_SEGMENTS: usize = 1;
pub(crate) const MAX_REQUEST_AHEAD: usize = 4;

pub(crate) struct FfmpegManager {
    process: tokio::process::Child,
    current_generating_segment: Arc<AtomicUsize>,
    last_requested_segment: Arc<AtomicUsize>,
    start_segment: usize,
    watcher_handle: tokio::task::JoinHandle<()>,
    segment_notify: Arc<Notify>,
    is_paused: Arc<AtomicBool>,
}

impl FfmpegManager {
    pub fn new(
        args: Vec<OsString>,
        start_segment: usize,
        work_dir: PathBuf,
    ) -> anyhow::Result<Self> {
        let ffmpeg_bin = get_ffmpeg_path();
        let mut command = tokio::process::Command::new(ffmpeg_bin);
        command.current_dir(work_dir);
        command.args(args);
        let mut process = command
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|error| anyhow::anyhow!("failed to spawn ffmpeg process: {error}"))?;

        let current_generating_segment = Arc::new(AtomicUsize::new(start_segment));
        let segment_notify = Arc::new(Notify::new());
        let is_paused = Arc::new(AtomicBool::new(false));
        let last_requested_segment = Arc::new(AtomicUsize::new(start_segment));
        let stdout = process
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("failed to capture ffmpeg stdout"))?;
        let stderr = process
            .stderr
            .take()
            .ok_or_else(|| anyhow::anyhow!("failed to capture ffmpeg stderr"))?;

        let watcher_handle = spawn_watcher(
            stdout,
            stderr,
            current_generating_segment.clone(),
            segment_notify.clone(),
            is_paused.clone(),
            last_requested_segment.clone(),
            process.id().unwrap_or_default() as i32,
        );

        Ok(Self {
            process,
            current_generating_segment,
            last_requested_segment,
            start_segment,
            watcher_handle,
            segment_notify,
            is_paused,
        })
    }

    pub fn start_segment(&self) -> usize {
        self.start_segment
    }

    pub fn current_generating_segment(&self) -> usize {
        self.current_generating_segment
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn completed_range(&self) -> std::ops::Range<usize> {
        self.start_segment..self.current_generating_segment()
    }

    pub async fn wait_for_segment(&self, target_segment: usize) -> Result<(), WaitForSegmentError> {
        if target_segment < self.start_segment {
            return Err(WaitForSegmentError::OutOfRange);
        }

        let current_generating = self.current_generating_segment();
        if target_segment
            > current_generating
                .saturating_sub(1)
                .saturating_add(MAX_REQUEST_AHEAD)
        {
            return Err(WaitForSegmentError::OutOfRange);
        }

        self.last_requested_segment
            .fetch_max(target_segment, std::sync::atomic::Ordering::SeqCst);

        if target_segment < current_generating {
            return Ok(());
        }

        if self.is_paused.load(std::sync::atomic::Ordering::SeqCst) {
            if let Some(pid) = self.process.id() {
                resume(&self.is_paused, pid as i32);
            }
        }

        loop {
            let current_generating = self.current_generating_segment();
            if target_segment < current_generating {
                break;
            }

            self.segment_notify.notified().await;
        }

        Ok(())
    }

    pub async fn kill(&mut self) {
        let _ = self.process.kill().await;
        self.watcher_handle.abort();
    }

    pub fn start_kill(&mut self) {
        let _ = self.process.start_kill();
        self.watcher_handle.abort();
    }
}

fn spawn_watcher(
    stdout: tokio::process::ChildStdout,
    stderr: tokio::process::ChildStderr,
    current_generating_segment: Arc<AtomicUsize>,
    segment_notify: Arc<Notify>,
    is_paused: Arc<AtomicBool>,
    last_requested_segment: Arc<AtomicUsize>,
    pid: i32,
) -> tokio::task::JoinHandle<()> {
    tokio::task::spawn(async move {
        let stdout_task = watch_ffmpeg_stream(
            stdout,
            Some(StreamWatcherState {
                current_generating_segment,
                segment_notify,
                is_paused,
                last_requested_segment,
                pid,
            }),
        );
        let stderr_task = watch_ffmpeg_stream(stderr, None);

        let _ = tokio::join!(stdout_task, stderr_task);
    })
}

struct StreamWatcherState {
    current_generating_segment: Arc<AtomicUsize>,
    segment_notify: Arc<Notify>,
    is_paused: Arc<AtomicBool>,
    last_requested_segment: Arc<AtomicUsize>,
    pid: i32,
}

// Stdout carries the generated playlist entries we need for segment tracking, while
// stderr is just ffmpeg diagnostics. Both get logged uniformly through tracing.
async fn watch_ffmpeg_stream<R>(stream: R, state: Option<StreamWatcherState>)
where
    R: AsyncRead + Unpin,
{
    let mut lines = BufReader::new(stream).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        tracing::debug!("ffmpeg: {line}");

        let Some(state) = state.as_ref() else {
            continue;
        };

        if let Some(last_generated_segment) = parse_generated_segment(&line) {
            state.current_generating_segment.store(
                last_generated_segment + 1,
                std::sync::atomic::Ordering::SeqCst,
            );
            state.segment_notify.notify_waiters();

            let last_requested = state
                .last_requested_segment
                .load(std::sync::atomic::Ordering::SeqCst);
            let dist_to_last_requested = last_generated_segment.saturating_sub(last_requested);
            if dist_to_last_requested > BUFFER_AHEAD_SEGMENTS {
                pause(&state.is_paused, state.pid);
            }
        }
    }
}

fn parse_generated_segment(line: &str) -> Option<usize> {
    let stripped = line
        .strip_prefix("seg")
        .and_then(|value| value.strip_suffix(".m4s"))?;
    stripped.parse::<usize>().ok()
}

impl Drop for FfmpegManager {
    fn drop(&mut self) {
        self.start_kill();
    }
}

fn pause(is_paused: &Arc<AtomicBool>, pid: i32) {
    if pid == 0 {
        return;
    }

    unsafe {
        libc::kill(pid, libc::SIGSTOP);
    }
    is_paused.store(true, std::sync::atomic::Ordering::SeqCst);
}

fn resume(is_paused: &Arc<AtomicBool>, pid: i32) {
    if pid == 0 {
        return;
    }

    unsafe {
        libc::kill(pid, libc::SIGCONT);
    }
    is_paused.store(false, std::sync::atomic::Ordering::SeqCst);
}

#[derive(thiserror::Error, Debug)]
pub enum WaitForSegmentError {
    #[error("segment out of range for this ffmpeg process")]
    OutOfRange,
}
