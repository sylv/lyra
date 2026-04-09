use anyhow::{Context, Result};
use lyra_bitsubconvert::{BitmapToWebVttOptions, bitmap_to_webvtt};
use std::path::PathBuf;
use tokio::fs;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let model_dir = PathBuf::from("models");
    let input_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("output.sup"));
    let output_path = input_path.with_extension("vtt");
    let webvtt = bitmap_to_webvtt(
        &model_dir,
        input_path,
        "en",
        BitmapToWebVttOptions::default(),
    )
    .await?;
    fs::write(&output_path, webvtt)
        .await
        .with_context(|| format!("failed to write {}", output_path.display()))?;

    Ok(())
}
