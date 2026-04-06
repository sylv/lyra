use std::{
    path::{Path, PathBuf},
    process::Stdio,
};

use anyhow::{Context, bail};
use lyra_probe::{get_ffmpeg_path, probe_with_cancellation};
use rusty_chromaprint::{Configuration, Fingerprinter, match_fingerprints};
use tokio::{io::AsyncReadExt, process::Command as TokioCommand};
use tokio::task::spawn_blocking;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};

pub const INTRO_DETECTION_BATCH_MIN_FILES: usize = 3;
pub const INTRO_DETECTION_BATCH_MAX_FILES: usize = 20;
pub const AUDIO_FINGERPRINT_VERSION: u32 = 1;

const FINGERPRINT_SCAN_RATIO: f64 = 0.40;
const FINGERPRINT_SAMPLE_RATE: u32 = 48_000;
const FINGERPRINT_CHANNELS: u32 = 2;
const MIN_MATCH_DURATION_SECONDS: f32 = 8.0;
const MAX_MATCH_DURATION_SECONDS: f32 = 180.0;
const MERGE_SEGMENT_GAP_SECONDS: f32 = 2.0;
const AUDIO_FINGERPRINT_CACHE_MAGIC: [u8; 4] = *b"LAFP";
const AUDIO_FINGERPRINT_CACHE_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IntroRange {
    pub start_seconds: f32,
    pub end_seconds: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IntroDetectionInputFile {
    pub path: PathBuf,
    pub fingerprint_cache: Option<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FileIntroDetection {
    pub path: PathBuf,
    pub intro: Option<IntroRange>,
    pub fingerprint_cache: Vec<u8>,
}

#[derive(Clone, Debug)]
struct InputFingerprint {
    path: PathBuf,
    fingerprint: Vec<u32>,
    fingerprint_cache: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
struct IntroSegmentCandidate {
    start_seconds: f32,
    end_seconds: f32,
    supporting_matches: u32,
    best_score: f64,
}

impl IntroSegmentCandidate {
    fn duration_seconds(self) -> f32 {
        self.end_seconds - self.start_seconds
    }
}

pub async fn detect_intros(
    input_files: &[IntroDetectionInputFile],
    cancellation_token: Option<&CancellationToken>,
) -> anyhow::Result<Option<Vec<FileIntroDetection>>> {
    if input_files.len() < INTRO_DETECTION_BATCH_MIN_FILES
        || input_files.len() > INTRO_DETECTION_BATCH_MAX_FILES
    {
        bail!(
            "input must contain between {} and {} files, found {}",
            INTRO_DETECTION_BATCH_MIN_FILES,
            INTRO_DETECTION_BATCH_MAX_FILES,
            input_files.len()
        );
    }

    info!(file_count = input_files.len(), "starting intro detection");

    let mut fingerprints = Vec::with_capacity(input_files.len());
    for (index, input_file) in input_files.iter().enumerate() {
        if cancellation_token.is_some_and(CancellationToken::is_cancelled) {
            return Ok(None);
        }

        let path = &input_file.path;
        let fingerprint = if let Some(fingerprint) =
            decode_fingerprint_cache(input_file.fingerprint_cache.as_deref())
        {
            debug!(
                file_index = index + 1,
                file_total = input_files.len(),
                path = %path.display(),
                fingerprint_length = fingerprint.len(),
                "using cached fingerprint"
            );
            fingerprint
        } else {
            info!(
                file_index = index + 1,
                file_total = input_files.len(),
                path = %path.display(),
                "extracting fingerprint"
            );
            let Some(fingerprint) = calc_fingerprint_async(path, cancellation_token).await?
            else {
                return Ok(None);
            };
            fingerprint
        };

        debug!(
            file_index = index + 1,
            fingerprint_length = fingerprint.len(),
            "fingerprint extracted"
        );
        fingerprints.push(InputFingerprint {
            path: path.clone(),
            fingerprint_cache: encode_fingerprint_cache(AUDIO_FINGERPRINT_VERSION, &fingerprint),
            fingerprint,
        });
    }

    let pair_total = fingerprints.len() * (fingerprints.len() - 1) / 2;
    let mut pair_index = 0usize;
    let mut file_segments = vec![Vec::<IntroSegmentCandidate>::new(); fingerprints.len()];
    for left in 0..fingerprints.len() {
        for right in (left + 1)..fingerprints.len() {
            if cancellation_token.is_some_and(CancellationToken::is_cancelled) {
                return Ok(None);
            }

            pair_index += 1;
            debug!(
                pair_index,
                pair_total,
                left_path = %fingerprints[left].path.display(),
                right_path = %fingerprints[right].path.display(),
                "matching fingerprints"
            );

            let (left_candidates, right_candidates, kept_for_pair) = match_fingerprint_pair(
                fingerprints[left].fingerprint.clone(),
                fingerprints[right].fingerprint.clone(),
            )
            .await?;
            file_segments[left].extend(left_candidates);
            file_segments[right].extend(right_candidates);

            debug!(
                pair_index,
                pair_total,
                kept_segments = kept_for_pair,
                "pair match complete"
            );
        }
    }

    let mut output = Vec::with_capacity(fingerprints.len());
    for (index, file) in fingerprints.iter().enumerate() {
        if cancellation_token.is_some_and(CancellationToken::is_cancelled) {
            return Ok(None);
        }

        let merged = merge_segments(file_segments[index].clone());
        let intro = select_intro_segment(&merged).map(|segment| IntroRange {
            start_seconds: segment.start_seconds,
            end_seconds: segment.end_seconds,
        });

        if let Some(intro) = intro {
            info!(
                path = %file.path.display(),
                start_seconds = intro.start_seconds,
                end_seconds = intro.end_seconds,
                "intro detected"
            );
        } else {
            info!(path = %file.path.display(), "intro not found");
        }

        output.push(FileIntroDetection {
            path: file.path.clone(),
            intro,
            fingerprint_cache: file.fingerprint_cache.clone(),
        });
    }

    info!("intro detection complete");
    Ok(Some(output))
}

async fn calc_fingerprint_async(
    path: impl AsRef<Path>,
    cancellation_token: Option<&CancellationToken>,
) -> anyhow::Result<Option<Vec<u32>>> {
    let path = path.as_ref();
    let probe = probe_with_cancellation(path, cancellation_token).await?;
    let Some(probe) = probe else {
        return Ok(None);
    };
    let duration_seconds = probe
        .duration_secs
        .context("missing file duration from probe")?;
    let scan_seconds = duration_seconds * FINGERPRINT_SCAN_RATIO;

    debug!(
        path = %path.display(),
        duration_seconds,
        scan_seconds,
        "preparing fingerprint decode"
    );

    let ffmpeg_path = get_ffmpeg_path();
    let (samples_tx, samples_rx) = std::sync::mpsc::channel::<Vec<i16>>();
    let fingerprint_worker = spawn_blocking(move || -> anyhow::Result<Vec<u32>> {
        let config = chromaprint_config();
        let mut printer = Fingerprinter::new(&config);
        printer
            .start(FINGERPRINT_SAMPLE_RATE, FINGERPRINT_CHANNELS)
            .context("initializing fingerprinter")?;

        while let Ok(samples) = samples_rx.recv() {
            if !samples.is_empty() {
                printer.consume(&samples);
            }
        }

        printer.finish();
        Ok(printer.fingerprint().to_vec())
    });

    let mut ffmpeg = TokioCommand::new(&ffmpeg_path)
        .kill_on_drop(true)
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-nostdin")
        .arg("-i")
        .arg(path)
        .arg("-map")
        .arg("0:a:0")
        .arg("-vn")
        .arg("-ac")
        .arg(FINGERPRINT_CHANNELS.to_string())
        .arg("-ar")
        .arg(FINGERPRINT_SAMPLE_RATE.to_string())
        .arg("-t")
        .arg(format!("{scan_seconds:.4}"))
        .arg("-f")
        .arg("s16le")
        .arg("pipe:1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to start ffmpeg for '{}'", path.display()))?;

    let mut stdout = ffmpeg
        .stdout
        .take()
        .context("failed to capture ffmpeg stdout")?;
    let mut bytes = [0_u8; 32_768];
    let mut samples = Vec::<i16>::with_capacity(bytes.len() / 2);
    let mut trailing_byte = None::<u8>;

    loop {
        let read = if let Some(cancellation_token) = cancellation_token {
            tokio::select! {
                read = stdout.read(&mut bytes) => read,
                _ = cancellation_token.cancelled() => {
                    let _ = ffmpeg.kill().await;
                    let _ = ffmpeg.wait().await;
                    drop(samples_tx);
                    let _ = fingerprint_worker.await;
                    return Ok(None);
                }
            }
        } else {
            stdout.read(&mut bytes).await
        }
        .with_context(|| format!("failed to read decoded audio for '{}'", path.display()))?;

        if read == 0 {
            break;
        }

        let mut offset = 0;
        if let Some(prev) = trailing_byte.take() {
            let sample = i16::from_le_bytes([prev, bytes[0]]);
            samples.push(sample);
            offset = 1;
        }

        let available = read.saturating_sub(offset);
        let pair_count = available / 2;
        for idx in 0..pair_count {
            let i = offset + idx * 2;
            samples.push(i16::from_le_bytes([bytes[i], bytes[i + 1]]));
        }

        if available % 2 == 1 {
            trailing_byte = Some(bytes[offset + pair_count * 2]);
        }

        if !samples.is_empty() {
            samples_tx
                .send(std::mem::take(&mut samples))
                .with_context(|| format!("fingerprint worker dropped for '{}'", path.display()))?;
        }
    }

    let output = ffmpeg.wait_with_output();
    tokio::pin!(output);
    let output = if let Some(cancellation_token) = cancellation_token {
        tokio::select! {
            output = &mut output => output?,
            _ = cancellation_token.cancelled() => {
                drop(samples_tx);
                let _ = fingerprint_worker.await;
                return Ok(None);
            },
        }
    } else {
        output.await?
    };
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("ffmpeg failed for '{}': {}", path.display(), stderr.trim());
    }

    if trailing_byte.is_some() {
        bail!(
            "ffmpeg returned truncated PCM data for '{}'",
            path.display()
        );
    }

    drop(samples_tx);
    let fingerprint = fingerprint_worker
        .await
        .context("fingerprint worker failed to join")??;
    Ok(Some(fingerprint))
}

async fn match_fingerprint_pair(
    left_fingerprint: Vec<u32>,
    right_fingerprint: Vec<u32>,
) -> anyhow::Result<(
    Vec<IntroSegmentCandidate>,
    Vec<IntroSegmentCandidate>,
    usize,
)> {
    spawn_blocking(move || {
        let config = chromaprint_config();
        let segments = match_fingerprints(&left_fingerprint, &right_fingerprint, &config)?;

        let mut left_candidates = Vec::new();
        let mut right_candidates = Vec::new();
        let mut kept_for_pair = 0usize;
        for segment in segments {
            let duration_seconds = segment.duration(&config);
            if !(MIN_MATCH_DURATION_SECONDS..=MAX_MATCH_DURATION_SECONDS)
                .contains(&duration_seconds)
            {
                continue;
            }

            kept_for_pair += 1;

            let start1 = segment.start1(&config);
            let end1 = segment.end1(&config);
            if end1 > start1 {
                left_candidates.push(IntroSegmentCandidate {
                    start_seconds: start1,
                    end_seconds: end1,
                    supporting_matches: 1,
                    best_score: segment.score,
                });
            }

            let start2 = segment.start2(&config);
            let end2 = segment.end2(&config);
            if end2 > start2 {
                right_candidates.push(IntroSegmentCandidate {
                    start_seconds: start2,
                    end_seconds: end2,
                    supporting_matches: 1,
                    best_score: segment.score,
                });
            }
        }

        Ok((left_candidates, right_candidates, kept_for_pair))
    })
    .await
    .context("fingerprint match worker failed to join")?
}

fn chromaprint_config() -> Configuration {
    Configuration::preset_test1()
        .with_id(50)
        .with_removed_silence(50)
}

fn decode_fingerprint_cache(cache: Option<&[u8]>) -> Option<Vec<u32>> {
    let cache = cache?;
    if cache.is_empty() {
        return None;
    }

    if cache.len() < 16 {
        return None;
    }
    if cache[0..4] != AUDIO_FINGERPRINT_CACHE_MAGIC {
        return None;
    }

    let schema_version = read_u32_le(cache, 4)?;
    if schema_version != AUDIO_FINGERPRINT_CACHE_SCHEMA_VERSION {
        return None;
    }

    let fingerprint_version = read_u32_le(cache, 8)?;
    if fingerprint_version != AUDIO_FINGERPRINT_VERSION {
        return None;
    }

    let value_count = read_u32_le(cache, 12)? as usize;
    let payload = cache.get(16..)?;
    if payload.len() != value_count.saturating_mul(4) {
        return None;
    }

    let mut output = Vec::with_capacity(value_count);
    for chunk in payload.chunks_exact(4) {
        output.push(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
    }
    Some(output)
}

fn encode_fingerprint_cache(fingerprint_version: u32, fingerprint: &[u32]) -> Vec<u8> {
    let mut output = Vec::with_capacity(16 + fingerprint.len().saturating_mul(4));
    output.extend_from_slice(&AUDIO_FINGERPRINT_CACHE_MAGIC);
    output.extend_from_slice(&AUDIO_FINGERPRINT_CACHE_SCHEMA_VERSION.to_le_bytes());
    output.extend_from_slice(&fingerprint_version.to_le_bytes());
    output.extend_from_slice(&(fingerprint.len() as u32).to_le_bytes());
    for value in fingerprint {
        output.extend_from_slice(&value.to_le_bytes());
    }
    output
}

fn read_u32_le(bytes: &[u8], offset: usize) -> Option<u32> {
    let end = offset.checked_add(4)?;
    let chunk = bytes.get(offset..end)?;
    Some(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
}

fn merge_segments(mut segments: Vec<IntroSegmentCandidate>) -> Vec<IntroSegmentCandidate> {
    if segments.is_empty() {
        return segments;
    }

    segments.sort_by(|a, b| {
        a.start_seconds
            .total_cmp(&b.start_seconds)
            .then_with(|| a.end_seconds.total_cmp(&b.end_seconds))
    });

    let mut merged = Vec::<IntroSegmentCandidate>::with_capacity(segments.len());
    for segment in segments {
        if let Some(last) = merged.last_mut() {
            if segment.start_seconds - last.end_seconds <= MERGE_SEGMENT_GAP_SECONDS {
                last.end_seconds = last.end_seconds.max(segment.end_seconds);
                last.supporting_matches += segment.supporting_matches;
                last.best_score = last.best_score.max(segment.best_score);
                continue;
            }
        }
        merged.push(segment);
    }

    merged
}

fn select_intro_segment(segments: &[IntroSegmentCandidate]) -> Option<IntroSegmentCandidate> {
    segments.iter().copied().max_by(|a, b| {
        a.supporting_matches
            .cmp(&b.supporting_matches)
            .then_with(|| a.duration_seconds().total_cmp(&b.duration_seconds()))
            .then_with(|| a.best_score.total_cmp(&b.best_score))
            .then_with(|| b.start_seconds.total_cmp(&a.start_seconds))
    })
}
