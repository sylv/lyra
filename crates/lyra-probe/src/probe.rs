use crate::{PROBE_ZSTD_DICTIONARY, paths::get_ffprobe_path, types::*};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::{
    collections::HashMap,
    io::{Cursor, Read, Write},
    path::Path,
    process::Command as StdCommand,
};
use tokio_util::sync::CancellationToken;

const PROBE_ZSTD_LEVEL: i32 = 12;

#[derive(Deserialize)]
struct FfprobeOutput {
    streams: Vec<FfprobeStream>,
    format: FfprobeFormat,
}

#[derive(Deserialize)]
struct FfprobeFormat {
    duration: Option<String>,
    bit_rate: Option<String>,
}

#[derive(Deserialize)]
struct FfprobeStream {
    index: u32,
    codec_name: Option<String>,
    codec_type: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    channels: Option<u16>,
    sample_rate: Option<String>,
    pix_fmt: Option<String>,
    color_transfer: Option<String>,
    color_space: Option<String>,
    r_frame_rate: Option<String>,
    bit_rate: Option<String>,
    time_base: Option<String>,
    #[serde(default)]
    disposition: FfprobeDisposition,
    #[serde(default)]
    tags: HashMap<String, String>,
    side_data_list: Option<Vec<FfprobeSideData>>,
}

#[derive(Deserialize, Default)]
struct FfprobeDisposition {
    #[serde(default)]
    default: i32,
    #[serde(default)]
    forced: i32,
    #[serde(default)]
    comment: i32,
    #[serde(default)]
    hearing_impaired: i32,
    #[serde(default)]
    visual_impaired: i32,
    #[serde(default)]
    original: i32,
    #[serde(default)]
    dub: i32,
}

#[derive(Deserialize)]
struct FfprobeSideData {
    side_data_type: Option<String>,
}

pub async fn probe(file_path: &Path) -> Result<ProbeData> {
    probe_with_cancellation(file_path, None)
        .await?
        .ok_or_else(|| anyhow::anyhow!("probe was cancelled"))
}

pub fn probe_blocking(file_path: &Path) -> Result<ProbeData> {
    let ffprobe_bin = get_ffprobe_path();
    let output = StdCommand::new(ffprobe_bin)
        .args(ffprobe_args())
        .arg(file_path)
        .output()
        .context("failed to execute ffprobe")?;

    parse_ffprobe_output(
        output.status.success(),
        output.status.code(),
        &output.stdout,
    )
}

pub async fn probe_with_cancellation(
    file_path: &Path,
    cancellation_token: Option<&CancellationToken>,
) -> Result<Option<ProbeData>> {
    let ffprobe_bin = get_ffprobe_path();
    let cancellation_token = cancellation_token
        .cloned()
        .unwrap_or_else(CancellationToken::new);

    let cmd = tokio::process::Command::new(ffprobe_bin)
        .args(ffprobe_args())
        .arg(file_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .context("failed to spawn ffprobe process")?;

    let wait = cmd.wait_with_output();
    tokio::pin!(wait);

    let output = tokio::select! {
        output = &mut wait => output.context("failed to wait for ffprobe process")?,
        _ = cancellation_token.cancelled() => {
            return Ok(None);
        }
    };

    parse_ffprobe_output(
        output.status.success(),
        output.status.code(),
        &output.stdout,
    )
    .map(Some)
}

pub fn encode_probe_data_json_zstd(value: &ProbeData) -> Result<Vec<u8>> {
    let json = serde_json::to_vec(value).context("failed to serialize probe JSON")?;
    let mut encoder =
        zstd::stream::Encoder::with_dictionary(Vec::new(), PROBE_ZSTD_LEVEL, PROBE_ZSTD_DICTIONARY)
            .context("failed to initialize zstd encoder for probe JSON")?;
    encoder
        .write_all(&json)
        .context("failed to write probe JSON into zstd encoder")?;
    encoder
        .finish()
        .context("failed to finalize probe JSON zstd payload")
}

pub fn decode_probe_data_json_zstd(payload: &[u8]) -> Result<ProbeData> {
    let decoded = decode_probe_payload(payload)?;
    serde_json::from_slice(&decoded).context("failed to parse cached probe JSON payload")
}

fn decode_probe_payload(payload: &[u8]) -> Result<Vec<u8>> {
    let mut decoded = Vec::new();
    if let Ok(mut decoder) =
        zstd::stream::Decoder::with_dictionary(Cursor::new(payload), PROBE_ZSTD_DICTIONARY)
    {
        decoder
            .read_to_end(&mut decoded)
            .context("failed to decode dictionary-compressed probe payload")?;
        return Ok(decoded);
    }

    decoded.clear();
    if let Ok(mut decoder) = zstd::stream::Decoder::new(Cursor::new(payload)) {
        decoder
            .read_to_end(&mut decoded)
            .context("failed to decode probe payload")?;
        return Ok(decoded);
    }

    Ok(payload.to_vec())
}

fn ffprobe_args() -> [&'static str; 6] {
    [
        "-v",
        "error",
        "-show_format",
        "-show_streams",
        "-of",
        "json",
    ]
}

fn parse_ffprobe_output(success: bool, exit_code: Option<i32>, stdout: &[u8]) -> Result<ProbeData> {
    if !success {
        return Err(anyhow::anyhow!(
            "ffprobe failed with exit code {}",
            exit_code.unwrap_or(-1)
        ));
    }

    let raw: FfprobeOutput =
        serde_json::from_slice(stdout).context("failed to parse ffprobe JSON output")?;

    convert(raw)
}

fn convert(raw: FfprobeOutput) -> Result<ProbeData> {
    let duration_secs = raw
        .format
        .duration
        .as_deref()
        .and_then(|s| s.parse::<f64>().ok());

    let overall_bit_rate = raw
        .format
        .bit_rate
        .as_deref()
        .and_then(|s| s.parse::<u64>().ok());

    let streams = raw
        .streams
        .into_iter()
        .filter_map(|s| match convert_stream(s) {
            Ok(stream) => Some(stream),
            Err(e) => {
                tracing::debug!("skipping stream: {e}");
                None
            }
        })
        .collect();

    Ok(ProbeData {
        duration_secs,
        overall_bit_rate,
        streams,
    })
}

fn convert_stream(raw: FfprobeStream) -> Result<Stream> {
    let codec_name = raw.codec_name.unwrap_or_default();

    let kind = match raw.codec_type.as_deref() {
        Some("video") => StreamKind::Video,
        Some("audio") => StreamKind::Audio,
        Some("subtitle") => StreamKind::Subtitle,
        other => anyhow::bail!("unknown codec_type: {:?}", other),
    };

    // Prefer the explicit bit_rate field; fall back to the BPS tag used by some MKV files.
    let bit_rate = raw
        .bit_rate
        .as_deref()
        .and_then(|s| s.parse::<u64>().ok())
        .or_else(|| {
            raw.tags
                .iter()
                .find(|(k, _)| k.starts_with("BPS"))
                .and_then(|(_, v)| v.parse::<u64>().ok())
        });

    let language_tag = raw.tags.get("language").map(|s| s.as_str());
    let language_bcp47 = language_tag.and_then(iso639_2_to_bcp47).map(str::to_string);
    let original_title = raw.tags.get("title").cloned();
    let disposition = convert_disposition(&raw.disposition);

    let display_name = build_display_name(
        language_bcp47.as_deref(),
        language_tag,
        original_title.as_deref(),
        disposition,
    );

    let details = match kind {
        StreamKind::Video => {
            let width = raw.width.context("video stream missing width")?;
            let height = raw.height.context("video stream missing height")?;
            let (time_base_num, time_base_den) = raw
                .time_base
                .as_deref()
                .and_then(parse_time_base)
                .context("video stream missing valid time_base")?;
            let frame_rate = raw.r_frame_rate.as_deref().and_then(parse_frame_rate);
            let bit_depth = raw.pix_fmt.as_deref().and_then(pix_fmt_to_bit_depth);
            let hdr_format = detect_hdr(
                raw.color_transfer.as_deref(),
                raw.color_space.as_deref(),
                raw.side_data_list.as_deref(),
            );
            StreamDetails::Video {
                width,
                height,
                time_base_num,
                time_base_den,
                frame_rate,
                bit_depth,
                hdr_format,
            }
        }
        StreamKind::Audio => {
            let channels = raw.channels.unwrap_or(0);
            let sample_rate = raw
                .sample_rate
                .as_deref()
                .and_then(|s| s.parse::<u32>().ok());
            StreamDetails::Audio {
                channels,
                sample_rate,
            }
        }
        StreamKind::Subtitle => {
            let format = codec_name_to_subtitle_format(&codec_name);
            StreamDetails::Subtitle { format }
        }
    };

    Ok(Stream {
        index: raw.index,
        codec_name,
        display_name,
        original_title,
        bit_rate,
        language_bcp47,
        disposition,
        details,
    })
}

fn convert_disposition(raw: &FfprobeDisposition) -> StreamDisposition {
    let mut flags = StreamDisposition::empty();
    if raw.default != 0 {
        flags |= StreamDisposition::DEFAULT;
    }
    if raw.forced != 0 {
        flags |= StreamDisposition::FORCED;
    }
    if raw.comment != 0 {
        flags |= StreamDisposition::COMMENTARY;
    }
    if raw.hearing_impaired != 0 {
        flags |= StreamDisposition::HEARING_IMPAIRED;
    }
    if raw.visual_impaired != 0 {
        flags |= StreamDisposition::VISUAL_IMPAIRED;
    }
    if raw.original != 0 {
        flags |= StreamDisposition::ORIGINAL;
    }
    if raw.dub != 0 {
        flags |= StreamDisposition::DUBBED;
    }
    flags
}

fn parse_time_base(value: &str) -> Option<(i64, i64)> {
    let (num, den) = value.split_once('/')?;
    let num = num.parse::<i64>().ok()?;
    let den = den.parse::<i64>().ok()?;
    if den <= 0 {
        return None;
    }
    Some((num, den))
}

fn parse_frame_rate(s: &str) -> Option<f32> {
    let (num, den) = s.split_once('/')?;
    let num: f32 = num.parse().ok()?;
    let den: f32 = den.parse().ok()?;
    if den == 0.0 {
        return None;
    }
    Some(num / den)
}

/// Extract bit depth from a pix_fmt string (e.g. "yuv420p10le" -> Some(10)).
fn pix_fmt_to_bit_depth(pix_fmt: &str) -> Option<u8> {
    if pix_fmt.contains("p16") {
        return Some(16);
    }
    if pix_fmt.contains("p12") {
        return Some(12);
    }
    if pix_fmt.contains("p10") {
        return Some(10);
    }
    Some(8)
}

/// Detect HDR format from color metadata and side data.
/// Dolby Vision is checked first via side data since a DV stream can also carry
/// HDR10 metadata, and we want the more specific label.
fn detect_hdr(
    color_transfer: Option<&str>,
    _color_space: Option<&str>,
    side_data_list: Option<&[FfprobeSideData]>,
) -> Option<HDRFormat> {
    if let Some(side_data) = side_data_list {
        for entry in side_data {
            if entry.side_data_type.as_deref() == Some("DOVI configuration record") {
                return Some(HDRFormat::DolbyVision);
            }
        }
    }

    match color_transfer {
        Some("smpte2084") => Some(HDRFormat::Hdr10),
        Some("arib-std-b67") => Some(HDRFormat::Hlg),
        _ => None,
    }
}

fn codec_name_to_subtitle_format(codec_name: &str) -> Option<SubtitleFormat> {
    match codec_name {
        "subrip" | "srt" => Some(SubtitleFormat::Srt),
        "ass" | "ssa" => Some(SubtitleFormat::Ass),
        "webvtt" => Some(SubtitleFormat::WebVtt),
        "hdmv_pgs_subtitle" | "pgs" => Some(SubtitleFormat::Pgs),
        "dvd_subtitle" | "vobsub" | "dvdsub" => Some(SubtitleFormat::VobSub),
        _ => None,
    }
}

fn iso639_2_to_bcp47(code: &str) -> Option<&'static str> {
    match code {
        "eng" => Some("en"),
        "fra" | "fre" => Some("fr"),
        "deu" | "ger" => Some("de"),
        "spa" => Some("es"),
        "ita" => Some("it"),
        "jpn" => Some("ja"),
        "zho" | "chi" => Some("zh"),
        "kor" => Some("ko"),
        "por" => Some("pt"),
        "rus" => Some("ru"),
        "ara" => Some("ar"),
        "hin" => Some("hi"),
        "nld" | "dut" => Some("nl"),
        "swe" => Some("sv"),
        "nor" => Some("no"),
        "dan" => Some("da"),
        "fin" => Some("fi"),
        "pol" => Some("pl"),
        "tur" => Some("tr"),
        "heb" => Some("he"),
        "tha" => Some("th"),
        "vie" => Some("vi"),
        "ind" => Some("id"),
        "ces" | "cze" => Some("cs"),
        "hun" => Some("hu"),
        "ron" | "rum" => Some("ro"),
        "ukr" => Some("uk"),
        "cat" => Some("ca"),
        _ => None,
    }
}

/// Build a human-readable display name from language and disposition flags.
fn build_display_name(
    language_bcp47: Option<&str>,
    language_tag: Option<&str>,
    original_title: Option<&str>,
    disposition: StreamDisposition,
) -> Option<String> {
    if let Some(title) = original_title {
        return Some(title.to_string());
    }

    let base = language_bcp47
        .and_then(bcp47_to_display_name)
        .or(language_tag)?;

    let mut qualifiers: Vec<&str> = Vec::new();
    if disposition.contains(StreamDisposition::FORCED) {
        qualifiers.push("Forced");
    }
    if disposition.contains(StreamDisposition::HEARING_IMPAIRED) {
        qualifiers.push("SDH");
    }
    if disposition.contains(StreamDisposition::COMMENTARY) {
        qualifiers.push("Commentary");
    }
    if disposition.contains(StreamDisposition::VISUAL_IMPAIRED) {
        qualifiers.push("Visual Impaired");
    }

    if qualifiers.is_empty() {
        Some(base.to_string())
    } else {
        Some(format!("{} ({})", base, qualifiers.join(", ")))
    }
}

fn bcp47_to_display_name(code: &str) -> Option<&'static str> {
    match code {
        "en" => Some("English"),
        "fr" => Some("French"),
        "de" => Some("German"),
        "es" => Some("Spanish"),
        "it" => Some("Italian"),
        "ja" => Some("Japanese"),
        "zh" => Some("Chinese"),
        "ko" => Some("Korean"),
        "pt" => Some("Portuguese"),
        "ru" => Some("Russian"),
        "ar" => Some("Arabic"),
        "hi" => Some("Hindi"),
        "nl" => Some("Dutch"),
        "sv" => Some("Swedish"),
        "no" => Some("Norwegian"),
        "da" => Some("Danish"),
        "fi" => Some("Finnish"),
        "pl" => Some("Polish"),
        "tr" => Some("Turkish"),
        "he" => Some("Hebrew"),
        "th" => Some("Thai"),
        "vi" => Some("Vietnamese"),
        "id" => Some("Indonesian"),
        "cs" => Some("Czech"),
        "hu" => Some("Hungarian"),
        "ro" => Some("Romanian"),
        "uk" => Some("Ukrainian"),
        "ca" => Some("Catalan"),
        _ => None,
    }
}
