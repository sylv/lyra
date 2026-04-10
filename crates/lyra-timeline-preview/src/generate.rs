use std::{path::PathBuf, process::Stdio, time::Duration};

use tokio::{io::AsyncWriteExt, process::Command};

use crate::{
    GAP_PX, MAX_FRAMES_PER_SHEET, MAX_UNCOMPRESSED_SIZE_BYTES, PreviewOptions, TimelinePreview,
    WEBP_QUALITY,
};
use lyra_probe::get_ffmpeg_path;

#[derive(Clone, Copy, Debug)]
struct SheetLayout {
    frames_per_sheet: usize,
}

pub(crate) async fn generate_sheets(
    frame_paths: &[(u32, PathBuf)],
    options: &PreviewOptions,
) -> anyhow::Result<Vec<TimelinePreview>> {
    if frame_paths.is_empty() {
        return Ok(Vec::new());
    }

    let (_, first_path) = &frame_paths[0];
    let first_frame = load_frame_rgb(first_path).await?;
    let frame_width = first_frame.width();
    let frame_height = first_frame.height();
    let layout = compute_layout(frame_width, frame_height, frame_paths.len())?;
    let frame_interval = Duration::from_secs_f64(options.frame_interval_seconds);
    tracing::debug!(
        "timeline layout: frame={}x{}, per_sheet={}",
        frame_width,
        frame_height,
        layout.frames_per_sheet
    );

    let mut timeline_previews: Vec<TimelinePreview> = Vec::new();
    let sheet_frame_counts = distribute_frame_counts(frame_paths.len(), layout.frames_per_sheet);
    let mut start_index = 0usize;

    // once extraction has finished, let the merge run through so we do not throw away
    // nearly-complete work for a short webp encode.
    for (sheet_index, frame_count) in sheet_frame_counts.iter().copied().enumerate() {
        let end_index = start_index + frame_count;
        let frame_chunk = &frame_paths[start_index..end_index];
        let (columns, _, sheet_width, sheet_height) =
            compute_sheet_geometry(frame_count, frame_width, frame_height);
        let mut sheet_rgb = vec![0_u8; rgb_buffer_len(sheet_width, sheet_height)?];

        for (chunk_index, (_, frame_path)) in frame_chunk.iter().enumerate() {
            let frame = if sheet_index == 0 && chunk_index == 0 {
                first_frame.clone()
            } else {
                load_frame_rgb(frame_path).await?
            };

            if frame.width() != frame_width || frame.height() != frame_height {
                anyhow::bail!(
                    "frame dimensions changed for {}: expected {}x{}, got {}x{}",
                    frame_path.display(),
                    frame_width,
                    frame_height,
                    frame.width(),
                    frame.height()
                );
            }

            let column = chunk_index % columns;
            let row = chunk_index / columns;
            let dest_x = GAP_PX + (column as u32 * (frame_width + GAP_PX));
            let dest_y = GAP_PX + (row as u32 * (frame_height + GAP_PX));
            blit_frame(&mut sheet_rgb, sheet_width, &frame, dest_x, dest_y);
        }

        let webp_bytes = convert_rgb_to_webp(sheet_width, sheet_height, &sheet_rgb).await?;

        timeline_previews.push(TimelinePreview {
            preview_bytes: webp_bytes,
            start_time: Duration::from_secs_f64(
                (start_index as f64) * options.frame_interval_seconds,
            ),
            end_time: Duration::from_secs_f64((end_index as f64) * options.frame_interval_seconds),
            frame_interval,
            width_px: sheet_width,
        });

        start_index = end_index;
    }

    tokio::fs::remove_dir_all(&options.working_dir).await.ok();
    Ok(timeline_previews)
}

fn compute_layout(
    frame_width: u32,
    frame_height: u32,
    total_frames: usize,
) -> anyhow::Result<SheetLayout> {
    if frame_width == 0 || frame_height == 0 {
        anyhow::bail!("frame dimensions must be non-zero");
    }
    if total_frames == 0 {
        anyhow::bail!("total_frames must be non-zero");
    }

    let per_frame_bytes = rgb_buffer_len(frame_width, frame_height)?;
    let max_frames_from_raw = MAX_UNCOMPRESSED_SIZE_BYTES / per_frame_bytes;
    if max_frames_from_raw == 0 {
        anyhow::bail!(
            "single frame exceeds max uncompressed size: frame={}x{}, max={} bytes",
            frame_width,
            frame_height,
            MAX_UNCOMPRESSED_SIZE_BYTES
        );
    }

    let max_candidate = total_frames
        .min(max_frames_from_raw)
        .min(MAX_FRAMES_PER_SHEET);
    for frames_per_sheet in (1..=max_candidate).rev() {
        let (_, _, sheet_width, sheet_height) =
            compute_sheet_geometry(frames_per_sheet, frame_width, frame_height);
        let sheet_bytes = rgb_buffer_len(sheet_width, sheet_height)?;
        if sheet_bytes <= MAX_UNCOMPRESSED_SIZE_BYTES {
            return Ok(SheetLayout { frames_per_sheet });
        }
    }

    anyhow::bail!(
        "failed to fit sprite sheet for frame={}x{} within {} bytes",
        frame_width,
        frame_height,
        MAX_UNCOMPRESSED_SIZE_BYTES
    );
}

fn compute_sheet_geometry(
    frame_count: usize,
    frame_width: u32,
    frame_height: u32,
) -> (usize, usize, u32, u32) {
    let columns = choose_columns(frame_count);
    let rows = frame_count.div_ceil(columns);
    let sheet_width = ((columns as u32) * frame_width) + ((columns as u32 + 1) * GAP_PX);
    let sheet_height = ((rows as u32) * frame_height) + ((rows as u32 + 1) * GAP_PX);
    (columns, rows, sheet_width, sheet_height)
}

fn choose_columns(frame_count: usize) -> usize {
    ((frame_count as f64).sqrt().ceil() as usize).max(1)
}

fn distribute_frame_counts(total_frames: usize, max_frames_per_sheet: usize) -> Vec<usize> {
    let sheet_count = total_frames.div_ceil(max_frames_per_sheet.max(1));
    let base = total_frames / sheet_count;
    let remainder = total_frames % sheet_count;

    (0..sheet_count)
        .map(|idx| base + usize::from(idx < remainder))
        .collect()
}

async fn load_frame_rgb(frame_path: &PathBuf) -> anyhow::Result<image::RgbImage> {
    let bytes = tokio::fs::read(frame_path).await?;
    let image = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)?;
    Ok(image.into_rgb8())
}

fn blit_frame(
    sheet_rgb: &mut [u8],
    sheet_width: u32,
    frame: &image::RgbImage,
    dest_x: u32,
    dest_y: u32,
) {
    let frame_width = frame.width() as usize;
    let frame_height = frame.height() as usize;
    let sheet_width = sheet_width as usize;
    let row_bytes = frame_width * 3;
    let src = frame.as_raw();

    for row in 0..frame_height {
        let src_start = row * row_bytes;
        let src_end = src_start + row_bytes;
        let dst_start = ((dest_y as usize + row) * sheet_width + dest_x as usize) * 3;
        let dst_end = dst_start + row_bytes;
        sheet_rgb[dst_start..dst_end].copy_from_slice(&src[src_start..src_end]);
    }
}

async fn convert_rgb_to_webp(width: u32, height: u32, rgb: &[u8]) -> anyhow::Result<Vec<u8>> {
    let expected_len = rgb_buffer_len(width, height)?;
    if rgb.len() != expected_len {
        anyhow::bail!(
            "rgb buffer size mismatch, expected {} bytes, got {} bytes",
            expected_len,
            rgb.len()
        );
    }

    let ffmpeg_bin = get_ffmpeg_path();
    let mut child = Command::new(ffmpeg_bin)
        .kill_on_drop(true)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
            "-video_size",
            &format!("{}x{}", width, height),
            "-i",
            "pipe:0",
            "-frames:v",
            "1",
            "-c:v",
            "libwebp",
            "-q:v",
            &WEBP_QUALITY.to_string(),
            "-f",
            "webp",
            "pipe:1",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow::anyhow!("failed to open ffmpeg stdin"))?;
        stdin.write_all(rgb).await?;
        stdin.shutdown().await?;
    }

    let output = child.wait_with_output().await?;
    if !output.status.success() {
        anyhow::bail!(
            "ffmpeg failed to encode webp: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(output.stdout)
}

fn rgb_buffer_len(width: u32, height: u32) -> anyhow::Result<usize> {
    let pixels = (width as usize)
        .checked_mul(height as usize)
        .ok_or_else(|| anyhow::anyhow!("overflow calculating pixel count"))?;
    pixels
        .checked_mul(3)
        .ok_or_else(|| anyhow::anyhow!("overflow calculating rgb byte length"))
}
