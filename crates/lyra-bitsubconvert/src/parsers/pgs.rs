use crate::{SubtitleFrame, SubtitlePositioning, convert_to_grayscale};
use anyhow::{Context, Result};
use image::{RgbaImage, imageops};
use pgs_rs::{
    parse::parse_pgs,
    render::{DisplaySetIterator, render_display_set},
};
use std::{fs, path::Path};

const DEFAULT_LAST_CUE_DURATION_MSECS: i64 = 5_000;
const PGS_TIMEBASE: u64 = 90_000;

pub(crate) fn parse_pgs_frames(sup_path: &Path) -> Result<Vec<SubtitleFrame>> {
    let mut data =
        fs::read(sup_path).with_context(|| format!("failed to read {}", sup_path.display()))?;
    let pgs = parse_pgs(&mut data).context("failed to parse PGS stream")?;
    let display_sets = DisplaySetIterator::new(&pgs).collect::<Vec<_>>();

    let mut frames = Vec::new();

    for (index, display_set) in display_sets.iter().enumerate() {
        if display_set.is_empty() {
            continue;
        }

        let rgba = match render_display_set(display_set) {
            Ok(rgba) => rgba,
            Err(error) => {
                tracing::warn!(%error, "failed to render PGS display set");
                continue;
            }
        };

        let Some(full_image) =
            RgbaImage::from_raw(display_set.width.into(), display_set.height.into(), rgba)
        else {
            tracing::warn!("failed to construct RGBA image from rendered PGS display set");
            continue;
        };

        let Some(content_bounds) = opaque_bounds(&full_image) else {
            continue;
        };

        let cropped_image = imageops::crop_imm(
            &full_image,
            content_bounds.x,
            content_bounds.y,
            content_bounds.width,
            content_bounds.height,
        )
        .to_image();

        let start_msecs = pts_to_msecs(display_set.presentation_timestamp);
        let end = display_sets
            .get(index + 1)
            .map(|next| pts_to_msecs(next.presentation_timestamp))
            .unwrap_or(start_msecs + DEFAULT_LAST_CUE_DURATION_MSECS);

        if end <= start_msecs {
            continue;
        }

        frames.push(SubtitleFrame {
            start: msecs_to_seconds(start_msecs),
            end: msecs_to_seconds(end),
            image: convert_to_grayscale(cropped_image),
            positioning: Some(SubtitlePositioning {
                canvas_width: display_set.width.into(),
                canvas_height: display_set.height.into(),
                left: content_bounds.x,
                top: content_bounds.y,
            }),
        })
    }

    Ok(frames)
}

#[derive(Clone, Copy)]
struct Bounds {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

fn opaque_bounds(image: &RgbaImage) -> Option<Bounds> {
    let mut bounds: Option<(u32, u32, u32, u32)> = None;

    for (x, y, pixel) in image.enumerate_pixels() {
        if pixel[3] == 0 {
            continue;
        }

        bounds = Some(match bounds {
            Some((min_x, min_y, max_x, max_y)) => {
                (min_x.min(x), min_y.min(y), max_x.max(x), max_y.max(y))
            }
            None => (x, y, x, y),
        });
    }

    let (min_x, min_y, max_x, max_y) = bounds?;
    let width = max_x.checked_sub(min_x)?.checked_add(1)?;
    let height = max_y.checked_sub(min_y)?.checked_add(1)?;

    Some(Bounds {
        x: min_x,
        y: min_y,
        width,
        height,
    })
}

#[cfg(test)]
mod tests {
    use super::opaque_bounds;
    use image::{Rgba, RgbaImage};

    #[test]
    fn opaque_bounds_returns_single_pixel_bounds() {
        let mut image = RgbaImage::from_pixel(4, 3, Rgba([0, 0, 0, 0]));
        image.put_pixel(2, 1, Rgba([255, 255, 255, 255]));

        let bounds = opaque_bounds(&image).expect("expected opaque bounds");

        assert_eq!(bounds.x, 2);
        assert_eq!(bounds.y, 1);
        assert_eq!(bounds.width, 1);
        assert_eq!(bounds.height, 1);
    }

    #[test]
    fn opaque_bounds_returns_rectangle_bounds() {
        let mut image = RgbaImage::from_pixel(8, 6, Rgba([0, 0, 0, 0]));
        image.put_pixel(2, 1, Rgba([255, 255, 255, 255]));
        image.put_pixel(5, 4, Rgba([255, 255, 255, 255]));

        let bounds = opaque_bounds(&image).expect("expected opaque bounds");

        assert_eq!(bounds.x, 2);
        assert_eq!(bounds.y, 1);
        assert_eq!(bounds.width, 4);
        assert_eq!(bounds.height, 4);
    }

    #[test]
    fn opaque_bounds_returns_none_for_fully_transparent_image() {
        let image = RgbaImage::from_pixel(4, 3, Rgba([0, 0, 0, 0]));
        assert!(opaque_bounds(&image).is_none());
    }
}

fn pts_to_msecs(pts: u32) -> i64 {
    let msecs = ((u64::from(pts) * 1_000) + (PGS_TIMEBASE / 2)) / PGS_TIMEBASE;
    msecs as i64
}

fn msecs_to_seconds(msecs: i64) -> f64 {
    msecs as f64 / 1_000.0
}
