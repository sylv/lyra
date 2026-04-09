use crate::{SubtitleFrame, SubtitlePositioning, convert_to_grayscale};
use anyhow::{Context, Result};
use image::{Rgba, RgbaImage};
use std::path::Path;
use subtile::{
    image::{ImageArea, ImageSize},
    time::TimeSpan,
    vobsub::{Index, Sub, VobSubIndexedImage},
};

pub(crate) fn parse_vobsub_frames(
    idx_path: &Path,
    sub_path: &Path,
    canvas_width: u32,
    canvas_height: u32,
) -> Result<Vec<SubtitleFrame>> {
    let idx =
        Index::open(idx_path).with_context(|| format!("failed to open {}", idx_path.display()))?;
    let sub =
        Sub::open(sub_path).with_context(|| format!("failed to open {}", sub_path.display()))?;

    let mut frames = Vec::new();

    for subtitle in sub.subtitles::<(TimeSpan, VobSubIndexedImage)>() {
        match subtitle {
            Ok((times, image)) => {
                let area = image.area();
                frames.push(SubtitleFrame {
                    start: timespan_to_seconds(&times, true),
                    end: timespan_to_seconds(&times, false),
                    image: convert_to_grayscale(render_vobsub_bitmap(&image, idx.palette())),
                    positioning: Some(SubtitlePositioning {
                        canvas_width,
                        canvas_height,
                        left: u32::from(area.left()),
                        top: u32::from(area.top()),
                    }),
                });
            }
            Err(error) => {
                tracing::warn!(%error, "failed to decode VobSub subtitle image");
            }
        }
    }

    Ok(frames)
}

fn render_vobsub_bitmap(image: &VobSubIndexedImage, palette: &[image::Rgb<u8>; 16]) -> RgbaImage {
    let width = image.width();
    let height = image.height();
    let subtitle_palette = image.palette();
    let subtitle_alpha = image.alpha();
    let raw_image = image.raw_image();

    RgbaImage::from_fn(width, height, |x, y| {
        let offset = (y * width + x) as usize;
        let subtitle_color_index = raw_image[offset] as usize;
        let palette_index = subtitle_palette[subtitle_color_index] as usize;
        let rgb = palette[palette_index];
        let alpha = subtitle_alpha[subtitle_color_index].saturating_mul(17);

        Rgba([rgb[0], rgb[1], rgb[2], alpha])
    })
}

fn timespan_to_seconds(times: &TimeSpan, is_start: bool) -> f64 {
    let point = if is_start { times.start } else { times.end };
    point.msecs() as f64 / 1_000.0
}
