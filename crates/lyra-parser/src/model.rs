use lazy_static::lazy_static;
use ort::{inputs, session::Session, value::TensorRef};
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Instant;
use thiserror::Error;
use tokenizers::PaddingParams;
use tokenizers::Tokenizer;

#[derive(Debug, Clone, Serialize)]
pub struct Entity {
    pub label: String,
    pub start: usize,
    pub end: usize,
    pub text: String,
}

impl Entity {
    pub fn range(&self) -> core::ops::Range<usize> {
        self.start..self.end
    }
}

#[derive(Debug, Deserialize)]
struct ModelConfig {
    id2label: Option<HashMap<usize, String>>,
}

const MODEL_BYTES: &[u8] = include_bytes!("../model/model_quantized.onnx");
const TOKENIZER_BYTES: &[u8] = include_bytes!("../model/tokenizer.json");
const CONFIG_BYTES: &[u8] = include_bytes!("../model/config.json");

lazy_static! {
    static ref TOKENIZER: Tokenizer = {
        let mut tokenizer = Tokenizer::from_bytes(TOKENIZER_BYTES).unwrap();
        // todo: shoudl be from the config?
        tokenizer.with_padding(Some(PaddingParams::default()));
        tokenizer
    };
}

lazy_static! {
    static ref ID2LABEL: HashMap<usize, String> = {
        let cfg: ModelConfig = serde_json::from_slice(&CONFIG_BYTES).unwrap();
        cfg.id2label
            .expect("missing id2label in config.json")
            .clone()
    };
}

fn aggregate_entities(
    token_labels: &[String],
    offsets: &[(usize, usize)],
    text: &str,
) -> Vec<Entity> {
    let mut entities: Vec<Entity> = Vec::new();
    let mut current_entity: Option<(String, usize, usize)> = None;

    for (label_str, &(start, end)) in token_labels.iter().zip(offsets) {
        if start == end {
            continue;
        }

        if let Some(entity_label) = label_str.strip_prefix("I-") {
            if let Some((current_label, _, current_end)) = current_entity.as_mut() {
                if current_label == entity_label {
                    *current_end = end;
                    continue;
                }
            }
        }

        if let Some((label, start, end)) = current_entity.take() {
            entities.push(Entity {
                label,
                start,
                end,
                text: text[start..end].to_string(),
            });
        }

        if let Some(entity_label) = label_str.strip_prefix("B-") {
            current_entity = Some((entity_label.to_string(), start, end));
        }
    }

    if let Some((label, start, end)) = current_entity.take() {
        entities.push(Entity {
            label,
            start,
            end,
            text: text[start..end].to_string(),
        });
    }

    entities
}

fn argmax_last_dim(data: &[f32], seq_len: usize, num_labels: usize) -> Vec<usize> {
    let mut result = Vec::with_capacity(seq_len);
    for slice in data.chunks_exact(num_labels) {
        let mut max_idx = 0usize;
        let mut max_val = f32::NEG_INFINITY;
        for (i, &v) in slice.iter().enumerate() {
            if v > max_val {
                max_val = v;
                max_idx = i;
            }
        }
        result.push(max_idx);
    }
    result
}

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("tokenization failed: {0}")]
    TokenizationError(#[from] tokenizers::Error),
    #[error("inference failed: {0}")]
    InferenceError(#[from] ort::Error),
}

pub fn run_batched_inference(texts: &[String]) -> Result<Vec<Vec<Entity>>, ModelError> {
    if texts.is_empty() {
        return Ok(vec![]);
    }

    let start = Instant::now();
    let mut session = Session::builder()
        .unwrap()
        .commit_from_memory_directly(MODEL_BYTES)
        .unwrap();

    // Encode all inputs in batch with automatic padding
    let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
    let encodings = TOKENIZER
        .encode_batch(text_refs, true)
        .map_err(|e| ModelError::TokenizationError(e))?;

    let batch_size = texts.len();
    let max_seq_len = encodings
        .iter()
        .map(|enc| enc.get_ids().len())
        .max()
        .unwrap_or(0);

    // Prepare batched tensors
    let mut batch_ids = Vec::with_capacity(batch_size * max_seq_len);
    let mut batch_attention_mask = Vec::with_capacity(batch_size * max_seq_len);
    let mut batch_type_ids = Vec::with_capacity(batch_size * max_seq_len);
    let mut all_offsets = Vec::with_capacity(batch_size);

    for encoding in &encodings {
        let ids = encoding.get_ids();
        let attention_mask = encoding.get_attention_mask();
        let type_ids = encoding.get_type_ids();

        // Store offsets for this sequence
        let offsets: Vec<(usize, usize)> = encoding
            .get_offsets()
            .iter()
            .map(|(s, e)| (*s as usize, *e as usize))
            .collect();
        all_offsets.push(offsets);

        batch_ids.extend(ids.iter().map(|&id| id as i64));
        batch_attention_mask.extend(attention_mask.iter().map(|&m| m as i64));
        batch_type_ids.extend(type_ids.iter().map(|&t| t as i64));
    }

    let ids_tensor = TensorRef::from_array_view((
        vec![batch_size as i64, max_seq_len as i64],
        batch_ids.as_slice(),
    ))?;
    let mask_tensor = TensorRef::from_array_view((
        vec![batch_size as i64, max_seq_len as i64],
        batch_attention_mask.as_slice(),
    ))?;
    let type_ids_tensor = TensorRef::from_array_view((
        vec![batch_size as i64, max_seq_len as i64],
        batch_type_ids.as_slice(),
    ))
    .unwrap();

    let outputs = session.run(inputs![ids_tensor, mask_tensor, type_ids_tensor])?;

    let (dims, logits) = outputs["logits"].try_extract_tensor::<f32>()?;
    let seq = dims[1] as usize;
    let num_labels = dims[2] as usize;

    let mut results = Vec::with_capacity(batch_size);
    for (batch_idx, (text, offsets)) in texts.iter().zip(all_offsets.iter()).enumerate() {
        let start_idx = batch_idx * seq * num_labels;
        let end_idx = start_idx + seq * num_labels;
        let sequence_logits = &logits[start_idx..end_idx];

        let pred_ids = argmax_last_dim(sequence_logits, seq, num_labels);

        let token_labels: Vec<String> = pred_ids
            .iter()
            .map(|i| ID2LABEL.get(i).cloned().unwrap_or_else(|| "O".to_string()))
            .collect();

        let actual_seq_len = encodings[batch_idx].get_ids().len();
        let entities = aggregate_entities(
            &token_labels[..actual_seq_len],
            &offsets[..actual_seq_len],
            text,
        );
        results.push(entities);
    }

    tracing::debug!("model inference in {:?}", start.elapsed());
    Ok(results)
}
