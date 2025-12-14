use anyhow::{Context, Result};
use serde::Deserialize;
use std::{path::Path, process::Command, time::Duration};

#[derive(Debug, Clone)]
pub struct MediaInfo {
    pub duration: Duration,
    pub video_stream_index: u32,
}

#[derive(Deserialize)]
struct FfprobeOutput {
    format: Format,
    streams: Vec<Stream>,
}

#[derive(Deserialize)]
struct Format {
    duration: Option<String>,
}

#[derive(Deserialize)]
struct Stream {
    codec_type: Option<String>,
    index: u32,
}

pub fn probe(path: impl AsRef<Path>) -> Result<MediaInfo> {
    let path = path.as_ref();
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            path.to_string_lossy().as_ref(),
        ])
        .output()
        .with_context(|| format!("failed to spawn ffprobe for {}", path.display()))?;

    if !output.status.success() {
        anyhow::bail!(
            "ffprobe failed on {}: {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let parsed: FfprobeOutput =
        serde_json::from_slice(&output.stdout).context("failed to parse ffprobe json")?;

    let duration = parsed
        .format
        .duration
        .and_then(|d| d.parse::<f64>().ok())
        .map(Duration::from_secs_f64)
        .unwrap_or_default();

    let video_stream = parsed
        .streams
        .into_iter()
        .find(|stream| {
            stream
                .codec_type
                .as_deref()
                .map(|kind| kind.eq_ignore_ascii_case("video"))
                .unwrap_or(false)
        })
        .unwrap_or(Stream {
            codec_type: None,
            index: 0,
        });

    Ok(MediaInfo {
        duration,
        video_stream_index: video_stream.index,
    })
}
