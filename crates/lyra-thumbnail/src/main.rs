use anyhow::{Context, Result, bail};
use lyra_thumbnail::{ThumbnailOptions, generate_thumbnail};
use std::{env, path::PathBuf};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let input_file: PathBuf = env::args()
        .nth(1)
        .context("Please provide a video file path as the first argument")?
        .into();

    if !input_file.exists() {
        bail!("Input file does not exist: {:?}", input_file);
    }

    let thumbnail = generate_thumbnail(&input_file, &ThumbnailOptions::default(), None)
        .await?
        .expect("thumbnail CLI should not cancel");

    let output_path = env::current_dir()?.join("generated_thumbnail.webp");
    tokio::fs::write(&output_path, &thumbnail.image_bytes).await?;
    println!("wrote {}", output_path.display());
    println!("mime={}", thumbnail.mime_type);
    println!("dimensions={}x{}", thumbnail.width, thumbnail.height);

    Ok(())
}
