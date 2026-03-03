use std::{
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{Context, bail};
use rusty_chromaprint::{Configuration, Fingerprinter, match_fingerprints};
use tracing::{debug, info};

pub const INTRO_DETECTION_BATCH_MIN_FILES: usize = 3;
pub const INTRO_DETECTION_BATCH_MAX_FILES: usize = 20;

const FINGERPRINT_SCAN_RATIO: f64 = 0.40;
const FINGERPRINT_SAMPLE_RATE: u32 = 48_000;
const FINGERPRINT_CHANNELS: u32 = 2;
const MIN_MATCH_DURATION_SECONDS: f32 = 8.0;
const MAX_MATCH_DURATION_SECONDS: f32 = 180.0;
const MERGE_SEGMENT_GAP_SECONDS: f32 = 2.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IntroRange {
    pub start_seconds: f32,
    pub end_seconds: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FileIntroDetection {
    pub path: PathBuf,
    pub intro: Option<IntroRange>,
}

#[derive(Clone, Debug)]
struct InputFingerprint {
    path: PathBuf,
    fingerprint: Vec<u32>,
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

pub fn detect_intros(input_files: &[PathBuf]) -> anyhow::Result<Vec<FileIntroDetection>> {
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

    lyra_ffprobe::paths::init_ffmpeg().context("failed to configure ffmpeg binary")?;
    let ffmpeg_path = lyra_ffprobe::paths::get_ffmpeg_path()?;
    let ffprobe_path = lyra_ffprobe::paths::get_ffprobe_path()?;
    detect_intros_with_tools(&ffmpeg_path, &ffprobe_path, input_files)
}

fn detect_intros_with_tools(
    ffmpeg_path: &str,
    ffprobe_path: &str,
    input_files: &[PathBuf],
) -> anyhow::Result<Vec<FileIntroDetection>> {
    let config = Configuration::preset_test1()
        .with_id(50)
        .with_removed_silence(50);

    let mut fingerprints = Vec::with_capacity(input_files.len());
    for (index, path) in input_files.iter().enumerate() {
        info!(
            file_index = index + 1,
            file_total = input_files.len(),
            path = %path.display(),
            "extracting fingerprint"
        );
        let fingerprint = calc_fingerprint(ffmpeg_path, ffprobe_path, path, &config)?;
        debug!(
            file_index = index + 1,
            fingerprint_length = fingerprint.len(),
            "fingerprint extracted"
        );
        fingerprints.push(InputFingerprint {
            path: path.clone(),
            fingerprint,
        });
    }

    let pair_total = fingerprints.len() * (fingerprints.len() - 1) / 2;
    let mut pair_index = 0usize;
    let mut file_segments = vec![Vec::<IntroSegmentCandidate>::new(); fingerprints.len()];
    for left in 0..fingerprints.len() {
        for right in (left + 1)..fingerprints.len() {
            pair_index += 1;
            debug!(
                pair_index,
                pair_total,
                left_path = %fingerprints[left].path.display(),
                right_path = %fingerprints[right].path.display(),
                "matching fingerprints"
            );

            let segments = match_fingerprints(
                &fingerprints[left].fingerprint,
                &fingerprints[right].fingerprint,
                &config,
            )?;

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
                    file_segments[left].push(IntroSegmentCandidate {
                        start_seconds: start1,
                        end_seconds: end1,
                        supporting_matches: 1,
                        best_score: segment.score,
                    });
                }

                let start2 = segment.start2(&config);
                let end2 = segment.end2(&config);
                if end2 > start2 {
                    file_segments[right].push(IntroSegmentCandidate {
                        start_seconds: start2,
                        end_seconds: end2,
                        supporting_matches: 1,
                        best_score: segment.score,
                    });
                }
            }

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
        });
    }

    info!("intro detection complete");
    Ok(output)
}

fn calc_fingerprint(
    ffmpeg_path: &str,
    ffprobe_path: &str,
    path: impl AsRef<Path>,
    config: &Configuration,
) -> anyhow::Result<Vec<u32>> {
    let path = path.as_ref();
    let probe = lyra_ffprobe::probe_streams(Path::new(ffprobe_path), path)
        .with_context(|| format!("failed to probe '{}'", path.display()))?;
    let duration_seconds = probe
        .duration_seconds
        .context("missing file duration from ffprobe")?;
    let scan_seconds = duration_seconds * FINGERPRINT_SCAN_RATIO;

    debug!(
        path = %path.display(),
        duration_seconds,
        scan_seconds,
        "preparing fingerprint decode"
    );

    let mut printer = Fingerprinter::new(config);
    printer
        .start(FINGERPRINT_SAMPLE_RATE, FINGERPRINT_CHANNELS)
        .context("initializing fingerprinter")?;

    let mut ffmpeg = Command::new(ffmpeg_path)
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
        let read = stdout
            .read(&mut bytes)
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
            printer.consume(&samples);
            samples.clear();
        }
    }

    let output = ffmpeg
        .wait_with_output()
        .with_context(|| format!("failed to finish ffmpeg for '{}'", path.display()))?;
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

    printer.finish();
    Ok(printer.fingerprint().to_vec())
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
