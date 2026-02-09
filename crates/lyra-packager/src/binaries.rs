use std::{path::PathBuf, sync::OnceLock};

static FFMPEG_BIN: OnceLock<PathBuf> = OnceLock::new();
static FFPROBE_BIN: OnceLock<PathBuf> = OnceLock::new();

pub fn configure_bins(ffmpeg_bin: impl Into<PathBuf>, ffprobe_bin: impl Into<PathBuf>) {
    let _ = FFMPEG_BIN.set(ffmpeg_bin.into());
    let _ = FFPROBE_BIN.set(ffprobe_bin.into());
}

pub fn configured_ffmpeg_bin() -> Option<PathBuf> {
    FFMPEG_BIN.get().cloned()
}

pub fn configured_ffprobe_bin() -> Option<PathBuf> {
    FFPROBE_BIN.get().cloned()
}
