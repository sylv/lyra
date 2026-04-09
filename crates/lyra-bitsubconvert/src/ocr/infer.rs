use super::pool::OcrPool;
use anyhow::{Context, Result, bail};
use image::{DynamicImage, RgbImage, imageops::FilterType};
use ndarray::{Array3, Array4};
use oar_ocr_core::{
    core::constants::{DEFAULT_MAX_IMG_WIDTH, DEFAULT_REC_IMAGE_SHAPE},
    processors::{BoundingBox, ImageScaleInfo, sort_quad_boxes},
    utils::BBoxCrop,
};
use ort::{inputs, value::Tensor};

#[derive(Debug)]
struct CroppedTextRegion {
    detection_index: usize,
    image: RgbImage,
    wh_ratio: f32,
}

/// Run OCR on a single image using lyra's pooled ORT sessions while mirroring oar-ocr's
/// detection resize/postprocess, rotated crop, and CRNN preprocessing as closely as possible.
pub fn infer(pool: &OcrPool, image: &RgbImage) -> Result<String> {
    let boxes = {
        let mut det = pool.det.acquire();
        let (det_tensor, img_shapes) = preprocess_detection(&det, image)?;
        let predictions = {
            let det_outputs = det.session.run(inputs![det_tensor])?;
            extract_array4_f32(&det_outputs[0], "det")?
        };
        let (boxes, _scores) = det.postprocessor.apply(&predictions, img_shapes, None);
        sort_quad_boxes(&boxes.into_iter().next().unwrap_or_default())
    };

    if boxes.is_empty() {
        return Ok(String::new());
    }

    let mut regions = crop_text_regions(image, &boxes);
    if regions.is_empty() {
        return Ok(String::new());
    }

    regions.sort_by(|a, b| {
        a.wh_ratio
            .partial_cmp(&b.wh_ratio)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut rec = pool.rec.acquire();
    let rec_tensor = preprocess_recognition(&regions)?;
    let predictions = {
        let rec_outputs = rec.session.run(inputs![rec_tensor])?;
        extract_array3_f32(&rec_outputs[0], "rec")?
    };
    let (texts, _scores) = rec.decoder.apply(&predictions);

    let mut ordered_lines: Vec<Option<String>> = vec![None; boxes.len()];
    for (region, text) in regions.iter().zip(texts.into_iter()) {
        if !text.is_empty() && region.detection_index < ordered_lines.len() {
            ordered_lines[region.detection_index] = Some(text);
        }
    }

    Ok(ordered_lines
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("\n"))
}

fn preprocess_detection(
    det: &super::pool::DetSession,
    image: &RgbImage,
) -> Result<(Tensor<f32>, Vec<ImageScaleInfo>)> {
    let (resized_images, img_shapes) = det.resizer.apply(
        vec![DynamicImage::ImageRgb8(image.clone())],
        None,
        None,
        None,
    );
    let batch_tensor = det.normalizer.normalize_batch_to(resized_images)?;
    let det_tensor = tensor_from_array4(batch_tensor)?;
    Ok((det_tensor, img_shapes))
}

fn crop_text_regions(image: &RgbImage, detection_boxes: &[BoundingBox]) -> Vec<CroppedTextRegion> {
    let mut regions = Vec::new();

    for (idx, bbox) in detection_boxes.iter().enumerate() {
        let Ok(crop) = BBoxCrop::crop_rotated_bounding_box(image, bbox) else {
            continue;
        };
        if crop.width() == 0 || crop.height() == 0 {
            continue;
        }

        let wh_ratio = crop.width() as f32 / crop.height().max(1) as f32;
        regions.push(CroppedTextRegion {
            detection_index: idx,
            image: crop,
            wh_ratio,
        });
    }

    regions
}

fn preprocess_recognition(regions: &[CroppedTextRegion]) -> Result<Tensor<f32>> {
    let [_img_c, img_h, img_w] = DEFAULT_REC_IMAGE_SHAPE;
    let base_ratio = img_w as f32 / img_h.max(1) as f32;
    let max_wh_ratio = regions
        .iter()
        .map(|region| region.wh_ratio)
        .fold(base_ratio, |acc, ratio| acc.max(ratio));
    let tensor_width = ((img_h as f32 * max_wh_ratio) as usize).min(DEFAULT_MAX_IMG_WIDTH);

    // Match oar-ocr's CRNN preprocessing: resize per crop, normalize in BGR order, then
    // zero-pad the batch to the widest crop in this recognition chunk.
    let mut batch_tensor = Array4::<f32>::zeros((regions.len(), 3, img_h, tensor_width));

    for (batch_idx, region) in regions.iter().enumerate() {
        let orig_w = region.image.width() as f32;
        let orig_h = region.image.height().max(1) as f32;
        let ratio = orig_w / orig_h;
        let resized_w = ((img_h as f32 * ratio).ceil() as usize).min(tensor_width);
        if resized_w == 0 {
            continue;
        }

        let resized = image::imageops::resize(
            &region.image,
            resized_w as u32,
            img_h as u32,
            FilterType::Triangle,
        );

        for y in 0..img_h {
            for x in 0..resized_w {
                let pixel = resized.get_pixel(x as u32, y as u32);
                batch_tensor[[batch_idx, 0, y, x]] = (pixel[2] as f32 / 255.0 - 0.5) / 0.5;
                batch_tensor[[batch_idx, 1, y, x]] = (pixel[1] as f32 / 255.0 - 0.5) / 0.5;
                batch_tensor[[batch_idx, 2, y, x]] = (pixel[0] as f32 / 255.0 - 0.5) / 0.5;
            }
        }
    }

    tensor_from_array4(batch_tensor)
}

fn tensor_from_array4(array: Array4<f32>) -> Result<Tensor<f32>> {
    let shape = array.shape();
    if shape.len() != 4 {
        bail!("unexpected tensor rank: {}", shape.len());
    }

    let data: Vec<f32> = array.iter().copied().collect();
    Tensor::from_array(([shape[0], shape[1], shape[2], shape[3]], data))
        .context("failed to build ORT tensor")
}

fn extract_array4_f32(output: &ort::value::DynValue, model: &str) -> Result<Array4<f32>> {
    let (shape, data) = output.try_extract_tensor::<f32>()?;
    if shape.len() != 4 {
        bail!("unexpected {model} output rank: {}", shape.len());
    }

    Array4::from_shape_vec(
        (
            shape[0] as usize,
            shape[1] as usize,
            shape[2] as usize,
            shape[3] as usize,
        ),
        data.to_vec(),
    )
    .with_context(|| format!("failed to reshape {model} output into Array4"))
}

fn extract_array3_f32(output: &ort::value::DynValue, model: &str) -> Result<Array3<f32>> {
    let (shape, data) = output.try_extract_tensor::<f32>()?;
    if shape.len() != 3 {
        bail!("unexpected {model} output rank: {}", shape.len());
    }

    Array3::from_shape_vec(
        (shape[0] as usize, shape[1] as usize, shape[2] as usize),
        data.to_vec(),
    )
    .with_context(|| format!("failed to reshape {model} output into Array3"))
}
