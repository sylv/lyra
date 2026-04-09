use crate::{SubtitleFrame, convert_to_grayscale};
use anyhow::{Context, Result};
use image::{Rgba, RgbaImage};
use std::{fs::File, io::BufReader, path::Path};
use subtile::{
    image::ToImage,
    pgs::{DecodeTimeImage, RleToImage, SupParser},
};

const DEFAULT_LAST_CUE_DURATION_MSECS: i64 = 5_000;

pub(crate) fn parse_pgs_frames(sup_path: &Path) -> Result<Vec<SubtitleFrame>> {
    let parser = SupParser::<BufReader<File>, DecodeTimeImage>::from_file(sup_path)
        .with_context(|| format!("failed to open {}", sup_path.display()))?;

    let mut frames = Vec::new();

    for subtitle in parser {
        let (times, image) = subtitle.context("failed to parse PGS subtitle")?;
        let end_msecs = normalize_end_msecs(times.start.msecs(), times.end.msecs());

        if end_msecs <= times.start.msecs() {
            continue;
        }

        let rgba_image: RgbaImage = RleToImage::new(&image, |pixel| {
            Rgba([pixel[0], pixel[0], pixel[0], pixel[1]])
        })
        .to_image();

        // TODO: subtile's PGS decoder does not expose placement metadata, so these cues currently
        // render without subtitle positioning information.
        frames.push(SubtitleFrame {
            start: times.start.to_secs(),
            end: msecs_to_seconds(end_msecs),
            image: convert_to_grayscale(rgba_image),
            positioning: None,
        });
    }

    Ok(frames)
}

fn normalize_end_msecs(start_msecs: i64, end_msecs: i64) -> i64 {
    if end_msecs > start_msecs {
        end_msecs
    } else {
        start_msecs + DEFAULT_LAST_CUE_DURATION_MSECS
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_end_msecs;

    #[test]
    fn normalize_end_msecs_preserves_positive_duration() {
        assert_eq!(normalize_end_msecs(1_000, 2_500), 2_500);
    }

    #[test]
    fn normalize_end_msecs_falls_back_for_missing_end_time() {
        assert_eq!(normalize_end_msecs(1_000, 1_000), 6_000);
    }
}

fn msecs_to_seconds(msecs: i64) -> f64 {
    msecs as f64 / 1_000.0
}
