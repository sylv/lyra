mod files;
pub(crate) mod job_extract;
pub(crate) mod job_process;
pub mod language;

use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

use crate::entities::file_subtitles::SubtitleKind;
use anyhow::{Result, bail};
use lyra_probe::{Codec, Stream, StreamDisposition, StreamKind, SubtitleFormat};

pub fn subtitle_kind_from_stream(stream: &Stream) -> Option<SubtitleKind> {
    if stream.kind() != StreamKind::Subtitle {
        return None;
    }

    match stream.details.subtitle_format() {
        Some(SubtitleFormat::Srt) => Some(SubtitleKind::Srt),
        Some(SubtitleFormat::WebVtt) => Some(SubtitleKind::Vtt),
        Some(SubtitleFormat::Pgs) => Some(SubtitleKind::Pgs),
        Some(SubtitleFormat::VobSub) => Some(SubtitleKind::VobSub),
        Some(SubtitleFormat::Ass) => Some(SubtitleKind::Ass),
        None => subtitle_kind_from_codec(&stream.codec),
    }
}

pub fn subtitle_kind_from_codec(codec: &Codec) -> Option<SubtitleKind> {
    match codec {
        Codec::SubtitleAss => Some(SubtitleKind::Ass),
        Codec::SubtitleMovText => Some(SubtitleKind::MovText),
        Codec::SubtitleSubRip => Some(SubtitleKind::Srt),
        Codec::SubtitleText => Some(SubtitleKind::Text),
        Codec::SubtitleTtml => Some(SubtitleKind::Ttml),
        Codec::SubtitleWebVtt => Some(SubtitleKind::Vtt),
        Codec::SubtitlePgs => Some(SubtitleKind::Pgs),
        Codec::SubtitleVobSub => Some(SubtitleKind::VobSub),
        _ => None,
    }
}

pub fn extension_for_subtitle_kind(kind: SubtitleKind) -> &'static str {
    match kind {
        SubtitleKind::Srt => "srt",
        SubtitleKind::Vtt => "vtt",
        SubtitleKind::Ass => "ass",
        SubtitleKind::MovText => "txt",
        SubtitleKind::Text => "txt",
        SubtitleKind::Ttml => "ttml",
        SubtitleKind::Pgs => "sup",
        SubtitleKind::VobSub => "tar",
    }
}

pub fn mime_type_for_subtitle_kind(kind: SubtitleKind) -> &'static str {
    match kind {
        SubtitleKind::Srt => "application/x-subrip",
        SubtitleKind::Vtt => "text/vtt",
        SubtitleKind::Ass => "text/x-ssa",
        SubtitleKind::MovText => "text/plain",
        SubtitleKind::Text => "text/plain",
        SubtitleKind::Ttml => "application/ttml+xml",
        SubtitleKind::Pgs => "application/octet-stream",
        SubtitleKind::VobSub => "application/x-tar",
    }
}

pub fn extension_for_asset_file(mime_type: &str) -> Result<&'static str> {
    let mime = mime_type.split(';').next().unwrap_or(mime_type).trim();
    match mime {
        "image/jpeg" => Ok("jpg"),
        "image/png" => Ok("png"),
        "image/webp" => Ok("webp"),
        "image/svg+xml" => Ok("svg"),
        "application/x-subrip" => Ok("srt"),
        "text/vtt" => Ok("vtt"),
        "text/x-ssa" => Ok("ass"),
        "text/plain" => Ok("txt"),
        "application/ttml+xml" => Ok("ttml"),
        "application/octet-stream" => Ok("bin"),
        "application/x-tar" => Ok("tar"),
        other => bail!("unsupported mime type: {other}"),
    }
}

pub fn maybe_compressed_extension(extension: &str, content_encoding: Option<&str>) -> String {
    match content_encoding {
        Some("zstd") => format!("{extension}.zst"),
        _ => extension.to_string(),
    }
}

pub fn subtitle_disposition_bits(disposition: StreamDisposition) -> i64 {
    i64::from(disposition.bits())
}

pub fn disposition_names(bits: i64) -> Vec<&'static str> {
    let Ok(bits) = u16::try_from(bits) else {
        return Vec::new();
    };
    let Some(disposition) = StreamDisposition::from_bits(bits) else {
        return Vec::new();
    };

    let mut names = Vec::new();
    if disposition.contains(StreamDisposition::FORCED) {
        names.push("Forced");
    }
    if disposition.contains(StreamDisposition::HEARING_IMPAIRED) {
        names.push("SDH");
    }
    if disposition.contains(StreamDisposition::COMMENTARY) {
        names.push("Commentary");
    }
    if disposition.contains(StreamDisposition::VISUAL_IMPAIRED) {
        names.push("Visual impaired");
    }
    names
}

trait StreamDetailsExt {
    fn subtitle_format(&self) -> Option<SubtitleFormat>;
}

impl StreamDetailsExt for lyra_probe::StreamDetails {
    fn subtitle_format(&self) -> Option<SubtitleFormat> {
        match self {
            lyra_probe::StreamDetails::Subtitle { format } => *format,
            _ => None,
        }
    }
}

pub(crate) fn register_jobs(
    jobs: &mut Vec<crate::jobs::RegisteredJob>,
    heavy_jobs: &mut Vec<Arc<dyn crate::jobs::HeavyJobRunner>>,
    pool: &DatabaseConnection,
    wake_signal: Arc<Notify>,
    startup_scans_complete: CancellationToken,
) {
    crate::jobs::register_job(
        Arc::new(job_extract::FileSubtitleExtractJob),
        jobs,
        heavy_jobs,
        pool,
        wake_signal.clone(),
        startup_scans_complete.clone(),
    );
    crate::jobs::register_job(
        Arc::new(job_process::FileSubtitleProcessJob),
        jobs,
        heavy_jobs,
        pool,
        wake_signal,
        startup_scans_complete,
    );
}
