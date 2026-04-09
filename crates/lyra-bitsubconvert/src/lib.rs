use crate::{
    models::{best_models_for_language, upsert_model},
    ocr::{OcrPool, infer},
    parsers::{pgs::parse_pgs_frames, vobsub::parse_vobsub_frames},
};
use anyhow::{Context, Result};
use image::{GrayImage, Rgb, RgbImage, RgbaImage};
use ort::session::builder::PrepackedWeights;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::path::{Path, PathBuf};
use tokio::task::spawn_blocking;

mod models;
mod ocr;
mod parsers;

const PADDING: u32 = 10;

// Pixels below the alpha threshold are treated as transparent (background).
// Pixels below the luma threshold are treated as too dark to be subtitle text.
const A_THRESHOLD: u8 = 50;
const L_THRESHOLD: u8 = 100;

/// Number of concurrent OCR sessions per model. Det and rec pools are independent,
/// so at peak there are 2*N sessions loaded. Each session uses ~100-300MB RAM.
const SESSION_POOL_SIZE: usize = 4;

const DEFAULT_BOTTOM_CENTER_HORIZONTAL_MARGIN_RATIO: f32 = 0.12;
const DEFAULT_BOTTOM_CENTER_VERTICAL_MARGIN_RATIO: f32 = 0.20;
const MERGE_GAP_SECONDS: f64 = 0.1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitmapSubtitleKind {
    Pgs,
    VobSub,
}

#[derive(Debug, Clone)]
pub enum ExtractedSubtitleInput {
    Pgs {
        sup_path: PathBuf,
    },
    VobSub {
        idx_path: PathBuf,
        sub_path: PathBuf,
    },
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BitmapToWebVttOptions {
    /// Emit cue settings for every cue, including bottom-centered subtitles that would otherwise
    /// be left unstyled so players can use their default placement.
    pub position_all_cues: bool,
}

#[derive(Debug, Clone)]
pub struct ConvertedSubtitle {
    pub codec: BitmapSubtitleKind,
    pub language_bcp47: Option<String>,
    pub cue_count: usize,
    pub captions: Vec<WebVttCaption>,
    pub webvtt: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WebVttCaption {
    pub start: f64,
    pub end: f64,
    pub text: String,
    pub settings: Option<WebVttCueSettings>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WebVttCueSettings {
    pub line_percent: f32,
    pub position_percent: f32,
    pub size_percent: f32,
    pub align: &'static str,
}

#[derive(Debug, Clone)]
pub(crate) struct SubtitleFrame {
    pub start: f64,
    pub end: f64,
    pub image: GrayImage,
    pub positioning: Option<SubtitlePositioning>,
}

/// Convert an RGBA subtitle bitmap to a binary grayscale image (0=text, 255=background).
/// Called immediately after rendering/cropping so the RGBA buffer is freed as soon as possible.
pub(crate) fn convert_to_grayscale(rgba: RgbaImage) -> GrayImage {
    let (width, height) = rgba.dimensions();
    let pixels: Vec<u8> = rgba
        .into_raw()
        .chunks_exact(4)
        .map(|px| {
            let r = px[0] as u16;
            let g = px[1] as u16;
            let b = px[2] as u16;
            let a = px[3];
            let luma = ((77 * r + 150 * g + 29 * b) >> 8) as u8;
            if a >= A_THRESHOLD && luma >= L_THRESHOLD {
                0
            } else {
                255
            }
        })
        .collect();
    GrayImage::from_raw(width, height, pixels).expect("grayscale buffer size mismatch")
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SubtitlePositioning {
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub left: u32,
    pub top: u32,
}

pub async fn bitmap_to_webvtt(
    model_dir: &Path,
    subtitle_path: PathBuf,
    language_bcp47: &str,
    options: BitmapToWebVttOptions,
) -> Result<String> {
    let start = std::time::Instant::now();
    let frames = spawn_blocking(move || parse_pgs_frames(&subtitle_path))
        .await
        .context("subtitle frame parser panicked")??;
    tracing::info!("parsed {} frames in {:.2?}", frames.len(), start.elapsed());

    let converted = convert_frames_to_webvtt(
        BitmapSubtitleKind::Pgs,
        frames,
        Some(language_bcp47.to_owned()),
        model_dir.to_path_buf(),
        options,
    )
    .await?;

    Ok(converted.webvtt)
}

pub async fn convert_extracted_bitmap_subtitles_to_webvtt(
    codec: BitmapSubtitleKind,
    extracted_input: ExtractedSubtitleInput,
    video_size: (u32, u32),
    language_bcp47: Option<String>,
    model_dir: &Path,
    options: BitmapToWebVttOptions,
) -> Result<ConvertedSubtitle> {
    let start = std::time::Instant::now();
    let frames = spawn_blocking(move || match extracted_input {
        ExtractedSubtitleInput::Pgs { sup_path } => parse_pgs_frames(&sup_path),
        ExtractedSubtitleInput::VobSub { idx_path, sub_path } => {
            parse_vobsub_frames(&idx_path, &sub_path, video_size.0, video_size.1)
        }
    })
    .await
    .context("subtitle frame parser panicked")??;
    tracing::info!("parsed {} frames in {:.2?}", frames.len(), start.elapsed());

    convert_frames_to_webvtt(
        codec,
        frames,
        language_bcp47,
        model_dir.to_path_buf(),
        options,
    )
    .await
}

async fn convert_frames_to_webvtt(
    codec: BitmapSubtitleKind,
    frames: Vec<SubtitleFrame>,
    language_bcp47: Option<String>,
    model_dir: PathBuf,
    options: BitmapToWebVttOptions,
) -> Result<ConvertedSubtitle> {
    let (det_model, rec_model) = best_models_for_language(language_bcp47.as_deref());
    let (det_model_path, _) = upsert_model(&model_dir, det_model).await?;
    let (rec_model_path, dict_path) = upsert_model(&model_dir, rec_model).await?;
    let dict_path = dict_path.context("recognition model dictionary is missing")?;

    spawn_blocking(move || {
        // PrepackedWeights lets all sessions share the same packed model weights in memory.
        let weights = PrepackedWeights::new();
        let pool = OcrPool::new(
            &det_model_path,
            &rec_model_path,
            &dict_path,
            SESSION_POOL_SIZE,
            &weights,
        )?;

        let ocr_results = frames
            .par_iter()
            .map(|frame| -> Result<Option<WebVttCaption>> {
                let preprocessed = preprocess_for_ocr(&frame.image);
                let text = normalize_ocr_text(infer(&pool, &preprocessed)?);
                tracing::debug!(start = frame.start, end = frame.end, text = %text, "ocr result");

                #[cfg(feature = "dump-images")]
                dump_ocr_debug_artifacts(frame, &preprocessed, &text)?;

                if text.is_empty() {
                    return Ok(None);
                }

                Ok(Some(WebVttCaption {
                    start: frame.start,
                    end: frame.end,
                    text,
                    settings: cue_settings_for_frame(frame, options),
                }))
            })
            .collect::<Vec<_>>();

        let mut captions = Vec::with_capacity(ocr_results.len());
        for result in ocr_results {
            if let Some(caption) = result? {
                captions.push(caption);
            }
        }

        let captions = merge_adjacent_cues(captions);
        let webvtt = render_webvtt(&captions);
        let cue_count = captions.len();

        Ok::<ConvertedSubtitle, anyhow::Error>(ConvertedSubtitle {
            codec,
            language_bcp47,
            cue_count,
            captions,
            webvtt,
        })
    })
    .await
    .context("subtitle conversion worker panicked")?
}

fn cue_settings_for_frame(
    frame: &SubtitleFrame,
    options: BitmapToWebVttOptions,
) -> Option<WebVttCueSettings> {
    let positioning = frame.positioning?;
    let canvas_width = positioning.canvas_width.max(1) as f32;
    let canvas_height = positioning.canvas_height.max(1) as f32;
    let (image_width, image_height) = frame.image.dimensions();
    let image_width = image_width as f32;
    let image_height = image_height as f32;

    let left = positioning.left as f32;
    let top = positioning.top as f32;
    let right = left + image_width;
    let center_x = left + image_width / 2.0;
    let center_y = top + image_height / 2.0;

    if !options.position_all_cues
        && is_near_bottom_center(center_x, center_y, canvas_width, canvas_height)
    {
        return None;
    }

    let (position_percent, align) = if center_x <= canvas_width * 0.33 {
        ((left / canvas_width * 100.0).clamp(0.0, 95.0), "start")
    } else if center_x >= canvas_width * 0.67 {
        ((right / canvas_width * 100.0).clamp(5.0, 100.0), "end")
    } else {
        ((center_x / canvas_width * 100.0).clamp(5.0, 95.0), "center")
    };

    Some(WebVttCueSettings {
        line_percent: (top / canvas_height * 100.0).clamp(0.0, 95.0),
        position_percent,
        size_percent: (image_width / canvas_width * 100.0).clamp(10.0, 95.0),
        align,
    })
}

fn is_near_bottom_center(
    center_x: f32,
    center_y: f32,
    canvas_width: f32,
    canvas_height: f32,
) -> bool {
    let bottom_center_x = canvas_width / 2.0;
    let bottom_threshold_y = canvas_height * (1.0 - DEFAULT_BOTTOM_CENTER_VERTICAL_MARGIN_RATIO);
    let horizontal_margin = canvas_width * DEFAULT_BOTTOM_CENTER_HORIZONTAL_MARGIN_RATIO;

    (center_x - bottom_center_x).abs() <= horizontal_margin && center_y >= bottom_threshold_y
}

fn normalize_ocr_text(text: String) -> String {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn merge_adjacent_cues(cues: Vec<WebVttCaption>) -> Vec<WebVttCaption> {
    let mut merged: Vec<WebVttCaption> = Vec::with_capacity(cues.len());

    for cue in cues {
        if let Some(previous) = merged.last_mut() {
            let same_text = previous.text == cue.text;
            let same_settings = match (&previous.settings, &cue.settings) {
                (Some(left), Some(right)) => same_cue_settings(left, right),
                (None, None) => true,
                _ => false,
            };

            if same_text && same_settings && cue.start - previous.end <= MERGE_GAP_SECONDS {
                previous.end = cue.end;
                continue;
            }
        }

        merged.push(cue);
    }

    merged
}

fn same_cue_settings(left: &WebVttCueSettings, right: &WebVttCueSettings) -> bool {
    left.align == right.align
        && (left.line_percent - right.line_percent).abs() < 0.5
        && (left.position_percent - right.position_percent).abs() < 0.5
        && (left.size_percent - right.size_percent).abs() < 0.5
}

fn render_webvtt(cues: &[WebVttCaption]) -> String {
    let mut output = String::from("WEBVTT\n\n");

    for cue in cues {
        output.push_str(&format!(
            "{} --> {}",
            format_timestamp(cue.start),
            format_timestamp(cue.end)
        ));
        if let Some(settings) = &cue.settings {
            output.push(' ');
            output.push_str(&serialize_cue_settings(settings));
        }
        output.push('\n');
        output.push_str(&cue.text);
        output.push_str("\n\n");
    }

    output
}

fn serialize_cue_settings(settings: &WebVttCueSettings) -> String {
    format!(
        "line:{:.2}% position:{:.2}% size:{:.2}% align:{}",
        settings.line_percent, settings.position_percent, settings.size_percent, settings.align
    )
}

fn format_timestamp(seconds: f64) -> String {
    let total_millis = (seconds.max(0.0) * 1_000.0).round() as u64;
    let hours = total_millis / 3_600_000;
    let minutes = (total_millis % 3_600_000) / 60_000;
    let secs = (total_millis % 60_000) / 1_000;
    let millis = total_millis % 1_000;
    format!("{hours:02}:{minutes:02}:{secs:02}.{millis:03}")
}

#[cfg(feature = "dump-images")]
fn dump_ocr_debug_artifacts(
    frame: &SubtitleFrame,
    preprocessed: &RgbImage,
    text: &str,
) -> Result<()> {
    let dump_dir = PathBuf::from("frames");
    std::fs::create_dir_all(&dump_dir)
        .with_context(|| format!("failed to create {}", dump_dir.display()))?;

    let dump_path = dump_dir.join(format!("frame_{:07.2}_{:07.2}.png", frame.start, frame.end));
    preprocessed
        .save(&dump_path)
        .with_context(|| format!("failed to save {}", dump_path.display()))?;

    let text_path = dump_path.with_extension("txt");
    std::fs::write(&text_path, text)
        .with_context(|| format!("failed to write {}", text_path.display()))?;

    Ok(())
}

/// Pad a grayscale subtitle image with a white border, producing an RgbImage ready for the OCR
/// model. The border gives the det model room at the edges so text touching the frame isn't clipped.
fn preprocess_for_ocr(image: &GrayImage) -> RgbImage {
    let (src_w, src_h) = image.dimensions();
    let dst_w = src_w + PADDING * 2;
    let dst_h = src_h + PADDING * 2;

    let mut out = RgbImage::from_pixel(dst_w, dst_h, Rgb([255, 255, 255]));

    for (i, &v) in image.as_raw().iter().enumerate() {
        let x = (i as u32) % src_w + PADDING;
        let y = (i as u32) / src_w + PADDING;
        out.put_pixel(x, y, Rgb([v, v, v]));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::GrayImage;

    fn test_frame(left: u32, top: u32, canvas_width: u32, canvas_height: u32) -> SubtitleFrame {
        SubtitleFrame {
            start: 1.0,
            end: 2.0,
            image: GrayImage::from_pixel(200, 60, image::Luma([255])),
            positioning: Some(SubtitlePositioning {
                canvas_width,
                canvas_height,
                left,
                top,
            }),
        }
    }

    #[test]
    fn leaves_bottom_center_cues_unstyled_by_default() {
        let frame = test_frame(860, 950, 1920, 1080);
        assert!(cue_settings_for_frame(&frame, BitmapToWebVttOptions::default()).is_none());
    }

    #[test]
    fn exports_bottom_center_cues_when_requested() {
        let frame = test_frame(860, 950, 1920, 1080);
        let settings = cue_settings_for_frame(
            &frame,
            BitmapToWebVttOptions {
                position_all_cues: true,
            },
        );
        assert!(settings.is_some());
    }

    #[test]
    fn exports_off_center_cues_with_positioning() {
        let frame = test_frame(120, 180, 1920, 1080);
        let settings =
            cue_settings_for_frame(&frame, BitmapToWebVttOptions::default()).expect("settings");
        assert_eq!(settings.align, "start");
        assert!(settings.position_percent < 20.0);
        assert!(settings.line_percent < 20.0);
    }

    #[test]
    fn renders_valid_webvtt() {
        let output = render_webvtt(&[WebVttCaption {
            start: 1.0,
            end: 2.5,
            text: "hello".to_string(),
            settings: Some(WebVttCueSettings {
                line_percent: 80.0,
                position_percent: 50.0,
                size_percent: 40.0,
                align: "center",
            }),
        }]);

        assert!(output.starts_with("WEBVTT"));
        assert!(output.contains("00:00:01.000 --> 00:00:02.500"));
        assert!(output.contains("hello"));
    }
}
