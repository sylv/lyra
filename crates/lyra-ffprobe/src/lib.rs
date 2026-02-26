use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path, process::Command};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StreamType {
    Video,
    Audio,
    Subtitle,
    Other(String),
}

#[derive(Clone, Debug)]
pub struct TimeBase {
    pub num: i64,
    pub den: i64,
}

#[derive(Clone, Debug)]
pub struct Stream {
    pub index: u32,
    pub stream_type: StreamType,
    pub codec_name: Option<String>,
    pub time_base: Option<TimeBase>,
    pub language: Option<String>,
    pub bit_rate: Option<u64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub channels: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channel_layout: Option<String>,
    pub profile: Option<String>,
    pub level: Option<i64>,
    pub codec_tag_string: Option<String>,
    pub codec_tag: Option<String>,
    pub pix_fmt: Option<String>,
    pub color_range: Option<String>,
    pub color_space: Option<String>,
    pub color_transfer: Option<String>,
    pub color_primaries: Option<String>,
    pub chroma_location: Option<String>,
    pub bits_per_raw_sample: Option<u32>,
    pub is_avc: Option<bool>,
    pub nal_length_size: Option<u32>,
    pub avg_frame_rate: Option<String>,
    pub r_frame_rate: Option<String>,
    pub extradata: Option<String>,
    pub extradata_size: Option<u32>,
}

#[derive(Clone, Debug)]
pub struct ProbeResult {
    pub streams: Vec<Stream>,
    pub duration_seconds: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FfprobeOutput {
    #[serde(default)]
    pub streams: Vec<FfprobeStream>,
    #[serde(default)]
    pub format: Option<FfprobeFormat>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FfprobeStream {
    #[serde(default)]
    pub index: Option<u32>,
    #[serde(default)]
    pub codec_type: Option<String>,
    #[serde(default)]
    pub codec_name: Option<String>,
    #[serde(default)]
    pub profile: Option<String>,
    #[serde(default)]
    pub level: Option<i64>,
    #[serde(default)]
    pub codec_tag_string: Option<String>,
    #[serde(default)]
    pub codec_tag: Option<String>,
    #[serde(default)]
    pub time_base: Option<String>,
    #[serde(default)]
    pub bit_rate: Option<String>,
    #[serde(default)]
    pub width: Option<i64>,
    #[serde(default)]
    pub height: Option<i64>,
    #[serde(default)]
    pub channels: Option<i64>,
    #[serde(default)]
    pub sample_rate: Option<String>,
    #[serde(default)]
    pub channel_layout: Option<String>,
    #[serde(default)]
    pub pix_fmt: Option<String>,
    #[serde(default)]
    pub color_range: Option<String>,
    #[serde(default)]
    pub color_space: Option<String>,
    #[serde(default)]
    pub color_transfer: Option<String>,
    #[serde(default)]
    pub color_primaries: Option<String>,
    #[serde(default)]
    pub chroma_location: Option<String>,
    #[serde(default)]
    pub bits_per_raw_sample: Option<String>,
    #[serde(default)]
    pub is_avc: Option<String>,
    #[serde(default)]
    pub nal_length_size: Option<String>,
    #[serde(default)]
    pub avg_frame_rate: Option<String>,
    #[serde(default)]
    pub r_frame_rate: Option<String>,
    #[serde(default)]
    pub extradata: Option<String>,
    #[serde(default)]
    pub extradata_size: Option<i64>,
    #[serde(default)]
    pub tags: Option<HashMap<String, String>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FfprobeFormat {
    #[serde(default)]
    pub duration: Option<String>,
    #[serde(default)]
    pub bit_rate: Option<String>,
}

#[derive(Deserialize)]
struct ProbeFrames {
    frames: Vec<ProbeFrame>,
}

#[derive(Deserialize)]
struct ProbeFrame {
    best_effort_timestamp: Option<i64>,
    pkt_pts: Option<i64>,
}

pub fn probe_streams(ffprobe_bin: &Path, input: &Path) -> Result<ProbeResult> {
    let parsed = probe_output(ffprobe_bin, input)?;
    probe_streams_from_output(&parsed)
}

pub fn probe_output(ffprobe_bin: &Path, input: &Path) -> Result<FfprobeOutput> {
    let output = Command::new(ffprobe_bin)
        .args([
            "-v",
            "error",
            "-show_streams",
            "-show_format",
            "-show_data",
            "-of",
            "json",
        ])
        .arg(input)
        .output()
        .with_context(|| format!("failed to run ffprobe with {}", ffprobe_bin.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("ffprobe failed: {stderr}");
    }

    serde_json::from_slice(&output.stdout).context("failed to parse ffprobe JSON")
}

pub fn probe_streams_from_output(parsed: &FfprobeOutput) -> Result<ProbeResult> {
    let duration_seconds = parsed
        .format
        .as_ref()
        .and_then(|format| format.duration.as_ref())
        .map(|value| {
            value
                .parse::<f64>()
                .context("failed to parse duration from ffprobe")
        })
        .transpose()?;

    let streams = parsed
        .streams
        .iter()
        .filter_map(|stream| {
            let index = stream.index?;
            let codec_type = stream.codec_type.as_ref()?;
            let stream_type = match codec_type.as_str() {
                "video" => StreamType::Video,
                "audio" => StreamType::Audio,
                "subtitle" => StreamType::Subtitle,
                _ => StreamType::Other(codec_type.to_string()),
            };

            let time_base = stream
                .time_base
                .as_deref()
                .and_then(|value| parse_time_base(value).ok());
            let language = stream
                .tags
                .as_ref()
                .and_then(|tags| tags.get("language"))
                .cloned();

            Some(Stream {
                index,
                stream_type,
                codec_name: stream.codec_name.clone(),
                time_base,
                language,
                bit_rate: parse_optional_u64(stream.bit_rate.as_deref()),
                width: parse_optional_u32_from_i64(stream.width),
                height: parse_optional_u32_from_i64(stream.height),
                channels: parse_optional_u32_from_i64(stream.channels),
                sample_rate: parse_optional_u32(stream.sample_rate.as_deref()),
                channel_layout: stream.channel_layout.clone(),
                profile: stream.profile.clone(),
                level: stream.level,
                codec_tag_string: stream.codec_tag_string.clone(),
                codec_tag: stream.codec_tag.clone(),
                pix_fmt: stream.pix_fmt.clone(),
                color_range: stream.color_range.clone(),
                color_space: stream.color_space.clone(),
                color_transfer: stream.color_transfer.clone(),
                color_primaries: stream.color_primaries.clone(),
                chroma_location: stream.chroma_location.clone(),
                bits_per_raw_sample: parse_optional_u32(stream.bits_per_raw_sample.as_deref()),
                is_avc: parse_optional_bool(stream.is_avc.as_deref()),
                nal_length_size: parse_optional_u32(stream.nal_length_size.as_deref()),
                avg_frame_rate: stream.avg_frame_rate.clone(),
                r_frame_rate: stream.r_frame_rate.clone(),
                extradata: stream.extradata.clone(),
                extradata_size: parse_optional_u32_from_i64(stream.extradata_size),
            })
        })
        .collect();

    Ok(ProbeResult {
        streams,
        duration_seconds,
    })
}

pub fn probe_keyframes_pts(ffprobe_bin: &Path, input: &Path) -> Result<Vec<i64>> {
    let output = Command::new(ffprobe_bin)
        .args([
            "-fflags",
            "+genpts",
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-skip_frame",
            "nokey",
            "-show_frames",
            "-show_entries",
            "frame=best_effort_timestamp,pkt_pts",
            "-of",
            "json",
        ])
        .arg(input)
        .output()
        .with_context(|| {
            format!(
                "failed to run ffprobe for keyframes with {}",
                ffprobe_bin.display()
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("ffprobe keyframe scan failed: {stderr}");
    }

    let frames: ProbeFrames =
        serde_json::from_slice(&output.stdout).context("failed to parse ffprobe keyframes JSON")?;

    let mut times = Vec::new();
    for frame in frames.frames {
        if let Some(value) = frame.best_effort_timestamp.or(frame.pkt_pts) {
            times.push(value);
        }
    }

    times.sort_unstable();
    times.dedup();
    Ok(times)
}

fn parse_time_base(value: &str) -> Result<TimeBase> {
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
    Ok(TimeBase { num, den })
}

fn parse_optional_u64(value: Option<&str>) -> Option<u64> {
    value.and_then(|raw| raw.parse::<u64>().ok())
}

fn parse_optional_u32(value: Option<&str>) -> Option<u32> {
    value.and_then(|raw| raw.parse::<u32>().ok())
}

fn parse_optional_u32_from_i64(value: Option<i64>) -> Option<u32> {
    value.and_then(|raw| u32::try_from(raw).ok())
}

fn parse_optional_bool(value: Option<&str>) -> Option<bool> {
    value.and_then(|raw| match raw {
        "1" | "true" => Some(true),
        "0" | "false" => Some(false),
        _ => None,
    })
}
