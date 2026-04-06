use std::path::PathBuf;

use anyhow::Context;
use rusty_chromaprint::match_fingerprints;
use tokio::task::spawn_blocking;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};

use crate::{
    Fingerprint, MAX_MATCH_DURATION_SECONDS, MERGE_SEGMENT_GAP_SECONDS, MIN_INTRO_EPISODE_COUNT,
    MIN_MATCH_DURATION_SECONDS, chromaprint_config,
};

#[derive(Clone, Debug, PartialEq)]
pub struct FileIntroDetection {
    pub path: PathBuf,
    pub intro: Option<IntroRange>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IntroRange {
    pub start_seconds: f32,
    pub end_seconds: f32,
}

#[derive(Clone, Debug)]
struct InputFingerprint {
    path: PathBuf,
    fingerprint: Vec<u32>,
}

#[derive(Clone, Debug)]
struct IntroSegmentCandidate {
    start_seconds: f32,
    end_seconds: f32,
    supporting_file_indexes: Vec<usize>,
    best_score: f64,
}

impl IntroSegmentCandidate {
    fn duration_seconds(&self) -> f32 {
        self.end_seconds - self.start_seconds
    }

    fn episode_count(&self) -> usize {
        self.supporting_file_indexes.len() + 1
    }
}

pub async fn detect_intros(
    input_files: &[(PathBuf, Fingerprint)],
    cancellation_token: Option<&CancellationToken>,
) -> anyhow::Result<Option<Vec<FileIntroDetection>>> {
    info!(file_count = input_files.len(), "starting intro detection");

    let mut fingerprints = Vec::with_capacity(input_files.len());
    for (index, (path, fingerprint)) in input_files.iter().enumerate() {
        if cancellation_token.is_some_and(CancellationToken::is_cancelled) {
            return Ok(None);
        }

        let fingerprint = fingerprint
            .decode()
            .with_context(|| format!("invalid fingerprint cache for '{}'", path.display()))?;

        debug!(
            file_index = index + 1,
            fingerprint_length = fingerprint.len(),
            "loaded fingerprint"
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
                left,
                fingerprints[left].fingerprint.clone(),
                right,
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
        });
    }

    info!("intro detection complete");
    Ok(Some(output))
}

async fn match_fingerprint_pair(
    left_index: usize,
    left_fingerprint: Vec<u32>,
    right_index: usize,
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
                    supporting_file_indexes: vec![right_index],
                    best_score: segment.score,
                });
            }

            let start2 = segment.start2(&config);
            let end2 = segment.end2(&config);
            if end2 > start2 {
                right_candidates.push(IntroSegmentCandidate {
                    start_seconds: start2,
                    end_seconds: end2,
                    supporting_file_indexes: vec![left_index],
                    best_score: segment.score,
                });
            }
        }

        Ok((left_candidates, right_candidates, kept_for_pair))
    })
    .await
    .context("fingerprint match worker failed to join")?
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
                // Count distinct supporting episodes so the intro threshold matches
                // how many episodes actually share this segment.
                last.supporting_file_indexes
                    .extend(segment.supporting_file_indexes);
                last.supporting_file_indexes.sort_unstable();
                last.supporting_file_indexes.dedup();
                last.best_score = last.best_score.max(segment.best_score);
                continue;
            }
        }
        merged.push(segment);
    }

    merged
}

fn select_intro_segment(segments: &[IntroSegmentCandidate]) -> Option<IntroSegmentCandidate> {
    segments
        .iter()
        .filter(|segment| segment.episode_count() >= MIN_INTRO_EPISODE_COUNT)
        .cloned()
        .max_by(|a, b| {
            a.episode_count()
                .cmp(&b.episode_count())
                .then_with(|| a.duration_seconds().total_cmp(&b.duration_seconds()))
                .then_with(|| a.best_score.total_cmp(&b.best_score))
                .then_with(|| b.start_seconds.total_cmp(&a.start_seconds))
        })
}

#[cfg(test)]
mod tests {
    use super::{IntroSegmentCandidate, merge_segments, select_intro_segment};

    #[test]
    fn merged_segments_count_distinct_supporting_episodes() {
        let merged = merge_segments(vec![
            IntroSegmentCandidate {
                start_seconds: 0.0,
                end_seconds: 10.0,
                supporting_file_indexes: vec![1],
                best_score: 0.8,
            },
            IntroSegmentCandidate {
                start_seconds: 0.5,
                end_seconds: 10.5,
                supporting_file_indexes: vec![1],
                best_score: 0.9,
            },
            IntroSegmentCandidate {
                start_seconds: 1.0,
                end_seconds: 11.0,
                supporting_file_indexes: vec![2],
                best_score: 0.7,
            },
        ]);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].supporting_file_indexes, vec![1, 2]);
        assert_eq!(merged[0].episode_count(), 3);
    }

    #[test]
    fn intro_selection_requires_three_total_episodes() {
        let intro = select_intro_segment(&[
            IntroSegmentCandidate {
                start_seconds: 0.0,
                end_seconds: 20.0,
                supporting_file_indexes: vec![1],
                best_score: 0.9,
            },
            IntroSegmentCandidate {
                start_seconds: 40.0,
                end_seconds: 60.0,
                supporting_file_indexes: vec![1, 2],
                best_score: 0.8,
            },
        ]);

        assert_eq!(
            intro.map(|segment| (segment.start_seconds, segment.end_seconds)),
            Some((40.0, 60.0))
        );
    }
}
